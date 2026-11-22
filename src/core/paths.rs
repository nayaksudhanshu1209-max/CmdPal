//! Locations of CmdPal's local data. All data stays on this PC.
use std::path::PathBuf;

fn env_dir(var: &str) -> Option<PathBuf> {
    std::env::var_os(var).map(|v| PathBuf::from(v).join("CmdPal"))
}

/// `%APPDATA%\CmdPal` - settings and notes.
pub fn data_dir() -> Option<PathBuf> {
    env_dir("APPDATA")
}

/// `%LOCALAPPDATA%\CmdPal` - caches (app list, file index) and log.
pub fn local_dir() -> Option<PathBuf> {
    env_dir("LOCALAPPDATA")
}

pub fn notes_dir() -> Option<PathBuf> {
    data_dir().map(|d| d.join("notes"))
}

/// Desktop / Documents / Downloads under the user profile (offered individually in settings).
pub fn known_folder(name: &str) -> Option<String> {
    std::env::var_os("USERPROFILE")
        .map(|p| PathBuf::from(p).join(name).to_string_lossy().into_owned())
}
