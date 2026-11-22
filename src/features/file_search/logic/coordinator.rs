use super::super::indexing::{content::ContentIndex, control, scanner};
use super::provider::FileProvider;
use crate::core::contracts::{Action, ActionHandler, Builtin, Feature, FeatureContext, FeatureResult, Outcome};
use crate::core::paths;
use crate::core::settings::FileSettings;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Default)]
pub struct FileSearch {
    handle: Option<std::thread::JoinHandle<()>>,
}

struct Handler;

impl ActionHandler for Handler {
    fn handle(&self, a: &Action) -> Option<Outcome> {
        match a {
            Action::Builtin(Builtin::RebuildFileIndex) => {
                control::rebuild();
                Some(Outcome { hide_palette: true, status: None })
            }
            Action::Builtin(Builtin::ToggleIndexPause) => {
                control::toggle_pause();
                Some(Outcome { hide_palette: true, status: None })
            }
            _ => None,
        }
    }
}

impl Feature for FileSearch {
    fn id(&self) -> &'static str {
        "file_search"
    }

    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult {
        let shared = control::init();
        let content = Arc::new(RwLock::new(ContentIndex::default()));
        let cfg: Arc<Mutex<FileSettings>> = Arc::new(Mutex::new(ctx.settings.borrow().files.clone()));
        ctx.registry.add_provider(Rc::new(FileProvider { shared: shared.clone(), content: content.clone(), settings: ctx.settings.clone() }));
        ctx.registry.add_handler(Rc::new(Handler));

        // Settings edits: refresh the scanner's snapshot and rescan (rules/roots may have changed).
        let (settings, cfg2) = (ctx.settings.clone(), cfg.clone());
        let last = std::cell::RefCell::new(settings.borrow().files.clone());
        ctx.registry.on_settings_changed(Rc::new(move || {
            let now = settings.borrow().files.clone();
            if *last.borrow() != now {
                if let Ok(mut g) = cfg2.lock() {
                    *g = now.clone();
                }
                *last.borrow_mut() = now;
                control::rebuild();
            }
        }));

        let cache = paths::local_dir().map(|d| d.join("file-index.tsv"));
        let refresh = ctx.registry.refresh.clone();
        self.handle = std::thread::Builder::new()
            .name("file-index".into())
            .spawn(move || scanner::run(shared, cfg, content, cache, refresh))
            .ok();
        Ok(())
    }

    fn stop(&mut self) {
        control::stop();
        // Do not block shutdown on a scan in progress; the thread exits at its next check.
        self.handle.take();
    }
}
