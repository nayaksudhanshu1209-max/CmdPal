//! Tiny error log (no telemetry, never records note text or command arguments).
use std::io::Write;

pub fn log(msg: &str) {
    let Some(dir) = super::paths::local_dir() else { return };
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("cmdpal.log");
    if std::fs::metadata(&path).map(|m| m.len() > 256 * 1024).unwrap_or(false) {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{msg}");
    }
}
