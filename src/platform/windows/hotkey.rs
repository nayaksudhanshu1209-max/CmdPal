//! Global hotkey parsing (pure) and registration (Win32 RegisterHotKey).
use ::windows::Win32::Foundation::HWND;
use ::windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_NOREPEAT,
};

const MOD_ALT: u32 = 0x1;
const MOD_CONTROL: u32 = 0x2;
const MOD_SHIFT: u32 = 0x4;
const MOD_WIN: u32 = 0x8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeySpec {
    pub modifiers: u32,
    pub vk: u32,
}

/// Parses text such as "Ctrl+Alt+Space". At least one modifier and exactly one key are required.
pub fn parse(text: &str) -> Result<HotkeySpec, String> {
    let mut modifiers = 0u32;
    let mut key: Option<u32> = None;
    for part in text.split('+').map(str::trim) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= MOD_CONTROL,
            "alt" => modifiers |= MOD_ALT,
            "shift" => modifiers |= MOD_SHIFT,
            "win" | "super" => modifiers |= MOD_WIN,
            "" => return Err("empty segment".into()),
            other => {
                if key.is_some() {
                    return Err("more than one non-modifier key".into());
                }
                key = Some(key_code(other).ok_or_else(|| format!("unknown key '{part}'"))?);
            }
        }
    }
    let vk = key.ok_or("no key given")?;
    if modifiers == 0 {
        return Err("at least one modifier (Ctrl/Alt/Shift/Win) is required".into());
    }
    Ok(HotkeySpec { modifiers, vk })
}

fn key_code(name: &str) -> Option<u32> {
    match name {
        "space" => Some(0x20),
        "enter" | "return" => Some(0x0D),
        "tab" => Some(0x09),
        n if n.len() == 1 => {
            let c = n.as_bytes()[0];
            if c.is_ascii_lowercase() {
                Some(c.to_ascii_uppercase() as u32)
            } else if c.is_ascii_digit() {
                Some(c as u32)
            } else {
                None
            }
        }
        n if n.starts_with('f') => n[1..]
            .parse::<u32>()
            .ok()
            .filter(|x| (1..=24).contains(x))
            .map(|x| 0x70 + x - 1),
        _ => None,
    }
}

pub fn register(hwnd: HWND, id: i32, spec: &HotkeySpec) -> Result<(), String> {
    unsafe {
        RegisterHotKey(
            Some(hwnd),
            id,
            HOT_KEY_MODIFIERS(spec.modifiers) | MOD_NOREPEAT,
            spec.vk,
        )
    }
    .map_err(|e| {
        format!("Could not register the global hotkey (is it used by another program?): {e}")
    })
}

pub fn unregister(hwnd: HWND, id: i32) {
    unsafe {
        let _ = UnregisterHotKey(Some(hwnd), id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default() {
        let s = parse("Ctrl+Alt+Space").unwrap();
        assert_eq!(
            s,
            HotkeySpec {
                modifiers: MOD_CONTROL | MOD_ALT,
                vk: 0x20
            }
        );
    }

    #[test]
    fn parses_letters_digits_function_keys() {
        assert_eq!(parse("shift+ctrl+p").unwrap().vk, 'P' as u32);
        assert_eq!(parse("Alt+7").unwrap().vk, '7' as u32);
        assert_eq!(parse("Win+F12").unwrap().vk, 0x7B);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse("").is_err());
        assert!(parse("Space").is_err());
        assert!(parse("Ctrl+").is_err());
        assert!(parse("Ctrl+A+B").is_err());
        assert!(parse("Ctrl+Banana").is_err());
        assert!(parse("Ctrl+F25").is_err());
    }
}
