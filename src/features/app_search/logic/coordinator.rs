//! Loads the cached list immediately, then refreshes it on a low-priority background thread.
use super::{cache, discovery, provider::AppProvider};
use crate::core::contracts::{Action, ActionHandler, Builtin, Feature, FeatureContext, FeatureResult, Outcome};
use crate::core::paths;
use crate::platform::windows::proc;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct AppSearch;

struct Rescan {
    apps: Arc<RwLock<Vec<discovery::AppEntry>>>,
    refresh: Arc<crate::core::contracts::RefreshHub>,
}

fn spawn_scan(apps: Arc<RwLock<Vec<discovery::AppEntry>>>, refresh: Arc<crate::core::contracts::RefreshHub>) {
    let _ = std::thread::Builder::new().name("app-scan".into()).spawn(move || {
        proc::begin_background_thread();
        let found = discovery::discover();
        if let Some(d) = paths::local_dir() {
            cache::save(&d.join("apps.cache"), &found);
        }
        if let Ok(mut w) = apps.write() {
            *w = found;
        }
        refresh.trigger();
    });
}

impl ActionHandler for Rescan {
    fn handle(&self, a: &Action) -> Option<Outcome> {
        if matches!(a, Action::Builtin(Builtin::RefreshApps)) {
            spawn_scan(self.apps.clone(), self.refresh.clone());
            return Some(Outcome { hide_palette: true, status: None });
        }
        None
    }
}

impl Feature for AppSearch {
    fn id(&self) -> &'static str {
        "app_search"
    }

    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult {
        let initial = paths::local_dir().map(|d| cache::load(&d.join("apps.cache"))).unwrap_or_default();
        let apps = Arc::new(RwLock::new(initial));
        ctx.registry.add_provider(Rc::new(AppProvider { apps: apps.clone() }));
        ctx.registry.add_handler(Rc::new(Rescan { apps: apps.clone(), refresh: ctx.registry.refresh.clone() }));
        spawn_scan(apps, ctx.registry.refresh.clone());
        Ok(())
    }

    fn stop(&mut self) {}
}
