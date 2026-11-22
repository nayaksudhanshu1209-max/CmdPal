//! Small helpers around strings, handles and messages.
use ::windows::core::PCWSTR;
use ::windows::Win32::Foundation::{COLORREF, HWND, LPARAM, WPARAM};
use ::windows::Win32::UI::HiDpi::GetDpiForWindow;
use ::windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, GetWindowTextW, PostMessageW, SendMessageW, SetWindowTextW,
};

pub const WM_APP: u32 = 0x8000;

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

pub fn send(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    unsafe { SendMessageW(h, msg, WPARAM(w), LPARAM(l)).0 }
}

pub fn post(h: HWND, msg: u32, w: usize, l: isize) {
    unsafe {
        let _ = PostMessageW(h, msg, WPARAM(w), LPARAM(l));
    }
}

pub fn get_text(h: HWND) -> String {
    unsafe {
        let n = GetWindowTextLengthW(h);
        if n <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; n as usize + 1];
        let got = GetWindowTextW(h, &mut buf);
        String::from_utf16_lossy(&buf[..got.max(0) as usize])
    }
}

pub fn set_text(h: HWND, s: &str) {
    let w = wide(s);
    unsafe {
        let _ = SetWindowTextW(h, PCWSTR(w.as_ptr()));
    }
}

pub fn hwnd_to_isize(h: HWND) -> isize {
    h.0 as isize
}

pub fn hwnd_from_isize(v: isize) -> HWND {
    HWND(v as *mut std::ffi::c_void)
}

pub fn dpi_for(h: HWND) -> u32 {
    let d = unsafe { GetDpiForWindow(h) };
    if d == 0 { 96 } else { d }
}

pub fn scale(v: i32, dpi: u32) -> i32 {
    v * dpi as i32 / 96
}

pub fn loword(v: usize) -> u32 {
    (v & 0xFFFF) as u32
}

pub fn hiword(v: usize) -> u32 {
    ((v >> 16) & 0xFFFF) as u32
}

use ::windows::Win32::Foundation::{BOOL, RECT};
use ::windows::Win32::Graphics::Gdi::InvalidateRect;
use ::windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use ::windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetWindowRect, KillTimer, SetForegroundWindow, SetTimer, SetWindowPos, ShowWindow,
    SET_WINDOW_POS_FLAGS, SW_HIDE, SW_SHOW,
};

pub fn move_window(h: HWND, x: i32, y: i32, w: i32, ht: i32) {
    unsafe {
        let _ = SetWindowPos(h, HWND::default(), x, y, w, ht, SET_WINDOW_POS_FLAGS(0x14)); // NOZORDER|NOACTIVATE
    }
}

pub fn invalidate(h: HWND) {
    unsafe {
        let _ = InvalidateRect(h, None, BOOL(0));
    }
}

pub fn set_timer(h: HWND, id: usize, ms: u32) {
    unsafe {
        SetTimer(h, id, ms, None);
    }
}

pub fn kill_timer(h: HWND, id: usize) {
    unsafe {
        let _ = KillTimer(h, id);
    }
}

pub fn client_rect(h: HWND) -> RECT {
    let mut r = RECT::default();
    unsafe {
        let _ = GetClientRect(h, &mut r);
    }
    r
}

pub fn window_rect(h: HWND) -> RECT {
    let mut r = RECT::default();
    unsafe {
        let _ = GetWindowRect(h, &mut r);
    }
    r
}

pub fn show(h: HWND, visible: bool) {
    unsafe {
        let _ = ShowWindow(h, if visible { SW_SHOW } else { SW_HIDE });
    }
}

pub fn foreground(h: HWND) {
    unsafe {
        let _ = SetForegroundWindow(h);
    }
}

pub fn focus(h: HWND) {
    unsafe {
        let _ = SetFocus(h);
    }
}

pub fn set_font(h: HWND, font: ::windows::Win32::Graphics::Gdi::HFONT) {
    send(h, 0x30, font.0 as usize, 1); // WM_SETFONT
}
