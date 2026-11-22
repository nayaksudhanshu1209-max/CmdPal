use super::{bang, provider::WebProvider};
use crate::core::contracts::{
    Action, ActionHandler, BangMatch, BangSource, Feature, FeatureContext, FeatureResult, Outcome, SharedSettings,
};
use crate::core::url;
use crate::platform::windows::proc;
use std::rc::Rc;

pub struct WebBangs;

struct Bangs {
    settings: SharedSettings,
}

impl BangSource for Bangs {
    fn match_bang(&self, text: &str) -> Option<BangMatch> {
        let s = self.settings.borrow();
        bang::match_alias(&s.bangs, text).map(|(b, rest)| BangMatch {
            id: format!("web:{}", b.alias.to_lowercase()),
            label: b.name.clone(),
            rest,
        })
    }
}

struct OpenUrl;

impl ActionHandler for OpenUrl {
    fn handle(&self, a: &Action) -> Option<Outcome> {
        let Action::OpenUrl(u) = a else { return None };
        // Re-validate right before handing off (HTTP/HTTPS only).
        if !url::is_valid_http_url(u) {
            return Some(Outcome { hide_palette: false, status: Some("Blocked: not a valid http(s) URL".into()) });
        }
        Some(match proc::shell_open(u) {
            Ok(()) => Outcome { hide_palette: true, status: None },
            Err(e) => Outcome { hide_palette: false, status: Some(e) },
        })
    }
}

impl Feature for WebBangs {
    fn id(&self) -> &'static str {
        "web_bangs"
    }

    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult {
        ctx.registry.add_bang_source(Rc::new(Bangs { settings: ctx.settings.clone() }));
        ctx.registry.add_provider(Rc::new(WebProvider { settings: ctx.settings.clone() }));
        ctx.registry.add_handler(Rc::new(OpenUrl));
        Ok(())
    }

    fn stop(&mut self) {}
}
