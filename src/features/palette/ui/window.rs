//! Foundation palette window: borderless, always-on-top popup. Renders placeholder text only;
//! the real input box and result list arrive in later milestones.
use super::super::logic::placement::{centered_upper, Rect};
use super::super::logic::visibility::{Action, Visibility};
use ::windows::core::{w, PCWSTR};
use ::windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use ::windows::Win32::Graphics::Gdi::{
    BeginPaint, DrawTextW, EndPaint, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HBRUSH, PAINTSTRUCT,
};
use ::windows::Win32::System::LibraryLoader::GetModuleHandleW;
use ::windows::Win32::UI::WindowsAndMessaging::*;
use std::cell::RefCell;

const WIDTH: i32 = 640;
const HEIGHT: i32 = 72;

thread_local! {
    static STATE: RefCell<Visibility> = RefCell::new(Visibility::default());
}

fn apply(hwnd: HWND, action: Action) {
    unsafe {
        match action {
            Action::Show => {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
            Action::Hide => {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
            Action::Nothing => {}
        }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let a = STATE.with(|s| s.borrow_mut().on_hotkey());
            apply(hwnd, a);
            LRESULT(0)
        }
        WM_ACTIVATE => {
            if (wp.0 & 0xFFFF) as u32 == WA_INACTIVE {
                let a = STATE.with(|s| s.borrow_mut().on_focus_lost());
                apply(hwnd, a);
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            if wp.0 as u32 == 0x1B {
                let a = STATE.with(|s| s.borrow_mut().on_escape());
                apply(hwnd, a);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            let mut text: Vec<u16> = "CmdPal foundation shell - Esc hides, Alt+F4 quits"
                .encode_utf16()
                .collect();
            DrawTextW(
                hdc,
                &mut text,
                &mut rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

/// Creates the (initially hidden) palette window on the primary monitor work area.
pub fn create() -> Result<HWND, String> {
    unsafe {
        let hinst: HINSTANCE = GetModuleHandleW(None).map_err(|e| e.to_string())?.into();
        let class_name = w!("CmdPalPalette");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinst,
            hCursor: LoadCursorW(None, IDC_ARROW).map_err(|e| e.to_string())?,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as usize as *mut _),
            lpszClassName: class_name,
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            return Err("RegisterClassW failed".into());
        }

        let mut wa = RECT::default();
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut wa as *mut RECT as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .map_err(|e| e.to_string())?;
        let r = centered_upper(
            Rect {
                left: wa.left,
                top: wa.top,
                right: wa.right,
                bottom: wa.bottom,
            },
            WIDTH,
            HEIGHT,
        );

        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            PCWSTR::null(),
            WS_POPUP | WS_BORDER,
            r.left,
            r.top,
            r.right - r.left,
            r.bottom - r.top,
            None,
            None,
            Some(hinst),
            None,
        )
        .map_err(|e| e.to_string())
    }
}

pub fn destroy(hwnd: HWND) {
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
