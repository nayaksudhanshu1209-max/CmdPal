//! On-disk app list so the first keystrokes work before the background scan finishes.
use super::discovery::AppEntry;
use std::path::Path;

pub fn encode(apps: &[AppEntry]) -> String {
    apps.iter().map(|a| format!("{}\t{}\n", a.name.replace(['\t', '\n'], " "), a.path.replace(['\t', '\n'], " "))).collect()
}

pub fn decode(text: &str) -> Vec<AppEntry> {
    text.lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(n, p)| AppEntry { name: n.into(), path: p.into() })
        .collect()
}

pub fn load(path: &Path) -> Vec<AppEntry> {
    std::fs::read_to_string(path).map(|t| decode(&t)).unwrap_or_default()
}

pub fn save(path: &Path, apps: &[AppEntry]) {
    if let Some(d) = path.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, encode(apps)).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let v = vec![AppEntry { name: "A b".into(), path: r"C:\x\A b.lnk".into() }];
        assert_eq!(decode(&encode(&v)), v);
    }
}
