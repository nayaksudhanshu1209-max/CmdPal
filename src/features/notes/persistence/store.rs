//! Local Markdown note storage with crash-safe saves.
//! Save = write `name.md.new` fully + sync, keep `name.md.bak` (previous version), then atomically
//! rename over `name.md`. A reader never sees a half-written `.md`; if `.md` is missing after a
//! crash, `.new` or `.bak` is recovered.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const RESERVED: &[&str] = &["con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "lpt1", "lpt2", "lpt3"];

pub fn sanitize_name(raw: &str) -> String {
    let mut s: String = raw.chars().map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { ' ' } else { c }).collect();
    s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let s = s.trim_matches(|c: char| c == '.' || c == ' ').to_string();
    let s: String = s.chars().take(80).collect();
    if s.is_empty() || RESERVED.contains(&s.to_lowercase().as_str()) {
        "Untitled".into()
    } else {
        s
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteInfo {
    pub name: String,
    pub modified: u64,
}

pub struct Store {
    pub dir: PathBuf,
}

fn secs(p: &Path) -> u64 {
    fs::metadata(p).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0)
}

impl Store {
    pub fn default_store() -> Option<Store> {
        crate::core::paths::notes_dir().map(|dir| Store { dir })
    }

    fn md(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{name}.md"))
    }
    fn ext(&self, name: &str, e: &str) -> PathBuf {
        self.dir.join(format!("{name}.md.{e}"))
    }
    fn toolbar_file(&self, name: &str) -> PathBuf {
        self.dir.join(".toolbar").join(format!("{name}.txt"))
    }

    pub fn exists(&self, name: &str) -> bool {
        self.md(name).exists()
    }

    pub fn unique_name(&self, base: &str) -> String {
        let base = sanitize_name(base);
        if !self.exists(&base) {
            return base;
        }
        (2..).map(|n| format!("{base} {n}")).find(|n| !self.exists(n)).unwrap_or(base)
    }

    pub fn list(&self) -> Vec<NoteInfo> {
        let mut v: Vec<NoteInfo> = fs::read_dir(&self.dir)
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| {
                        let p = e.path();
                        let n = p.file_name()?.to_string_lossy().into_owned();
                        let stem = n.strip_suffix(".md")?.to_string();
                        Some(NoteInfo { name: stem, modified: secs(&p) })
                    })
                    .collect()
            })
            .unwrap_or_default();
        v.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.name.cmp(&b.name)));
        v
    }

    /// Reads a note, recovering from `.new` / `.bak` if the main file is missing.
    pub fn read(&self, name: &str) -> Result<String, String> {
        for p in [self.md(name), self.ext(name, "new"), self.ext(name, "bak")] {
            if let Ok(bytes) = fs::read(&p) {
                return Ok(String::from_utf8_lossy(&bytes).into_owned());
            }
        }
        Err(format!("note '{name}' could not be read"))
    }

    pub fn create_empty(&self, name: &str) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        if !self.exists(name) {
            self.save(name, "", false)?;
        }
        Ok(())
    }

    pub fn save(&self, name: &str, text: &str, keep_backup: bool) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let new = self.ext(name, "new");
        {
            let mut f = fs::File::create(&new).map_err(|e| e.to_string())?;
            f.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
            f.sync_all().map_err(|e| e.to_string())?;
        }
        let md = self.md(name);
        if keep_backup && md.exists() {
            let _ = fs::copy(&md, self.ext(name, "bak"));
        }
        fs::rename(&new, &md).map_err(|e| e.to_string())
    }

    pub fn rename(&self, old: &str, new_raw: &str) -> Result<String, String> {
        let new = sanitize_name(new_raw);
        if new == old {
            return Ok(new);
        }
        if self.exists(&new) {
            return Err(format!("a note named '{new}' already exists"));
        }
        fs::rename(self.md(old), self.md(&new)).map_err(|e| e.to_string())?;
        for e in ["bak", "new"] {
            let _ = fs::rename(self.ext(old, e), self.ext(&new, e));
        }
        let _ = fs::rename(self.toolbar_file(old), self.toolbar_file(&new));
        Ok(new)
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        fs::remove_file(self.md(name)).map_err(|e| e.to_string())?;
        for e in ["bak", "new"] {
            let _ = fs::remove_file(self.ext(name, e));
        }
        let _ = fs::remove_file(self.toolbar_file(name));
        Ok(())
    }

    pub fn load_toolbar(&self, name: &str) -> Vec<String> {
        fs::read_to_string(self.toolbar_file(name)).map(|t| t.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()).unwrap_or_default()
    }

    pub fn save_toolbar(&self, name: &str, order: &[String]) {
        let p = self.toolbar_file(name);
        if let Some(d) = p.parent() {
            let _ = fs::create_dir_all(d);
        }
        let _ = fs::write(p, order.join("\n"));
    }

    /// Case-insensitive substring search in a note's text (first 256 KB).
    pub fn contains_text(&self, name: &str, needle_lower: &str) -> bool {
        fs::read(self.md(name)).map(|b| String::from_utf8_lossy(&b[..b.len().min(256 * 1024)]).to_lowercase().contains(needle_lower)).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(tag: &str) -> Store {
        let dir = std::env::temp_dir().join(format!("cmdpal-notes-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        Store { dir }
    }

    #[test]
    fn sanitizes_names() {
        assert_eq!(sanitize_name("a/b:c*?"), "a b c");
        assert_eq!(sanitize_name("  ..  "), "Untitled");
        assert_eq!(sanitize_name("CON"), "Untitled");
        assert_eq!(sanitize_name(&"x".repeat(200)).len(), 80);
    }

    #[test]
    fn save_read_backup_and_unique() {
        let s = store("a");
        s.save("n", "one", true).unwrap();
        s.save("n", "two", true).unwrap();
        assert_eq!(s.read("n").unwrap(), "two");
        assert_eq!(fs::read_to_string(s.ext("n", "bak")).unwrap(), "one");
        assert_eq!(s.unique_name("n"), "n 2");
        assert_eq!(s.list().len(), 1);
        assert!(s.contains_text("n", "tw"));
        let _ = fs::remove_dir_all(&s.dir);
    }

    #[test]
    fn recovers_after_crash() {
        let s = store("b");
        s.save("n", "good", true).unwrap();
        fs::write(s.ext("n", "new"), "half-writt").unwrap();
        assert_eq!(s.read("n").unwrap(), "good");
        fs::remove_file(s.md("n")).unwrap();
        assert!(s.read("n").is_ok());
        let _ = fs::remove_dir_all(&s.dir);
    }

    #[test]
    fn rename_delete_toolbar() {
        let s = store("c");
        s.save("a", "x", false).unwrap();
        s.save("b", "y", false).unwrap();
        s.save_toolbar("a", &["bold".into(), "italic".into()]);
        assert!(s.rename("a", "b").is_err());
        assert_eq!(s.rename("a", "c").unwrap(), "c");
        assert_eq!(s.load_toolbar("c"), vec!["bold".to_string(), "italic".to_string()]);
        s.delete("c").unwrap();
        assert!(!s.exists("c"));
        let _ = fs::remove_dir_all(&s.dir);
    }
}
