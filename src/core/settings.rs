//! Canonical settings model: the single source of truth shared by the settings GUI and the
//! human-editable `%APPDATA%\CmdPal\settings.toml`.
use super::{hotkey, url};
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
    /// Keys of results the user hid (apps and other hideable results).
    pub exclusions: Vec<String>,
    pub default_search_alias: String,
    pub bangs: Vec<BangAlias>,
    pub bookmarks: Vec<Bookmark>,
    pub commands: Vec<UserCommand>,
    pub files: FileSettings,
    pub quick_folders: Vec<QuickFolder>,
    pub notes: NoteSettings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaletteSettings {
    pub hotkey: String,
    pub launch_at_startup: bool,
    pub position_near_cursor: bool,
    pub show_suggestions_at_startup: bool,
    pub compact_mode: bool,
    pub animations: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BangAlias {
    pub alias: String,
    pub name: String,
    /// URL template containing `{query}`.
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UserCommand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub program: String,
    /// Each element is passed as one argument; `{input}` is replaced by the typed input.
    pub args: Vec<String>,
    pub working_dir: String,
    pub confirm: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileSettings {
    pub roots: Vec<FileRoot>,
    /// Names/globs (`*.tmp`, `node_modules`) or full folder paths to skip.
    pub exclude_patterns: Vec<String>,
    /// Extensions without dot, e.g. `tmp`.
    pub exclude_extensions: Vec<String>,
    /// Full-text content indexing. Off by default (costs RAM, CPU and disk reads).
    pub content_index: bool,
    /// 0 = only scan at startup / on demand.
    pub scan_interval_minutes: u32,
    pub max_entries: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileRoot {
    pub path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct QuickFolder {
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NoteSettings {
    pub alias: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            palette: PaletteSettings::default(),
            exclusions: vec![],
            default_search_alias: "g".into(),
            bangs: vec![
                bang("g", "Google", "https://www.google.com/search?q={query}"),
                bang("yt", "YouTube", "https://www.youtube.com/results?search_query={query}"),
                bang("w", "Wikipedia", "https://en.wikipedia.org/w/index.php?search={query}"),
            ],
            bookmarks: vec![],
            commands: vec![],
            files: FileSettings::default(),
            quick_folders: vec![],
            notes: NoteSettings::default(),
        }
    }
}

fn bang(alias: &str, name: &str, url: &str) -> BangAlias {
    BangAlias { alias: alias.into(), name: name.into(), url: url.into(), enabled: true }
}

impl Default for PaletteSettings {
    fn default() -> Self {
        Self {
            hotkey: DEFAULT_HOTKEY.into(),
            launch_at_startup: false,
            position_near_cursor: false,
            show_suggestions_at_startup: true,
            compact_mode: false,
            animations: true,
        }
    }
}
impl Default for BangAlias {
    fn default() -> Self {
        Self { alias: String::new(), name: String::new(), url: String::new(), enabled: true }
    }
}
impl Default for Bookmark {
    fn default() -> Self {
        Self { name: String::new(), url: String::new(), enabled: true }
    }
}
impl Default for UserCommand {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            program: String::new(),
            args: vec![],
            working_dir: String::new(),
            confirm: true,
            enabled: true,
        }
    }
}
impl Default for FileSettings {
    fn default() -> Self {
        Self {
            roots: vec![],
            exclude_patterns: vec![".git".into(), "node_modules".into(), "*.tmp".into()],
            exclude_extensions: vec![],
            content_index: false,
            scan_interval_minutes: 30,
            max_entries: 100_000,
        }
    }
}
impl Default for FileRoot {
    fn default() -> Self {
        Self { path: String::new(), enabled: true }
    }
}
impl Default for QuickFolder {
    fn default() -> Self {
        Self { name: String::new(), path: String::new(), enabled: true }
    }
}
impl Default for NoteSettings {
    fn default() -> Self {
        Self { alias: "n".into() }
    }
}

/// `%APPDATA%\CmdPal\settings.toml`
pub fn settings_path() -> Option<PathBuf> {
    super::paths::data_dir().map(|d| d.join("settings.toml"))
}

pub fn parse(text: &str) -> Result<Settings, String> {
    let s: Settings = toml::from_str(text).map_err(|e| e.to_string())?;
    if s.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(format!(
            "settings schema_version {} is newer than this build supports ({})",
            s.schema_version, CURRENT_SCHEMA_VERSION
        ));
    }
    let errs = validate(&s);
    if errs.is_empty() { Ok(s) } else { Err(errs.join("\n")) }
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

/// Moves an unreadable settings file aside (`settings.toml.bad`) so defaults can be used safely.
pub fn quarantine(path: &Path) -> io::Result<()> {
    fs::rename(path, path.with_extension("toml.bad"))
}

fn bad_alias(a: &str) -> bool {
    a.is_empty() || a.chars().any(|c| c.is_whitespace())
}

/// Validation for GUI edits and hand-edited files. Empty result = valid.
pub fn validate(s: &Settings) -> Vec<String> {
    let mut e = vec![];
    if let Err(m) = hotkey::parse(&s.palette.hotkey) {
        e.push(format!("hotkey '{}': {m}", s.palette.hotkey));
    }
    let mut seen: Vec<String> = vec![];
    for b in s.bangs.iter().filter(|b| b.enabled) {
        if bad_alias(&b.alias) {
            e.push(format!("bang alias '{}' must be non-empty without spaces", b.alias));
        }
        if b.name.trim().is_empty() {
            e.push(format!("bang '{}' needs a display name", b.alias));
        }
        if url::build_search_url(&b.url, "test").is_none() {
            e.push(format!("bang '{}': URL must be http(s) and contain {{query}} outside the host", b.alias));
        }
        let l = b.alias.to_lowercase();
        if seen.contains(&l) {
            e.push(format!("duplicate bang alias '{}'", b.alias));
        }
        seen.push(l);
    }
    if bad_alias(&s.notes.alias) {
        e.push("note alias must be non-empty without spaces".into());
    } else if seen.contains(&s.notes.alias.to_lowercase()) {
        e.push(format!("note alias '{}' collides with a bang alias", s.notes.alias));
    }
    if !s.default_search_alias.is_empty()
        && !s.bangs.iter().any(|b| b.enabled && b.alias.eq_ignore_ascii_case(&s.default_search_alias))
    {
        e.push(format!("default search alias '{}' is not an enabled bang", s.default_search_alias));
    }
    for b in &s.bookmarks {
        if b.name.trim().is_empty() || !url::is_valid_http_url(&b.url) {
            e.push(format!("bookmark '{}' needs a name and a valid http(s) URL", b.name));
        }
    }
    let mut ids: Vec<String> = vec![];
    for c in &s.commands {
        if c.id.trim().is_empty() || c.id.chars().any(|ch| ch.is_whitespace()) {
            e.push(format!("command '{}' needs an id without spaces", c.name));
        }
        if c.name.trim().is_empty() || c.program.trim().is_empty() {
            e.push(format!("command '{}' needs a name and a program", c.id));
        }
        if ids.contains(&c.id.to_lowercase()) {
            e.push(format!("duplicate command id '{}'", c.id));
        }
        ids.push(c.id.to_lowercase());
    }
    for q in &s.quick_folders {
        if q.name.trim().is_empty() || q.path.trim().is_empty() {
            e.push("quick folders need a name and a path".into());
        }
    }
    for r in &s.files.roots {
        if r.path.trim().is_empty() {
            e.push("an indexing root has an empty path".into());
        }
    }
    if s.files.scan_interval_minutes > 10_080 {
        e.push("scan_interval_minutes must be at most 10080".into());
    }
    if !(1_000..=2_000_000).contains(&s.files.max_entries) {
        e.push("max_entries must be between 1000 and 2000000".into());
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_roundtrip() {
        assert!(validate(&Settings::default()).is_empty());
        let text = toml::to_string_pretty(&Settings::default()).unwrap();
        assert_eq!(parse(&text).unwrap(), Settings::default());
    }

    #[test]
    fn missing_fields_use_defaults() {
        assert_eq!(parse("").unwrap(), Settings::default());
        let s = parse("[palette]\ncompact_mode = true").unwrap();
        assert!(s.palette.compact_mode && s.palette.animations);
    }

    #[test]
    fn invalid_files_are_rejected() {
        assert!(parse("palette = [").is_err());
        assert!(parse("schema_version = 999").is_err());
        assert!(parse("[palette]\nhotkey = \"Space\"").is_err());
    }

    #[test]
    fn validation_catches_problems() {
        let mut s = Settings::default();
        s.bangs.push(bang("G", "dup", "https://x.com/?q={query}"));
        assert!(!validate(&s).is_empty());
        let mut s = Settings::default();
        s.bangs[0].url = "javascript:{query}".into();
        assert!(!validate(&s).is_empty());
        let mut s = Settings::default();
        s.notes.alias = "g".into();
        assert!(!validate(&s).is_empty());
        let mut s = Settings::default();
        s.bookmarks.push(Bookmark { name: "x".into(), url: "file:///c:/".into(), enabled: true });
        assert!(!validate(&s).is_empty());
    }

    #[test]
    fn save_and_load_file() {
        let dir = std::env::temp_dir().join(format!("cmdpal-test-{}", std::process::id()));
        let path = dir.join("settings.toml");
        let mut s = Settings::default();
        s.palette.hotkey = "Ctrl+Shift+P".into();
        s.commands.push(UserCommand { id: "c1".into(), name: "C".into(), program: "notepad.exe".into(), args: vec!["{input}".into()], ..Default::default() });
        save_to(&path, &s).unwrap();
        assert_eq!(load_from(&path).unwrap(), s);
        quarantine(&path).unwrap();
        assert!(!path.exists());
        let _ = fs::remove_dir_all(dir);
    }
}
