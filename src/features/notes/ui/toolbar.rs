//! Bottom formatting bar: custom-drawn, scrolls horizontally (mouse wheel), actions run on
//! click, and buttons can be reordered by dragging. Parent receives WM_APP+10 (click, wParam =
//! index) and WM_APP+11 (order changed).
use super::super::markdown_model::assist;
use crate::platform::windows::gdi::{self, Buffer, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER};
use crate::platform::windows::util::{self, rgb, WM_APP};
use crate::platform::windows::window::{create, register_class, WS_CHILD, WS_VISIBLE};
use ::windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use ::windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, HFONT, PAINTSTRUCT};
use ::windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use ::windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_ERASEBKGND};
use std::cell::RefCell;

pub const MSG_CLICK: u32 = WM_APP + 10;
pub const MSG_REORDER: u32 = WM_APP + 11;
pub const HEIGHT: i32 = 34;

struct State {
    order: Vec<String>,
    widths: Vec<i32>,
    offset: i32,
    down: Option<(usize, i32)>,
    dragging: bool,
    font: Option<isize>,
}

thread_local! {
    static ST: RefCell<State> = RefCell::new(State { order: assist::default_order(), widths: vec![], offset: 0, down: None, dragging: false, font: None });
}

pub fn create_bar(parent: HWND, id: isize) -> Result<HWND, String> {
    register_class("CmdPalNoteBar", Some(proc), None, Default::default())?;
    create(0, "CmdPalNoteBar", "", WS_CHILD | WS_VISIBLE, (0, 0, 100, HEIGHT), parent, id)
}

pub fn set_order(h: HWND, order: Vec<String>) {
    ST.with(|s| {
        let mut s = s.borrow_mut();
        s.order = order;
        s.offset = 0;
    });
    util::invalidate(h);
}

pub fn order() -> Vec<String> {
    ST.with(|s| s.borrow().order.clone())
}

fn index_at(s: &State, x: i32) -> Option<usize> {
    let mut left = 8 - s.offset;
    for (i, w) in s.widths.iter().enumerate() {
        if x >= left && x < left + *w {
            return Some(i);
        }
        left += *w + 6;
    }
    None
}

fn clamp_offset(s: &mut State, client_w: i32) {
    let total: i32 = s.widths.iter().map(|w| w + 6).sum::<i32>() + 16;
    s.offset = s.offset.clamp(0, (total - client_w).max(0));
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let rc = util::client_rect(hwnd);
            let buf = Buffer::new(hdc, rc.right, rc.bottom);
            gdi::fill(buf.dc, &rc, rgb(38, 38, 44));
            ST.with(|s| {
                let mut s = s.borrow_mut();
                let font: HFONT = match s.font {
                    Some(f) => HFONT(f as *mut std::ffi::c_void),
                    None => {
                        let f = gdi::make_font("Segoe UI", 14, false, false);
                        s.font = Some(f.0 as isize);
                        f
                    }
                };
                let widths: Vec<i32> = s
                    .order
                    .iter()
                    .map(|id| assist::tool(id).map(|t| gdi::text_width(buf.dc, font, t.label) + 22).unwrap_or(30))
                    .collect();
                s.widths = widths;
                clamp_offset(&mut s, rc.right);
                let mut left = 8 - s.offset;
                for (i, id) in s.order.iter().enumerate() {
                    let w = s.widths[i];
                    let r = gdi::rect(left, 4, left + w, rc.bottom - 4);
                    let active = s.dragging && s.down.map(|d| d.0) == Some(i);
                    gdi::fill(buf.dc, &r, if active { rgb(70, 110, 170) } else { rgb(58, 58, 68) });
                    if let Some(t) = assist::tool(id) {
                        gdi::draw_text(buf.dc, font, t.label, &r, rgb(230, 230, 235), DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                    }
                    left += w + 6;
                }
            });
            buf.present(hdc);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lp.0 & 0xFFFF) as i16 as i32;
            ST.with(|s| {
                let mut s = s.borrow_mut();
                s.down = index_at(&s, x).map(|i| (i, x));
                s.dragging = false;
            });
            SetCapture(hwnd);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let x = (lp.0 & 0xFFFF) as i16 as i32;
            let changed = ST.with(|s| {
                let mut s = s.borrow_mut();
                if let Some((_, sx)) = s.down {
                    if !s.dragging && (x - sx).abs() > 8 {
                        s.dragging = true;
                        return true;
                    }
                }
                false
            });
            if changed {
                util::invalidate(hwnd);
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let x = (lp.0 & 0xFFFF) as i16 as i32;
            let _ = ReleaseCapture();
            let (click, reordered) = ST.with(|s| {
                let mut s = s.borrow_mut();
                let (down, dragging) = (s.down.take(), s.dragging);
                s.dragging = false;
                match down {
                    Some((i, _)) if dragging => {
                        let target = index_at(&s, x).unwrap_or(if x < 0 { 0 } else { s.order.len() - 1 });
                        let mut o = s.order.clone();
                        assist::move_item(&mut o, i, target);
                        let changed = o != s.order;
                        s.order = o;
                        (None, changed)
                    }
                    Some((i, _)) => (Some(i), false),
                    None => (None, false),
                }
            });
            util::invalidate(hwnd);
            let parent = ::windows::Win32::UI::WindowsAndMessaging::GetParent(hwnd).unwrap_or_default();
            if let Some(i) = click {
                util::post(parent, MSG_CLICK, i, 0);
            }
            if reordered {
                util::post(parent, MSG_REORDER, 0, 0);
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = (wp.0 >> 16) as u16 as i16 as i32;
            ST.with(|s| s.borrow_mut().offset -= delta / 120 * 80);
            util::invalidate(hwnd);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
