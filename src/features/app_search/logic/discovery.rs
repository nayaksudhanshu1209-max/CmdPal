//! Discovers apps from Start Menu shortcut folders (documented, no registry scraping).
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
}

pub fn is_uninstaller(name: &str) -> bool {
    let l = name.to_lowercase();
    l.contains("uninstall") || l.starts_with("remove ")
}

/// Removes uninstallers and duplicate names (first wins), sorted by name.
pub fn clean(mut v: Vec<AppEntry>) -> Vec<AppEntry> {
    v.retain(|a| !is_uninstaller(&a.name));
    let mut seen = std::collections::HashSet::new();
    v.retain(|a| seen.insert(a.name.to_lowercase()));
    v.sort_by_key(|a| a.name.to_lowercase());
    v
}

fn walk(dir: &Path, depth: u32, out: &mut Vec<AppEntry>) {
    if depth > 6 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            walk(&p, depth + 1, out);
        } else if p.extension().map(|x| x.eq_ignore_ascii_case("lnk")).unwrap_or(false) {
            if let Some(stem) = p.file_stem() {
                out.push(AppEntry { name: stem.to_string_lossy().into_owned(), path: p.to_string_lossy().into_owned() });
            }
        }
    }
}

pub fn start_menu_roots() -> Vec<PathBuf> {
    let mut v = vec![];
    for (var, tail) in [("APPDATA", r"Microsoft\Windows\Start Menu\Programs"), ("ProgramData", r"Microsoft\Windows\Start Menu\Programs")] {
        if let Some(b) = std::env::var_os(var) {
            v.push(PathBuf::from(b).join(tail));
        }
    }
    v
}

pub fn discover() -> Vec<AppEntry> {
    let mut out = vec![];
    for r in start_menu_roots() {
        walk(&r, 0, &mut out);
    }
    clean(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(n: &str) -> AppEntry {
        AppEntry { name: n.into(), path: format!("{n}.lnk") }
    }

    #[test]
    fn cleans_duplicates_and_uninstallers() {
        let v = clean(vec![e("Notepad"), e("notepad"), e("Uninstall Foo"), e("Alpha")]);
        assert_eq!(v.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), vec!["Alpha", "Notepad"]);
    }
}
