//! Index data: per-directory records (name lists + modification time) enabling incremental
//! rescans, plus a simple TSV persistence format.
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DirRecord {
    pub mtime: u64,
    pub files: Vec<String>,
    pub dirs: Vec<String>,
}

pub type Records = HashMap<String, Arc<DirRecord>>;

pub fn count(r: &Records) -> usize {
    r.values().map(|d| d.files.len() + d.dirs.len()).sum()
}

pub fn encode(r: &Records) -> String {
    let mut s = String::new();
    for (p, d) in r {
        s.push_str(&format!("D\t{p}\t{}\n", d.mtime));
        for f in &d.files {
            s.push_str(&format!("F\t{f}\n"));
        }
        for sd in &d.dirs {
            s.push_str(&format!("S\t{sd}\n"));
        }
    }
    s
}

pub fn decode(text: &str) -> Records {
    let mut out = Records::new();
    let mut cur: Option<(String, DirRecord)> = None;
    let mut flush = |c: &mut Option<(String, DirRecord)>, out: &mut Records| {
        if let Some((p, d)) = c.take() {
            out.insert(p, Arc::new(d));
        }
    };
    for line in text.lines() {
        let mut parts = line.splitn(3, '\t');
        match (parts.next(), parts.next(), parts.next()) {
            (Some("D"), Some(p), Some(m)) => {
                flush(&mut cur, &mut out);
                cur = Some((p.to_string(), DirRecord { mtime: m.parse().unwrap_or(0), ..Default::default() }));
            }
            (Some("F"), Some(n), None) => {
                if let Some((_, d)) = cur.as_mut() {
                    d.files.push(n.to_string());
                }
            }
            (Some("S"), Some(n), None) => {
                if let Some((_, d)) = cur.as_mut() {
                    d.dirs.push(n.to_string());
                }
            }
            _ => {}
        }
    }
    flush(&mut cur, &mut out);
    out
}

pub fn save(path: &Path, r: &Records) {
    if let Some(d) = path.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, encode(r)).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

pub fn load(path: &Path) -> Records {
    std::fs::read_to_string(path).map(|t| decode(&t)).unwrap_or_default()
}

/// ASCII-fast, allocation-free case-insensitive substring test; `needle_lower` must be lowercase.
pub fn ci_contains(hay: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    if hay.is_ascii() && needle_lower.is_ascii() {
        let (h, n) = (hay.as_bytes(), needle_lower.as_bytes());
        return h.len() >= n.len() && h.windows(n.len()).any(|w| w.iter().zip(n).all(|(a, b)| a.to_ascii_lowercase() == *b));
    }
    hay.to_lowercase().contains(needle_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut r = Records::new();
        r.insert(r"C:\a".into(), Arc::new(DirRecord { mtime: 5, files: vec!["x.txt".into(), "y z.md".into()], dirs: vec!["sub".into()] }));
        r.insert(r"C:\a\sub".into(), Arc::new(DirRecord::default()));
        let d = decode(&encode(&r));
        assert_eq!(d, r);
        assert_eq!(count(&d), 3);
    }

    #[test]
    fn contains_ci() {
        assert!(ci_contains("Quarterly Report.XLSX", "report"));
        assert!(!ci_contains("abc", "abcd"));
        assert!(ci_contains("Überblick.txt", "über"));
        assert!(ci_contains("x", ""));
    }
}
