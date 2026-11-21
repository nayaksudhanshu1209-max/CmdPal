//! Message loop, process-level setup and error dialogs.
use ::windows::core::PCWSTR;
use ::windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MessageBoxW, TranslateMessage, MB_ICONERROR, MB_OK, MSG,
};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn show_error(title: &str, text: &str) {
    let (t, m) = (wide(title), wide(text));
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(m.as_ptr()),
            PCWSTR(t.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

/// Per-monitor DPI awareness; must run before any window is created.
pub fn enable_dpi_awareness() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

/// Blocks until WM_QUIT; returns its exit code.
pub fn run_message_loop() -> i32 {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    msg.wParam.0 as i32
}
