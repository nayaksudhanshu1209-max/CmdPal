//! Feature coordinator: validates the hotkey, creates the window, registers the hotkey.
use super::super::ui::window;
use crate::core::contracts::{Feature, FeatureResult};
use crate::platform::windows::{hotkey, shell};
use ::windows::Win32::Foundation::HWND;

const HOTKEY_ID: i32 = 1;

pub struct PaletteCoordinator {
    hotkey_text: String,
    hwnd: Option<HWND>,
}

impl PaletteCoordinator {
    pub fn new(hotkey_text: String) -> Self {
        Self {
            hotkey_text,
            hwnd: None,
        }
    }
}

impl Feature for PaletteCoordinator {
    fn id(&self) -> &'static str {
        "palette"
    }

    fn start(&mut self) -> FeatureResult {
        let spec = hotkey::parse(&self.hotkey_text)
            .map_err(|e| format!("Invalid hotkey '{}': {e}", self.hotkey_text))?;
        shell::enable_dpi_awareness();
        let hwnd = window::create()?;
        if let Err(e) = hotkey::register(hwnd, HOTKEY_ID, &spec) {
            window::destroy(hwnd);
            return Err(e);
        }
        self.hwnd = Some(hwnd);
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(h) = self.hwnd.take() {
            hotkey::unregister(h, HOTKEY_ID);
            window::destroy(h);
        }
    }
}
