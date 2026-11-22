//! Optional full-text content index (OFF by default). Bounded: text-like files up to 256 KB,
//! at most ~2M distinct (file, word) pairs. Whole-word matching only.
use std::collections::{HashMap, HashSet};

pub const MAX_BYTES: u64 = 256 * 1024;
pub const MAX_PAIRS: usize = 2_000_000;
const EXTS: &[&str] = &["txt", "md", "markdown", "log", "csv", "json", "xml", "yml", "yaml", "toml", "ini", "cfg", "rs", "py", "js", "ts", "c", "h", "cpp", "hpp", "cs", "java", "html", "css", "ps1", "bat", "cmd", "sh"];

#[derive(Default)]
pub struct ContentIndex {
    pub files: HashMap<String, (u64, HashSet<String>)>,
    pub pairs: usize,
}

pub fn is_text_ext(name: &str) -> bool {
    name.rsplit_once('.').map(|(_, e)| EXTS.contains(&e.to_lowercase().as_str())).unwrap_or(false)
}

pub fn words(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric()).filter(|w| w.chars().count() >= 3).map(|w| w.to_lowercase()).collect()
}

impl ContentIndex {
    pub fn insert(&mut self, path: String, mtime: u64, w: HashSet<String>) {
        if self.pairs + w.len() > MAX_PAIRS {
            return;
        }
        self.pairs += w.len();
        if let Some((_, old)) = self.files.insert(path, (mtime, w)) {
            self.pairs = self.pairs.saturating_sub(old.len());
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        let toks: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
        if toks.is_empty() {
            return vec![];
        }
        self.files.iter().filter(|(_, (_, w))| toks.iter().all(|t| w.contains(t))).map(|(p, _)| p.clone()).take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_and_search() {
        let mut c = ContentIndex::default();
        c.insert("a.txt".into(), 1, words("Hello brave new World, it is 42!"));
        c.insert("b.txt".into(), 1, words("hello again"));
        assert_eq!(c.search("hello world", 10), vec!["a.txt".to_string()]);
        assert_eq!(c.search("hello", 10).len(), 2);
        assert!(c.search("zzz", 10).is_empty());
        assert!(is_text_ext("x.MD") && !is_text_ext("x.exe"));
    }
}
