//! Maps GUI controls to the canonical `Settings` model (pure, tested). Scalar values use
//! `get_value`/`set_value`; lists use `get_rows`/`set_rows` (each row = list of field strings;
//! booleans are "1"/"0").
use crate::core::hotkey;
use crate::core::settings::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldKind {
    Text,
    Bool,
    Path,
}

#[derive(Debug, Clone, Copy)]
pub struct Field {
    pub label: &'static str,
    pub kind: FieldKind,
}

const fn t(label: &'static str) -> Field {
    Field { label, kind: FieldKind::Text }
}
const fn b(label: &'static str) -> Field {
    Field { label, kind: FieldKind::Bool }
}
const fn p(label: &'static str) -> Field {
    Field { label, kind: FieldKind::Path }
}

pub fn fields(list: &str) -> Vec<Field> {
    match list {
        "exclusions" => vec![t("Hidden result key")],
        "bangs" => vec![t("Alias (e.g. g)"), t("Display name"), t("URL template with {query}"), b("Enabled")],
        "bookmarks" => vec![t("Name"), t("URL (http/https)"), b("Enabled")],
        "commands" => vec![
            t("Id (no spaces)"),
            t("Name"),
            t("Description"),
            p("Program"),
            t("Arguments (separate with |, {input} = typed text)"),
            p("Working folder (optional)"),
            b("Ask before running"),
            b("Enabled"),
        ],
        "roots" => vec![p("Folder to index"), b("Enabled")],
        "patterns" => vec![t("Folder path or name pattern (*, ?)")],
        "extensions" => vec![t("File extension (without dot)")],
        "quick_folders" => vec![t("Name"), p("Folder"), b("Enabled")],
        _ => vec![],
    }
}

fn bs(v: bool) -> String {
    if v { "1" } else { "0" }.into()
}

pub fn get_value(s: &Settings, key: &str) -> String {
    match key {
        "hotkey" => s.palette.hotkey.clone(),
        "startup" => bs(s.palette.launch_at_startup),
        "near_cursor" => bs(s.palette.position_near_cursor),
        "show_suggestions" => bs(s.palette.show_suggestions_at_startup),
        "compact" => bs(s.palette.compact_mode),
        "animations" => bs(s.palette.animations),
        "default_search" => s.default_search_alias.clone(),
        "notes_alias" => s.notes.alias.clone(),
        "content_index" => bs(s.files.content_index),
        "scan_interval" => s.files.scan_interval_minutes.to_string(),
        "max_entries" => s.files.max_entries.to_string(),
        _ => String::new(),
    }
}

pub fn set_value(s: &mut Settings, key: &str, v: &str) -> Result<(), String> {
    let on = v == "1";
    let num = |what: &str| v.trim().parse::<u32>().map_err(|_| format!("{what} must be a whole number"));
    match key {
        "hotkey" => {
            hotkey::parse(v.trim()).map_err(|e| format!("Hotkey: {e}"))?;
            s.palette.hotkey = v.trim().to_string();
        }
        "startup" => s.palette.launch_at_startup = on,
        "near_cursor" => s.palette.position_near_cursor = on,
        "show_suggestions" => s.palette.show_suggestions_at_startup = on,
        "compact" => s.palette.compact_mode = on,
        "animations" => s.palette.animations = on,
        "default_search" => s.default_search_alias = v.trim().to_string(),
        "notes_alias" => s.notes.alias = v.trim().to_string(),
        "content_index" => s.files.content_index = on,
        "scan_interval" => s.files.scan_interval_minutes = num("Scan interval")?,
        "max_entries" => s.files.max_entries = num("Entry limit")?,
        _ => return Err(format!("unknown setting {key}")),
    }
    Ok(())
}

pub fn get_rows(s: &Settings, list: &str) -> Vec<Vec<String>> {
    match list {
        "exclusions" => s.exclusions.iter().map(|k| vec![k.clone()]).collect(),
        "bangs" => s.bangs.iter().map(|x| vec![x.alias.clone(), x.name.clone(), x.url.clone(), bs(x.enabled)]).collect(),
        "bookmarks" => s.bookmarks.iter().map(|x| vec![x.name.clone(), x.url.clone(), bs(x.enabled)]).collect(),
        "commands" => s
            .commands
            .iter()
            .map(|c| vec![c.id.clone(), c.name.clone(), c.description.clone(), c.program.clone(), c.args.join(" | "), c.working_dir.clone(), bs(c.confirm), bs(c.enabled)])
            .collect(),
        "roots" => s.files.roots.iter().map(|r| vec![r.path.clone(), bs(r.enabled)]).collect(),
        "patterns" => s.files.exclude_patterns.iter().map(|x| vec![x.clone()]).collect(),
        "extensions" => s.files.exclude_extensions.iter().map(|x| vec![x.clone()]).collect(),
        "quick_folders" => s.quick_folders.iter().map(|q| vec![q.name.clone(), q.path.clone(), bs(q.enabled)]).collect(),
        _ => vec![],
    }
}

