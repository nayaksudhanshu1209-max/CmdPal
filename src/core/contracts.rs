//! Stable cross-feature contracts: Feature lifecycle, result providers, action handlers, bang
//! sources, and the shared registry features plug into. No feature-specific behaviour lives here.
use super::settings::Settings;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub type FeatureResult = Result<(), String>;
pub type SharedSettings = Rc<RefCell<Settings>>;

pub trait Feature {
    fn id(&self) -> &'static str;
    /// Called once on the UI thread before the message loop starts.
    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult;
    /// Called once after the message loop ends. Must be safe if `start` never ran.
    fn stop(&mut self);
}

/// Display order of categories (also the tie-break order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Category {
    Commands,
    Applications,
    Settings,
    Bookmarks,
    Web,
    Notes,
    QuickFolders,
    Files,
}

impl Category {
    pub fn title(&self) -> &'static str {
        match self {
            Category::Commands => "Commands",
            Category::Applications => "Applications",
            Category::Settings => "Settings",
            Category::Bookmarks => "Bookmarks",
            Category::Web => "Web",
            Category::Notes => "Notes",
            Category::QuickFolders => "Quick folders",
            Category::Files => "Files and folders",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Quit,
    ReloadSettings,
    RebuildFileIndex,
    ToggleIndexPause,
    RefreshApps,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Launch an application shortcut/executable.
    LaunchPath(String),
    /// Open a file or folder with Windows.
    OpenPath(String),
    /// Hand an http(s) URL to the default browser.
    OpenUrl(String),
    RunUserCommand { id: String, input: String },
    Builtin(Builtin),
    OpenSettings(String),
    OpenNote(String),
    NewNote(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResultItem {
    /// Stable key used for exclusions.
    pub key: String,
    pub title: String,
    pub subtitle: String,
    pub category: Category,
    pub score: i32,
    pub action: Action,
    /// Path to resolve a shell icon from (apps, files).
    pub icon_path: Option<String>,
    pub hideable: bool,
    /// When set, the palette asks for confirmation with this text before running.
    pub confirm: Option<String>,
}

impl ResultItem {
    pub fn new(key: impl Into<String>, title: impl Into<String>, category: Category, score: i32, action: Action) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            subtitle: String::new(),
            category,
            score,
            action,
            icon_path: None,
            hideable: false,
            confirm: None,
        }
    }
}

/// What the palette searches. `bang` is set while a chip is active.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Query {
    /// Text after any chip.
    pub text: String,
    pub bang: Option<String>,
}

pub trait ResultProvider {
    fn id(&self) -> &'static str;
    fn query(&self, q: &Query) -> Vec<ResultItem>;
}

#[derive(Debug, Clone, Default)]
pub struct Outcome {
    pub hide_palette: bool,
    /// Shown in the palette status line (errors / feedback).
    pub status: Option<String>,
}

pub trait ActionHandler {
    /// Returns None when the action is not this handler's.
    fn handle(&self, action: &Action) -> Option<Outcome>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BangMatch {
    pub id: String,
    pub label: String,
    pub rest: String,
}

pub trait BangSource {
    /// If `text` starts with a configured alias followed by a space, returns the chip to show.
    fn match_bang(&self, text: &str) -> Option<BangMatch>;
}

/// Lets background threads ask the palette to refresh its results.
#[derive(Default)]
pub struct RefreshHub {
    f: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

impl RefreshHub {
    pub fn set(&self, f: Arc<dyn Fn() + Send + Sync>) {
        if let Ok(mut g) = self.f.lock() {
            *g = Some(f);
        }
    }
    pub fn trigger(&self) {
        let f = self.f.lock().ok().and_then(|g| g.clone());
        if let Some(f) = f {
            f();
        }
    }
}

#[derive(Default)]
pub struct Registry {
    providers: RefCell<Vec<Rc<dyn ResultProvider>>>,
    handlers: RefCell<Vec<Rc<dyn ActionHandler>>>,
    bangs: RefCell<Vec<Rc<dyn BangSource>>>,
    listeners: RefCell<Vec<Rc<dyn Fn()>>>,
    pub refresh: Arc<RefreshHub>,
}

impl Registry {
    pub fn add_provider(&self, p: Rc<dyn ResultProvider>) {
        self.providers.borrow_mut().push(p);
    }
    pub fn add_handler(&self, h: Rc<dyn ActionHandler>) {
        self.handlers.borrow_mut().push(h);
    }
    pub fn add_bang_source(&self, b: Rc<dyn BangSource>) {
        self.bangs.borrow_mut().push(b);
    }
    pub fn on_settings_changed(&self, f: Rc<dyn Fn()>) {
        self.listeners.borrow_mut().push(f);
    }

    pub fn query_all(&self, q: &Query) -> Vec<ResultItem> {
        let providers: Vec<_> = self.providers.borrow().clone();
        providers.iter().flat_map(|p| p.query(q)).collect()
    }

    pub fn match_bang(&self, text: &str) -> Option<BangMatch> {
        let bangs: Vec<_> = self.bangs.borrow().clone();
        bangs.iter().find_map(|b| b.match_bang(text))
    }

    pub fn dispatch(&self, a: &Action) -> Outcome {
        let handlers: Vec<_> = self.handlers.borrow().clone();
        for h in handlers {
            if let Some(o) = h.handle(a) {
                return o;
            }
        }
        Outcome { hide_palette: false, status: Some("Nothing handled this action".into()) }
    }

    pub fn notify_settings_changed(&self) {
        let ls: Vec<_> = self.listeners.borrow().clone();
        for l in ls {
            l();
        }
    }
}

pub struct FeatureContext {
    pub settings: SharedSettings,
    pub registry: Rc<Registry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct P(&'static str);
    impl ResultProvider for P {
        fn id(&self) -> &'static str {
            self.0
        }
        fn query(&self, _q: &Query) -> Vec<ResultItem> {
            vec![ResultItem::new(self.0, self.0, Category::Commands, 1, Action::OpenSettings("x".into()))]
        }
    }
    struct H;
    impl ActionHandler for H {
        fn handle(&self, a: &Action) -> Option<Outcome> {
            matches!(a, Action::OpenNote(_)).then(|| Outcome { hide_palette: true, status: None })
        }
    }

    #[test]
    fn registry_collects_and_dispatches() {
        let r = Registry::default();
        r.add_provider(Rc::new(P("a")));
        r.add_provider(Rc::new(P("b")));
        assert_eq!(r.query_all(&Query::default()).len(), 2);
        r.add_handler(Rc::new(H));
        assert!(r.dispatch(&Action::OpenNote("n".into())).hide_palette);
        assert!(r.dispatch(&Action::OpenPath("x".into())).status.is_some());
    }
}
