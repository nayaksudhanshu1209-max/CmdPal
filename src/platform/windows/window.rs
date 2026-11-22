//! Window class/creation helpers and monitor work-area queries.
use super::util::wide;
use ::windows::core::PCWSTR;
use ::windows::Win32::Foundation::{GetLastError, HINSTANCE, HWND, POINT, RECT};
use ::windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, HBRUSH, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use ::windows::Win32::System::LibraryLoader::GetModuleHandleW;
use ::windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetCursorPos, LoadCursorW, LoadIconW, RegisterClassExW, SystemParametersInfoW,
    HICON, HMENU, IDC_ARROW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WINDOW_EX_STYLE,
    WINDOW_STYLE, WNDCLASSEXW, WNDPROC,
};

pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_POPUP: u32 = 0x8000_0000;
pub const WS_BORDER: u32 = 0x0080_0000;
pub const WS_VSCROLL: u32 = 0x0020_0000;
pub const WS_TABSTOP: u32 = 0x0001_0000;
pub const WS_CLIPCHILDREN: u32 = 0x0200_0000;
pub const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
pub const WS_EX_TOPMOST: u32 = 0x8;
pub const WS_EX_TOOLWINDOW: u32 = 0x80;
pub const WS_EX_APPWINDOW: u32 = 0x4_0000;
pub const WS_EX_LAYERED: u32 = 0x8_0000;

pub fn hinstance() -> HINSTANCE {
    unsafe { GetModuleHandleW(PCWSTR::null()).map(|m| m.into()).unwrap_or_default() }
}

/// App icon from the exe resource (id 1), if the build embedded one.
pub fn app_icon() -> HICON {
    unsafe { LoadIconW(hinstance(), PCWSTR(1usize as *const u16)).unwrap_or_default() }
}

/// Registers a window class (already-registered is fine).
pub fn register_class(name: &str, proc: WNDPROC, bg: Option<HBRUSH>, icon: HICON) -> Result<(), String> {
    let n: &'static [u16] = Box::leak(wide(name).into_boxed_slice());
    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: proc,
            hInstance: hinstance(),
            hCursor: LoadCursorW(HINSTANCE::default(), IDC_ARROW).unwrap_or_default(),
            hbrBackground: bg.unwrap_or_default(),
            hIcon: icon,
            hIconSm: icon,
            lpszClassName: PCWSTR(n.as_ptr()),
            ..Default::default()
        };
        if RegisterClassExW(&wc) == 0 && GetLastError().0 != 1410 {
            return Err(format!("RegisterClassExW failed for {name}"));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    ex: u32,
    class: &str,
    title: &str,
    style: u32,
    (x, y, w, h): (i32, i32, i32, i32),
    parent: HWND,
    id: isize,
) -> Result<HWND, String> {
    let (c, t) = (wide(class), wide(title));
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(ex),
            PCWSTR(c.as_ptr()),
            PCWSTR(t.as_ptr()),
            WINDOW_STYLE(style),
            x,
            y,
            w,
            h,
            parent,
            HMENU(id as *mut std::ffi::c_void),
            hinstance(),
            None,
        )
    }
    .map_err(|e| format!("CreateWindowExW({class}): {e}"))
}

pub fn cursor_pos() -> POINT {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    p
}

/// Work area (excluding taskbar) of the monitor nearest `pt`.
pub fn work_area_at(pt: POINT) -> RECT {
    unsafe {
        let m = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(m, &mut mi).as_bool() {
            return mi.rcWork;
        }
        let mut wa = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut wa as *mut RECT as *mut std::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        wa
    }
}
