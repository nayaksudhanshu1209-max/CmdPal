//! Canonical settings model (single source of truth) persisted as human-editable TOML.
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+Space";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    pub palette: PaletteSettings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaletteSettings {
    pub hotkey: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            palette: PaletteSettings::default(),
        }
    }
}

impl Default for PaletteSettings {
    fn default() -> Self {
        Self {
            hotkey: DEFAULT_HOTKEY.to_string(),
        }
    }
}

/// `%APPDATA%\CmdPal\settings.toml`
pub fn settings_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("CmdPal").join("settings.toml"))
}

pub fn parse(text: &str) -> Result<Settings, String> {
    let s: Settings = toml::from_str(text).map_err(|e| e.to_string())?;
    if s.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(format!(
            "settings schema_version {} is newer than this build supports ({})",
            s.schema_version, CURRENT_SCHEMA_VERSION
        ));
    }
    Ok(s)
}

/// Missing file => defaults. Unreadable/invalid file => error (caller must not overwrite it).
pub fn load_from(path: &Path) -> Result<Settings, String> {
    match fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.to_string()),
    }
}

/// Write to a temp file then rename over the target.
pub fn save_to(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = toml::to_string_pretty(settings).map_err(io::Error::other)?;
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_roundtrip() {
        let text = toml::to_string_pretty(&Settings::default()).unwrap();
        assert_eq!(parse(&text).unwrap(), Settings::default());
    }

    #[test]
    fn missing_fields_use_defaults() {
        assert_eq!(parse("").unwrap(), Settings::default());
    }

    #[test]
    fn invalid_toml_is_error() {
        assert!(parse("palette = [").is_err());
    }

    #[test]
    fn newer_schema_rejected() {
        assert!(parse("schema_version = 999").is_err());
    }

    #[test]
    fn save_and_load_file() {
        let dir = std::env::temp_dir().join(format!("cmdpal-test-{}", std::process::id()));
        let path = dir.join("settings.toml");
        let mut s = Settings::default();
        s.palette.hotkey = "Ctrl+Shift+P".into();
        save_to(&path, &s).unwrap();
        assert_eq!(load_from(&path).unwrap(), s);
        let _ = fs::remove_dir_all(dir);
    }
}
