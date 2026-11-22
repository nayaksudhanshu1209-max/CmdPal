//! Generic action handling owned by the composition root (no feature-specific behaviour).
use crate::core::contracts::{Action, ActionHandler, Builtin, Outcome, Registry, SharedSettings};
use crate::core::settings;
use crate::platform::windows::{proc, shell};
use std::rc::Rc;

pub struct ShellHandler {
    pub settings: SharedSettings,
    pub registry: Rc<Registry>,
}

fn open(target: &str) -> Outcome {
    match proc::shell_open(target) {
        Ok(()) => Outcome { hide_palette: true, status: None },
        Err(e) => Outcome { hide_palette: false, status: Some(e) },
    }
}

impl ActionHandler for ShellHandler {
    fn handle(&self, a: &Action) -> Option<Outcome> {
        match a {
            Action::LaunchPath(p) | Action::OpenPath(p) => Some(open(p)),
            Action::Builtin(Builtin::Quit) => {
                shell::quit();
                Some(Outcome { hide_palette: true, status: None })
            }
            Action::Builtin(Builtin::ReloadSettings) => Some(match settings::settings_path().map(|p| settings::load_from(&p)) {
                Some(Ok(s)) => {
                    *self.settings.borrow_mut() = s;
                    self.registry.notify_settings_changed();
                    Outcome { hide_palette: true, status: None }
                }
                Some(Err(e)) => Outcome { hide_palette: false, status: Some(format!("settings.toml not loaded: {}", e.lines().next().unwrap_or(""))) },
                None => Outcome { hide_palette: false, status: Some("settings location unavailable".into()) },
            }),
            _ => None,
        }
    }
}
