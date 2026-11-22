//! Shell icon lookup with a small bounded cache (never blocks startup; used while painting).
use super::util::wide;
use ::windows::core::PCWSTR;
use ::windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use ::windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON};
use ::windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON};
use std::cell::RefCell;
use std::collections::HashMap;

const MAX_CACHE: usize = 96;

thread_local! {
    static CACHE: RefCell<HashMap<String, isize>> = RefCell::new(HashMap::new());
}

fn load(path: &str) -> isize {
    let p = wide(path);
    let mut info = SHFILEINFOW::default();
    let r = unsafe {
        SHGetFileInfoW(
            PCWSTR(p.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info as *mut SHFILEINFOW),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON,
        )
    };
    if r != 0 && !info.hIcon.is_invalid() { info.hIcon.0 as isize } else { 0 }
}

/// Returns the small shell icon for `path` (cached; None if unavailable).
pub fn get(path: &str) -> Option<HICON> {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some(&v) = c.get(path) {
            return (v != 0).then(|| HICON(v as *mut std::ffi::c_void));
        }
        if c.len() >= MAX_CACHE {
            for (_, v) in c.drain() {
                if v != 0 {
                    unsafe {
                        let _ = DestroyIcon(HICON(v as *mut std::ffi::c_void));
                    }
                }
            }
        }
        let v = load(path);
        c.insert(path.to_string(), v);
        (v != 0).then(|| HICON(v as *mut std::ffi::c_void))
    })
}
