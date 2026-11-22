use super::exec;
use crate::core::contracts::{Action, Builtin, Category, Query, ResultItem, ResultProvider, SharedSettings};
use crate::core::fuzzy;

pub struct CommandProvider {
    pub settings: SharedSettings,
}

const BUILTINS: &[(&str, &str, &str, Builtin)] = &[
    ("settings", "CmdPal settings", "Open the settings window", Builtin::ReloadSettings),
    ("reload", "Reload settings", "Re-read settings.toml", Builtin::ReloadSettings),
    ("refresh-apps", "Refresh applications", "Rescan installed applications", Builtin::RefreshApps),
    ("rebuild-index", "Rebuild file index", "Rescan indexed folders from scratch", Builtin::RebuildFileIndex),
    ("pause-index", "Pause or resume file indexing", "Toggle the background indexer", Builtin::ToggleIndexPause),
    ("quit", "Quit CmdPal", "Exit the application", Builtin::Quit),
];

impl ResultProvider for CommandProvider {
    fn id(&self) -> &'static str {
        "commands"
    }

    fn query(&self, q: &Query) -> Vec<ResultItem> {
        if q.bang.is_some() {
            return vec![];
        }
        let empty = q.text.trim().is_empty();
        let mut out = vec![];
        for (key, title, desc, b) in BUILTINS {
            let score = if empty { Some(if *key == "settings" { 300 } else { 100 }) } else { fuzzy::score(&q.text, title) };
            if let Some(s) = score {
                let action = if *key == "settings" { Action::OpenSettings("general".into()) } else { Action::Builtin(*b) };
                let mut it = ResultItem::new(format!("cmd:builtin:{key}"), *title, Category::Commands, s + 20, action);
                it.subtitle = (*desc).into();
                out.push(it);
            }
        }
        if empty {
            out.truncate(3);
        }
        let s = self.settings.borrow();
        let ql = q.text.trim().to_lowercase();
        for c in s.commands.iter().filter(|c| c.enabled) {
            let nl = c.name.to_lowercase();
            let (score, input) = if let Some(rest) = ql.strip_prefix(&format!("{nl} ")) {
                // keep the user's original casing for the input
                let orig = q.text.trim();
                (Some(950), orig[orig.len() - rest.len()..].to_string())
            } else if empty {
                (Some(60), String::new())
            } else {
                (fuzzy::score(&q.text, &c.name).map(|v| v + 30), String::new())
            };
            if let Some(sc) = score {
                let mut it = ResultItem::new(
                    format!("cmd:user:{}", c.id),
                    c.name.clone(),
                    Category::Commands,
                    sc,
                    Action::RunUserCommand { id: c.id.clone(), input: input.clone() },
                );
                it.subtitle = if c.description.is_empty() { exec::preview(c, &input) } else { c.description.clone() };
                if c.confirm {
                    it.confirm = Some(format!("Run this command?\n\n{}", exec::preview(c, &input)));
                }
                out.push(it);
            }
        }
        out
    }
}
