//! Exclusion rules: subfolder paths, name globs, file extensions.
use crate::core::glob::glob_match;
use crate::core::settings::FileSettings;

#[derive(Debug, Clone, Default)]
pub struct Rules {
    globs: Vec<String>,
    path_prefixes: Vec<String>,
    exts: Vec<String>,
}

impl Rules {
    pub fn new(f: &FileSettings) -> Rules {
        let mut r = Rules::default();
        for p in f.exclude_patterns.iter().map(|p| p.trim()).filter(|p| !p.is_empty()) {
            if p.contains(['\\', '/', ':']) {
                r.path_prefixes.push(p.replace('/', "\\").trim_end_matches('\\').to_lowercase());
            } else {
                r.globs.push(p.to_string());
            }
        }
        r.exts = f.exclude_extensions.iter().map(|e| e.trim().trim_start_matches('.').to_lowercase()).filter(|e| !e.is_empty()).collect();
        r
    }

    pub fn excluded(&self, full_path: &str, name: &str, is_dir: bool) -> bool {
        if self.globs.iter().any(|g| glob_match(g, name)) {
            return true;
        }
        let lp = full_path.to_lowercase();
        if self.path_prefixes.iter().any(|p| lp == *p || lp.starts_with(&format!("{p}\\"))) {
            return true;
        }
        if !is_dir {
            if let Some((_, ext)) = name.rsplit_once('.') {
                let e = ext.to_lowercase();
                return self.exts.iter().any(|x| *x == e);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Rules {
        let mut f = FileSettings::default();
        f.exclude_patterns = vec!["node_modules".into(), "*.tmp".into(), r"C:\Data\Private".into()];
        f.exclude_extensions = vec![".LOG".into()];
        Rules::new(&f)
    }

    #[test]
    fn globs_paths_and_extensions() {
        let r = rules();
        assert!(r.excluded(r"C:\p\node_modules", "node_modules", true));
        assert!(r.excluded(r"C:\p\a.TMP", "a.TMP", false));
        assert!(r.excluded(r"c:\data\private\x.txt", "x.txt", false));
        assert!(r.excluded(r"C:\Data\Private", "Private", true));
        assert!(!r.excluded(r"C:\Data\PrivateStuff", "PrivateStuff", true));
        assert!(r.excluded(r"C:\p\run.log", "run.log", false));
        assert!(!r.excluded(r"C:\p\log", "log", true));
        assert!(!r.excluded(r"C:\p\a.txt", "a.txt", false));
    }
}
