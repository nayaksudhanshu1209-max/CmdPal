//! Notes feature entry: note bang chip, note search results, open/create handler.
use super::super::persistence::store::{sanitize_name, Store};
use super::super::ui::note_window;
use crate::core::contracts::{
    Action, ActionHandler, BangMatch, BangSource, Category, Feature, FeatureContext, FeatureResult, Outcome, Query,
    ResultItem, ResultProvider, SharedSettings,
};
use crate::core::fuzzy;
use std::rc::Rc;

pub struct Notes;

/// "n my note" -> chip "Notes", rest "my note".
pub fn match_alias(alias: &str, text: &str) -> Option<String> {
    let n = alias.chars().count();
    let head: String = text.chars().take(n).collect();
    (!alias.is_empty() && head.eq_ignore_ascii_case(alias) && text.chars().nth(n) == Some(' ')).then(|| text.chars().skip(n + 1).collect())
}

struct Bang(SharedSettings);

impl BangSource for Bang {
    fn match_bang(&self, text: &str) -> Option<BangMatch> {
        let alias = self.0.borrow().notes.alias.clone();
        match_alias(&alias, text).map(|rest| BangMatch { id: "notes".into(), label: "Notes".into(), rest })
    }
}

struct Provider;

impl ResultProvider for Provider {
    fn id(&self) -> &'static str {
        "notes"
    }

    fn query(&self, q: &Query) -> Vec<ResultItem> {
        let in_bang = q.bang.as_deref() == Some("notes");
        if q.bang.is_some() && !in_bang {
            return vec![];
        }
        let text = q.text.trim();
        if !in_bang && text.is_empty() {
            return vec![];
        }
        let Some(store) = Store::default_store() else { return vec![] };
        let mut out = vec![];
        let lower = text.to_lowercase();
        for (i, n) in store.list().into_iter().take(300).enumerate() {
            let score = if text.is_empty() {
                Some(500 - i as i32)
            } else if let Some(s) = fuzzy::score(text, &n.name) {
                Some(s + 100)
            } else if in_bang && store.contains_text(&n.name, &lower) {
                Some(40)
            } else {
                None
            };
            if let Some(s) = score {
                let mut it = ResultItem::new(format!("note:{}", n.name.to_lowercase()), n.name.clone(), Category::Notes, s, Action::OpenNote(n.name.clone()));
                it.subtitle = "Note".into();
                it.hideable = false;
                out.push(it);
            }
        }
        if in_bang {
            let title = if text.is_empty() { "New note".to_string() } else { format!("Create note \"{}\"", sanitize_name(text)) };
            out.push(ResultItem::new("note:new", title, Category::Notes, 5, Action::NewNote(text.to_string())));
        }
        out
    }
}

struct Handler;

impl ActionHandler for Handler {
    fn handle(&self, a: &Action) -> Option<Outcome> {
        let name = match a {
            Action::OpenNote(n) => n.clone(),
            Action::NewNote(n) => {
                let store = Store::default_store()?;
                store.unique_name(if n.trim().is_empty() { "Untitled" } else { n })
            }
            _ => return None,
        };
        Some(match note_window::open(&name) {
            Ok(()) => Outcome { hide_palette: true, status: None },
            Err(e) => Outcome { hide_palette: false, status: Some(e) },
        })
    }
}

impl Feature for Notes {
    fn id(&self) -> &'static str {
        "notes"
    }

    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult {
        ctx.registry.add_bang_source(Rc::new(Bang(ctx.settings.clone())));
        ctx.registry.add_provider(Rc::new(Provider));
        ctx.registry.add_handler(Rc::new(Handler));
        Ok(())
    }

    fn stop(&mut self) {
        note_window::flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_matching() {
        assert_eq!(match_alias("n", "n hello world").as_deref(), Some("hello world"));
        assert_eq!(match_alias("n", "N ").as_deref(), Some(""));
        assert!(match_alias("n", "note").is_none());
        assert!(match_alias("n", "n").is_none());
    }
}
