//! Opening things with Windows, folder dialogs, process helpers.
use super::util::wide;
use ::windows::core::PCWSTR;
use ::windows::Win32::Foundation::HWND;
use ::windows::Win32::System::Com::CoTaskMemFree;
use ::windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use ::windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, SetThreadPriority, THREAD_MODE_BACKGROUND_BEGIN,
};
use ::windows::Win32::UI::Shell::{
    SHBrowseForFolderW, SHGetPathFromIDListW, ShellExecuteW, BROWSEINFOW,
};
use ::windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Opens a file, folder, shortcut or http(s) URL with its Windows default handler.
pub fn shell_open(target: &str) -> Result<(), String> {
    let (t, verb) = (wide(target), wide("open"));
    unsafe {
        let r = ShellExecuteW(
            HWND::default(),
            PCWSTR(verb.as_ptr()),
            PCWSTR(t.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
        let code = r.0 as isize;
        if code > 32 {
            Ok(())
        } else {
            Err(format!("Windows could not open '{target}' (code {code})"))
        }
    }
}

/// Native folder picker. Returns None if cancelled.
pub fn browse_folder(owner: HWND, title: &str) -> Option<String> {
    unsafe {
        let t = wide(title);
        let mut bi = BROWSEINFOW::default();
        bi.hwndOwner = owner;
        bi.lpszTitle = PCWSTR(t.as_ptr());
        bi.ulFlags = 0x1; // BIF_RETURNONLYFSDIRS
        let pidl = SHBrowseForFolderW(&bi);
        if pidl.is_null() {
            return None;
        }
        let mut buf = [0u16; 260];
        let ok = SHGetPathFromIDListW(pidl, &mut buf).as_bool();
        CoTaskMemFree(Some(pidl as *const std::ffi::c_void));
        if !ok {
            return None;
        }
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..n]))
    }
}

/// Lowers CPU and I/O priority of the calling thread (for the background indexer).
pub fn begin_background_thread() {
    unsafe {
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_BEGIN);
    }
}

pub fn peak_working_set_kb() -> u64 {
    unsafe {
        let mut c = PROCESS_MEMORY_COUNTERS::default();
        c.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        if GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb).is_ok() {
            (c.PeakWorkingSetSize / 1024) as u64
        } else {
            0
        }
    }
}

pub fn current_exe_string() -> String {
    std::env::current_exe().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()
}
