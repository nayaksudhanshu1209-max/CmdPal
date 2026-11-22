//! Feature coordinator: hotkey registration, window creation, settings-change reactions.
use super::super::ui::window;
use crate::core::contracts::{Feature, FeatureContext, FeatureResult};
use crate::core::hotkey::{self, HotkeySpec};
use crate::platform::windows::{hotkey as os_hotkey, proc, startup, util};
use ::windows::Win32::Foundation::HWND;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

const HOTKEY_ID: i32 = 1;

#[derive(Default)]
pub struct PaletteCoordinator {
    hwnd: Option<HWND>,
}

impl Feature for PaletteCoordinator {
    fn id(&self) -> &'static str {
        "palette"
    }

    fn start(&mut self, ctx: &FeatureContext) -> FeatureResult {
        let text = ctx.settings.borrow().palette.hotkey.clone();
        let spec = hotkey::parse(&text).map_err(|e| format!("Invalid hotkey '{text}': {e}\nEdit settings.toml or delete it to reset."))?;
        let hwnd = window::create(ctx.settings.clone(), ctx.registry.clone())?;
        if let Err(e) = os_hotkey::register(hwnd, HOTKEY_ID, &spec) {
            window::destroy(hwnd);
            return Err(e);
        }
        self.hwnd = Some(hwnd);

        // async refresh requests from background threads
        let hi = util::hwnd_to_isize(hwnd);
        ctx.registry.refresh.set(Arc::new(move || util::post(util::hwnd_from_isize(hi), window::MSG_REFRESH, 0, 0)));

        // react to settings edits: hotkey, startup registration, layout (compact/animations)
        let (settings, current) = (ctx.settings.clone(), Rc::new(Cell::new(spec)));
        let last_text = Rc::new(RefCell::new(text));
        sync_startup(settings.borrow().palette.launch_at_startup);
        ctx.registry.on_settings_changed(Rc::new(move || {
            let (new_text, startup_on) = {
                let s = settings.borrow();
                (s.palette.hotkey.clone(), s.palette.launch_at_startup)
            };
            let h = util::hwnd_from_isize(hi);
            if *last_text.borrow() != new_text {
                if let Ok(spec) = hotkey::parse(&new_text) {
                    os_hotkey::unregister(h, HOTKEY_ID);
                    if os_hotkey::register(h, HOTKEY_ID, &spec).is_ok() {
                        current.set(spec);
                        *last_text.borrow_mut() = new_text;
                    } else {
                        let old: HotkeySpec = current.get();
                        let _ = os_hotkey::register(h, HOTKEY_ID, &old);
                        window::set_status(h, "That hotkey is already in use; kept the previous one.");
                    }
                }
            }
            sync_startup(startup_on);
            window::settings_changed(h);
        }));
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(h) = self.hwnd.take() {
            os_hotkey::unregister(h, HOTKEY_ID);
            window::destroy(h);
        }
    }
}

fn sync_startup(want: bool) {
    if startup::is_enabled() != want {
        let _ = startup::set_enabled(want, &proc::current_exe_string());
    }
}
