//! Window subclassing via SetWindowLongPtr (used for EDIT / RichEdit key handling).
use ::windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use ::windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use ::windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WNDPROC,
};

pub type Proc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

/// Installs `p` and returns the previous procedure (store it; pass to `call_prev`).
pub fn install(hwnd: HWND, p: Proc) -> isize {
    unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, p as usize as isize) }
}

pub fn call_prev(prev: isize, hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        let f: WNDPROC = std::mem::transmute::<isize, WNDPROC>(prev);
        CallWindowProcW(f, hwnd, msg, wp, lp)
    }
}

pub fn key_down(vk: i32) -> bool {
    unsafe { (GetKeyState(vk) as u16 & 0x8000) != 0 }
}
