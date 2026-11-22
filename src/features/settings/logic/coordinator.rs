use super::pages::search_entries;
use crate::core::contracts::{Action, ActionHandler, Category, Feature, FeatureContext, FeatureResult, Outcome, Query, ResultItem, ResultProvider};
use crate::core::fuzzy;
use crate::features::settings::ui::window;
use std::rc::Rc;

pub struct SettingsFeature;

struct Search;

impl ResultProvider for Search {
    fn id(&self) -> &'static str {
        "settings"
    }

    fn query(&self, q: &Query) -> Vec<ResultItem> {
        let text = q.text.trim();
        if q.bang.is_some() || text.is_empty() {
            return vec![];
        }
        search_entries()
            .into_iter()
            .filter_map(|(title, kw, page)| {
                let s = fuzzy::score(text, title).or_else(|| fuzzy::score(text, kw).map(|v| v / 2))?;
                let mut it = ResultItem::new(format!("setting:{title}"), title, Category::Settings, s, Action::OpenSettings(page.to_string()));
                it.subtitle = "CmdPal setting".into();
                Some(it)
            })
            .collect()
    }
}

struct Opener {
    ctx_settings: crate::core::contracts::SharedSettings,
    registry: Rc<crate::core::contracts::Registry>,
}

impl ActionHandler for Opener {
    fn handle(&self, a: &Action) -> Option<Outcome> {
        let Action::OpenSettings(page) = a else { return None };
        Some(match window::open(page, self.ctx_settings.clone(), self.registry.clone()) {
            Ok(()) => Outcome { hide_palette: true, status: None },
            Err(e) => Outcome { hide_palette: false, status: Some(e) },
        })
    }
}

impl Feature for SettingsFeature {
    fn id(&self) -> &'static str {
        "settings"
    }

    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult {
        ctx.registry.add_provider(Rc::new(Search));
        ctx.registry.add_handler(Rc::new(Opener { ctx_settings: ctx.settings.clone(), registry: ctx.registry.clone() }));
        Ok(())
    }

    fn stop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_search_finds_pages() {
        let r = Search.query(&Query { text: "hotkey".into(), bang: None });
        assert!(r.iter().any(|i| i.action == Action::OpenSettings("general".into())));
        assert!(Search.query(&Query { text: "".into(), bang: None }).is_empty());
    }
}
