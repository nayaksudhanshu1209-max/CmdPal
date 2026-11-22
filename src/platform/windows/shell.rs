//! Message loop, process-level setup and dialogs.
use super::util::wide;
use ::windows::core::PCWSTR;
use ::windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use ::windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, IsChild, IsDialogMessageW, MessageBoxW, PostQuitMessage,
    TranslateMessage, IDYES, MB_ICONERROR, MB_ICONQUESTION, MB_OK, MB_YESNO, MSG,
};
use std::sync::atomic::{AtomicIsize, Ordering};

static DIALOG: AtomicIsize = AtomicIsize::new(0);

/// Window (the settings window) that should get Tab/arrow keyboard navigation; 0 = none.
pub fn set_dialog_window(h: isize) {
    DIALOG.store(h, Ordering::SeqCst);
}

pub fn show_error(title: &str, text: &str) {
    let (t, m) = (wide(title), wide(text));
    unsafe {
        MessageBoxW(HWND::default(), PCWSTR(m.as_ptr()), PCWSTR(t.as_ptr()), MB_OK | MB_ICONERROR);
    }
}

pub fn confirm(owner: HWND, title: &str, text: &str) -> bool {
    let (t, m) = (wide(title), wide(text));
    unsafe { MessageBoxW(owner, PCWSTR(m.as_ptr()), PCWSTR(t.as_ptr()), MB_YESNO | MB_ICONQUESTION) == IDYES }
}

/// Per-monitor DPI awareness; must run before any window is created.
pub fn enable_dpi_awareness() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

pub fn quit() {
    unsafe { PostQuitMessage(0) }
}

/// Blocks until WM_QUIT; returns its exit code.
pub fn run_message_loop() -> i32 {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            let d = DIALOG.load(Ordering::SeqCst);
            if d != 0 {
                let dh = HWND(d as *mut std::ffi::c_void);
                if (msg.hwnd == dh || IsChild(dh, msg.hwnd).as_bool()) && IsDialogMessageW(dh, &msg).as_bool() {
                    continue;
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    msg.wParam.0 as i32
}

pub fn current_thread_id() -> u32 {
    unsafe { ::windows::Win32::System::Threading::GetCurrentThreadId() }
}

/// Asks the UI thread's message loop to exit (used by `--smoke` runs).
pub fn post_quit_to_thread(tid: u32) {
    unsafe {
        let _ = ::windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(tid, 0x0012, WPARAM(0), LPARAM(0));
    }
}
