//! Background incremental scanner: low priority, cancellable, bounded. A directory whose
//! modification time is unchanged reuses its cached listing instead of being re-read.
use super::content::{self, ContentIndex};
use super::control::{Control, Shared};
use super::rules::Rules;
use super::store::{self, DirRecord, Records};
use crate::core::contracts::RefreshHub;
use crate::core::settings::FileSettings;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_DEPTH: u32 = 40;

fn mtime(p: &Path) -> u64 {
    std::fs::metadata(p).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0)
}

pub struct ScanCtx<'a> {
    pub rules: &'a Rules,
    pub old: &'a Records,
    pub max_entries: usize,
    pub content_on: bool,
    pub aborted: &'a dyn Fn() -> bool,
}

#[derive(Default)]
pub struct ScanOut {
    pub records: Records,
    pub errors: Vec<String>,
    pub total: usize,
    pub content_files: Vec<(String, u64)>,
}

pub fn walk(dir: &str, depth: u32, cx: &ScanCtx, out: &mut ScanOut) {
    if depth > MAX_DEPTH || out.total >= cx.max_entries || (cx.aborted)() {
        return;
    }
    let path = Path::new(dir);
    let mt = mtime(path);
    let rec: Arc<DirRecord> = match cx.old.get(dir) {
        Some(r) if r.mtime == mt && mt != 0 => r.clone(),
        _ => {
            let mut rec = DirRecord { mtime: mt, ..Default::default() };
            match std::fs::read_dir(path) {
                Ok(rd) => {
                    for e in rd.flatten() {
                        let Ok(ft) = e.file_type() else { continue };
                        if ft.is_symlink() {
                            continue;
                        }
                        let name = e.file_name().to_string_lossy().into_owned();
                        let full = format!("{}\\{}", dir.trim_end_matches('\\'), name);
                        if cx.rules.excluded(&full, &name, ft.is_dir()) {
                            continue;
                        }
                        if ft.is_dir() {
                            rec.dirs.push(name);
                        } else {
                            rec.files.push(name);
                        }
                    }
                }
                Err(e) => {
                    if out.errors.len() < 20 {
                        out.errors.push(format!("{dir}: {:?}", e.kind()));
                    }
                }
            }
            Arc::new(rec)
        }
    };
    let room = cx.max_entries.saturating_sub(out.total);
    out.total += rec.files.len() + rec.dirs.len();
    if cx.content_on {
        for f in &rec.files {
            if content::is_text_ext(f) {
                let full = format!("{}\\{}", dir.trim_end_matches('\\'), f);
                out.content_files.push((full, 0));
            }
        }
    }
    let subs = rec.dirs.clone();
    out.records.insert(dir.to_string(), rec);
    if room == 0 {
        return;
    }
    for s in subs {
        walk(&format!("{}\\{}", dir.trim_end_matches('\\'), s), depth + 1, cx, out);
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn set_state(sh: &Shared, f: impl FnOnce(&mut super::control::Status)) {
    if let Ok(mut s) = sh.status.lock() {
        f(&mut s);
    }
}

pub fn run(
    sh: Arc<Shared>,
    cfg: Arc<Mutex<FileSettings>>,
    content_idx: Arc<RwLock<ContentIndex>>,
    cache_path: Option<std::path::PathBuf>,
    refresh: Arc<RefreshHub>,
) {
    crate::platform::windows::proc::begin_background_thread();
    if let Some(p) = &cache_path {
        let loaded = store::load(p);
        let n = store::count(&loaded);
        if let Ok(mut r) = sh.records.write() {
            *r = loaded;
        }
        set_state(&sh, |s| s.entries = n);
    }
    // Initial delay keeps startup quiet.
    let mut first = true;
    loop {
        // wait for work
        {
            let Ok(mut c) = sh.control.lock() else { return };
            let interval = cfg.lock().map(|c| c.scan_interval_minutes).unwrap_or(0);
            let timeout = if first { Some(Duration::from_secs(4)) } else if interval > 0 { Some(Duration::from_secs(interval as u64 * 60)) } else { None };
            if !c.rescan && !c.clear && !c.stop {
                match timeout {
                    Some(t) => {
                        c = sh.cv.wait_timeout(c, t).map(|(g, _)| g).unwrap_or_else(|e| e.into_inner().0);
                    }
                    None => {
                        c = sh.cv.wait(c).unwrap_or_else(|e| e.into_inner());
                    }
                }
            }
            if c.stop {
                return;
            }
            if c.clear {
                c.clear = false;
                drop(c);
                if let Ok(mut r) = sh.records.write() {
                    r.clear();
                }
                if let Ok(mut ci) = content_idx.write() {
                    *ci = ContentIndex::default();
                }
                if let Some(p) = &cache_path {
                    let _ = std::fs::remove_file(p);
                }
                set_state(&sh, |s| {
                    s.entries = 0;
                    s.last_scan = None;
                    s.errors.clear();
                    s.state = "Cleared".into();
                });
                refresh.trigger();
                continue;
            }
            if c.paused {
                let st: &mut Control = &mut c;
                st.rescan = false;
                drop(c);
                set_state(&sh, |s| s.state = "Paused".into());
                first = false;
                continue;
            }
        }
        first = false;
        let full = if let Ok(mut c) = sh.control.lock() {
            c.rescan = false;
            std::mem::take(&mut c.full)
        } else {
            false
        };
        let fs = cfg.lock().map(|c| c.clone()).unwrap_or_default();
        let roots: Vec<String> = fs.roots.iter().filter(|r| r.enabled && !r.path.trim().is_empty()).map(|r| r.path.trim().trim_end_matches('\\').to_string()).collect();
        if roots.is_empty() {
            set_state(&sh, |s| s.state = "Idle (no folders to index)".into());
            continue;
        }
        set_state(&sh, |s| {
            s.state = "Scanning".into();
            s.errors.clear();
        });
        let old: Records = if full { HashMap::new() } else { sh.records.read().map(|r| r.clone()).unwrap_or_default() };
        let rules = Rules::new(&fs);
        let aborted = || sh.control.lock().map(|c| c.stop || c.paused || c.clear || c.rescan).unwrap_or(true);
        let cx = ScanCtx { rules: &rules, old: &old, max_entries: fs.max_entries as usize, content_on: fs.content_index, aborted: &aborted };
        let mut out = ScanOut::default();
        for r in &roots {
            walk(r, 0, &cx, &mut out);
        }
        if aborted() {
            // Cancelled/paused/changed: keep the previous index; loop will react to the flag.
            set_state(&sh, |s| s.state = "Interrupted".into());
            continue;
        }
        if fs.content_index {
            let mut ci = ContentIndex::default();
            let prev = content_idx.read().ok();
            for (path, _) in &out.content_files {
                if aborted() || ci.pairs >= content::MAX_PAIRS {
                    break;
                }
                let Ok(md) = std::fs::metadata(path) else { continue };
                if md.len() > content::MAX_BYTES {
                    continue;
                }
                let mt = md.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
                if let Some((pm, w)) = prev.as_ref().and_then(|p| p.files.get(path)) {
                    if *pm == mt {
                        ci.insert(path.clone(), mt, w.clone());
                        continue;
                    }
                }
                if let Ok(bytes) = std::fs::read(path) {
                    ci.insert(path.clone(), mt, content::words(&String::from_utf8_lossy(&bytes)));
                }
            }
            drop(prev);
            if let Ok(mut w) = content_idx.write() {
                *w = ci;
            }
        } else if let Ok(mut w) = content_idx.write() {
            *w = ContentIndex::default();
        }
        let total = store::count(&out.records);
        if let Some(p) = &cache_path {
            store::save(p, &out.records);
        }
        if let Ok(mut r) = sh.records.write() {
            *r = out.records;
        }
        let truncated = total >= fs.max_entries as usize;
        set_state(&sh, |s| {
            s.state = "Idle".into();
            s.entries = total;
            s.last_scan = Some(now_secs());
            s.errors = out.errors;
            if truncated {
                s.errors.push(format!("entry limit ({}) reached; raise max_entries to index more", fs.max_entries));
            }
        });
        refresh.trigger();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_excludes_and_reuses_unchanged_dirs() {
        let base = std::env::temp_dir().join(format!("cmdpal-scan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("sub")).unwrap();
        std::fs::create_dir_all(base.join("node_modules")).unwrap();
        std::fs::write(base.join("a.txt"), "x").unwrap();
        std::fs::write(base.join("sub").join("b.md"), "y").unwrap();
        std::fs::write(base.join("skip.tmp"), "z").unwrap();
        let fs = FileSettings::default();
        let rules = Rules::new(&fs);
        let never = || false;
        let old = Records::new();
        let cx = ScanCtx { rules: &rules, old: &old, max_entries: 1000, content_on: false, aborted: &never };
        let mut out = ScanOut::default();
        let root = base.to_string_lossy().into_owned();
        walk(&root, 0, &cx, &mut out);
        assert_eq!(out.records.len(), 2); // root + sub (node_modules excluded)
        let rec = &out.records[&root];
        assert_eq!(rec.files, vec!["a.txt".to_string()]);
        assert_eq!(rec.dirs, vec!["sub".to_string()]);
        // second pass: same Arc reused for unchanged dirs
        let old2 = out.records.clone();
        let cx2 = ScanCtx { rules: &rules, old: &old2, max_entries: 1000, content_on: false, aborted: &never };
        let mut out2 = ScanOut::default();
        walk(&root, 0, &cx2, &mut out2);
        assert!(Arc::ptr_eq(&old2[&root], &out2.records[&root]));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn respects_entry_limit() {
        let base = std::env::temp_dir().join(format!("cmdpal-scan-lim-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        for i in 0..20 {
            std::fs::write(base.join(format!("f{i}.dat")), "x").unwrap();
        }
        let fs = FileSettings::default();
        let rules = Rules::new(&fs);
        let never = || false;
        let old = Records::new();
        let cx = ScanCtx { rules: &rules, old: &old, max_entries: 5, content_on: false, aborted: &never };
        let mut out = ScanOut::default();
        walk(&base.to_string_lossy(), 0, &cx, &mut out);
        assert!(out.total >= 5);
        let _ = std::fs::remove_dir_all(&base);
    }
}
