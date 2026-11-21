//! Pure show/hide state machine (CORE-03 behaviour) with no OS dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Show,
    Hide,
    Nothing,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Visibility {
    visible: bool,
}

impl Visibility {
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn on_hotkey(&mut self) -> Action {
        self.visible = !self.visible;
        if self.visible {
            Action::Show
        } else {
            Action::Hide
        }
    }

    pub fn on_escape(&mut self) -> Action {
        self.hide_if_visible()
    }

    pub fn on_focus_lost(&mut self) -> Action {
        self.hide_if_visible()
    }

    fn hide_if_visible(&mut self) -> Action {
        if self.visible {
            self.visible = false;
            Action::Hide
        } else {
            Action::Nothing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_toggles() {
        let mut v = Visibility::default();
        assert_eq!(v.on_hotkey(), Action::Show);
        assert_eq!(v.on_hotkey(), Action::Hide);
    }

    #[test]
    fn escape_and_focus_loss_hide_once() {
        let mut v = Visibility::default();
        v.on_hotkey();
        assert_eq!(v.on_focus_lost(), Action::Hide);
        assert_eq!(v.on_focus_lost(), Action::Nothing);
        v.on_hotkey();
        assert_eq!(v.on_escape(), Action::Hide);
        assert!(!v.is_visible());
    }
}