pub fn set_rows(s: &mut Settings, list: &str, rows: &[Vec<String>]) -> Result<(), String> {
    let need = fields(list).len();
    if need == 0 {
        return Err(format!("unknown list {list}"));
    }
    if rows.iter().any(|r| r.len() != need) {
        return Err("a row has the wrong number of fields".into());
    }
    let on = |x: &String| x == "1";
    let tr = |x: &String| x.trim().to_string();
    match list {
        "exclusions" => s.exclusions = rows.iter().map(|r| tr(&r[0])).filter(|x| !x.is_empty()).collect(),
        "bangs" => s.bangs = rows.iter().map(|r| BangAlias { alias: tr(&r[0]), name: tr(&r[1]), url: tr(&r[2]), enabled: on(&r[3]) }).collect(),
        "bookmarks" => s.bookmarks = rows.iter().map(|r| Bookmark { name: tr(&r[0]), url: tr(&r[1]), enabled: on(&r[2]) }).collect(),
        "commands" => {
            s.commands = rows
                .iter()
                .map(|r| UserCommand {
                    id: tr(&r[0]),
                    name: tr(&r[1]),
                    description: tr(&r[2]),
                    program: tr(&r[3]),
                    args: r[4].split('|').map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect(),
                    working_dir: tr(&r[5]),
                    confirm: on(&r[6]),
                    enabled: on(&r[7]),
                })
                .collect()
        }
        "roots" => s.files.roots = rows.iter().map(|r| FileRoot { path: tr(&r[0]), enabled: on(&r[1]) }).collect(),
        "patterns" => s.files.exclude_patterns = rows.iter().map(|r| tr(&r[0])).filter(|x| !x.is_empty()).collect(),
        "extensions" => s.files.exclude_extensions = rows.iter().map(|r| tr(&r[0])).filter(|x| !x.is_empty()).collect(),
        "quick_folders" => s.quick_folders = rows.iter().map(|r| QuickFolder { name: tr(&r[0]), path: tr(&r[1]), enabled: on(&r[2]) }).collect(),
        _ => return Err(format!("unknown list {list}")),
    }
    Ok(())
}

/// One-line text shown in the list box for a row.
pub fn summary(list: &str, row: &[String]) -> String {
    let fs = fields(list);
    let mut parts: Vec<String> = vec![];
    let mut disabled = false;
    for (f, v) in fs.iter().zip(row) {
        match f.kind {
            FieldKind::Bool => {
                if f.label == "Enabled" && v == "0" {
                    disabled = true;
                }
            }
            _ if parts.len() < 3 && !v.trim().is_empty() => parts.push(v.clone()),
            _ => {}
        }
    }
    let mut s = parts.join("  |  ");
    if disabled {
        s.push_str("   (disabled)");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTS: &[&str] = &["exclusions", "bangs", "bookmarks", "commands", "roots", "patterns", "extensions", "quick_folders"];

    #[test]
    fn rows_roundtrip_for_every_list() {
        let mut s = Settings::default();
        s.exclusions = vec!["app:x".into()];
        s.bookmarks = vec![Bookmark { name: "A".into(), url: "https://a.com".into(), enabled: false }];
        s.commands = vec![UserCommand { id: "c".into(), name: "C".into(), program: "p.exe".into(), args: vec!["-a".into(), "{input}".into()], ..Default::default() }];
        s.files.roots = vec![FileRoot { path: "C:\\x".into(), enabled: true }];
        s.quick_folders = vec![QuickFolder { name: "Q".into(), path: "C:\\q".into(), enabled: true }];
        for l in LISTS {
            let rows = get_rows(&s, l);
            let mut s2 = Settings::default();
            set_rows(&mut s2, l, &rows).unwrap();
            assert_eq!(get_rows(&s2, l), rows, "{l}");
            for r in &rows {
                assert_eq!(r.len(), fields(l).len());
            }
        }
    }

    #[test]
    fn scalar_values() {
        let mut s = Settings::default();
        set_value(&mut s, "compact", "1").unwrap();
        assert_eq!(get_value(&s, "compact"), "1");
        assert!(set_value(&mut s, "hotkey", "Space").is_err());
        set_value(&mut s, "hotkey", " Ctrl+Shift+K ").unwrap();
        assert_eq!(s.palette.hotkey, "Ctrl+Shift+K");
        assert!(set_value(&mut s, "scan_interval", "abc").is_err());
        set_value(&mut s, "scan_interval", "15").unwrap();
        assert_eq!(s.files.scan_interval_minutes, 15);
        assert!(set_value(&mut s, "nope", "1").is_err());
    }

    #[test]
    fn bad_rows_rejected_and_summary() {
        let mut s = Settings::default();
        assert!(set_rows(&mut s, "bangs", &[vec!["x".into()]]).is_err());
        assert!(set_rows(&mut s, "nope", &[]).is_err());
        let sm = summary("bookmarks", &["A".into(), "https://a.com".into(), "0".into()]);
        assert!(sm.contains("A") && sm.contains("(disabled)"));
    }
}
