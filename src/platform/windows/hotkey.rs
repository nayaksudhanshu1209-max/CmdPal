//! Global hotkey registration (Win32 RegisterHotKey). Parsing lives in `core::hotkey`.
use crate::core::hotkey::HotkeySpec;
use ::windows::Win32::Foundation::HWND;
use ::windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_NOREPEAT,
};

pub fn register(hwnd: HWND, id: i32, spec: &HotkeySpec) -> Result<(), String> {
    unsafe { RegisterHotKey(hwnd, id, HOT_KEY_MODIFIERS(spec.modifiers) | MOD_NOREPEAT, spec.vk) }
        .map_err(|e| {
            format!("Could not register the global hotkey (is it used by another program?): {e}")
        })
}

pub fn unregister(hwnd: HWND, id: i32) {
    unsafe {
        let _ = UnregisterHotKey(hwnd, id);
    }
}
