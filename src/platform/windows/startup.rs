//! Optional launch-at-login via the per-user Run registry key (no admin needed).
use super::util::wide;
use ::windows::core::PCWSTR;
use ::windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "CmdPal";

fn open(access: u32) -> Option<HKEY> {
    let k = wide(RUN_KEY);
    let mut h = HKEY::default();
    let r = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(k.as_ptr()),
            0,
            ::windows::Win32::System::Registry::REG_SAM_FLAGS(access),
            &mut h,
        )
    };
    if r.0 == 0 { Some(h) } else { None }
}

pub fn is_enabled() -> bool {
    let Some(h) = open(KEY_QUERY_VALUE.0) else { return false };
    let v = wide(VALUE);
    let r = unsafe { RegQueryValueExW(h, PCWSTR(v.as_ptr()), None, None, None, None) };
    unsafe {
        let _ = RegCloseKey(h);
    }
    r.0 == 0
}

pub fn set_enabled(enable: bool, exe: &str) -> Result<(), String> {
    let h = open(KEY_SET_VALUE.0).ok_or("could not open the Run registry key")?;
    let v = wide(VALUE);
    let res = if enable {
        let data = wide(&format!("\"{exe}\""));
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2) };
        unsafe { RegSetValueExW(h, PCWSTR(v.as_ptr()), 0, REG_SZ, Some(bytes)) }
    } else {
        unsafe { RegDeleteValueW(h, PCWSTR(v.as_ptr())) }
    };
    unsafe {
        let _ = RegCloseKey(h);
    }
    // 2 = ERROR_FILE_NOT_FOUND (already disabled) is fine when disabling.
    if res.0 == 0 || (!enable && res.0 == 2) { Ok(()) } else { Err(format!("registry error {}", res.0)) }
}
