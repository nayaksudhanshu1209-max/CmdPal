use super::{exec, provider::CommandProvider};
use crate::core::contracts::{Action, ActionHandler, Feature, FeatureContext, FeatureResult, Outcome, SharedSettings};
use std::rc::Rc;

pub struct Commands;

struct Runner {
    settings: SharedSettings,
}

impl ActionHandler for Runner {
    fn handle(&self, a: &Action) -> Option<Outcome> {
        let Action::RunUserCommand { id, input } = a else { return None };
        let cmd = self.settings.borrow().commands.iter().find(|c| &c.id == id && c.enabled).cloned();
        Some(match cmd {
            None => Outcome { hide_palette: false, status: Some(format!("Command '{id}' is missing or disabled")) },
            Some(c) => match exec::run(&c, input) {
                Ok(()) => Outcome { hide_palette: true, status: None },
                Err(e) => Outcome { hide_palette: false, status: Some(e) },
            },
        })
    }
}

impl Feature for Commands {
    fn id(&self) -> &'static str {
        "commands"
    }

    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult {
        ctx.registry.add_provider(Rc::new(CommandProvider { settings: ctx.settings.clone() }));
        ctx.registry.add_handler(Rc::new(Runner { settings: ctx.settings.clone() }));
        Ok(())
    }

    fn stop(&mut self) {}
}
