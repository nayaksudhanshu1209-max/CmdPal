//! The standalone Notes window: frameless, not topmost, does not close on focus loss; Esc and
//! Ctrl+W save and close. Created lazily on first use so the palette starts fast.
use super::super::markdown_model::{assist, model};
use super::super::persistence::store::Store;
use super::{richedit, toolbar};
use crate::platform::windows::util::{self, rgb, WM_APP};
use crate::platform::windows::window::{self, WS_CLIPCHILDREN, WS_EX_APPWINDOW, WS_POPUP};
use crate::platform::windows::{shell, subclass};
use ::windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use ::windows::Win32::Graphics::Gdi::CreateSolidBrush;
use ::windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, GetParent, WM_CLOSE, WM_SIZE};
use std::cell::RefCell;

const ID_EDIT: isize = 101;
const ID_BAR: isize = 102;
const TIMER_FORMAT: usize = 2;
const TIMER_SAVE: usize = 3;
const PAD_TOP: i32 = 12;
const PAD_SIDE: i32 = 12;
const BORDER: i32 = 6;
const MAX_FORMAT_UNITS: usize = 200_000;

struct Note {
    hwnd: HWND,
    edit: HWND,
    bar: HWND,
    name: String,
    dirty: bool,
    loading: bool,
    backup_done: bool,
    prev_edit_proc: isize,
    sized_once: bool,
}

thread_local! {
    static NOTE: RefCell<Option<Note>> = RefCell::new(None);
}

fn with<R>(f: impl FnOnce(&mut Note) -> R) -> Option<R> {
    NOTE.with(|n| n.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(f)))
}

fn ensure_window() -> Result<(), String> {
    if NOTE.with(|n| n.borrow().is_some()) {
        return Ok(());
    }
    if !richedit::load_library() {
        return Err("Could not load the Windows RichEdit component (Msftedit.dll)".into());
    }
    let bg = unsafe { CreateSolidBrush(rgb(richedit::BG.0, richedit::BG.1, richedit::BG.2)) };
    window::register_class("CmdPalNote", Some(proc), Some(bg), window::app_icon())?;
    let dpi = 96;
    let wa = window::work_area_at(window::cursor_pos());
    let (w, h) = (util::scale(680, dpi), util::scale(380, dpi));
    let x = wa.left + ((wa.right - wa.left) - w) / 2;
    let y = wa.top + ((wa.bottom - wa.top) - h) / 3;
    // Popup + thick frame (for resize hit-testing) with the frame removed in WM_NCCALCSIZE.
    let hwnd = window::create(WS_EX_APPWINDOW, "CmdPalNote", "CmdPal note", WS_POPUP | 0x0004_0000 | WS_CLIPCHILDREN, (x, y, w, h), HWND::default(), 0)?;
    let edit = richedit::create_edit(hwnd, ID_EDIT)?;
    let bar = toolbar::create_bar(hwnd, ID_BAR)?;
    let prev = subclass::install(edit, edit_proc);
    NOTE.with(|n| {
        *n.borrow_mut() = Some(Note { hwnd, edit, bar, name: String::new(), dirty: false, loading: false, backup_done: false, prev_edit_proc: prev, sized_once: false });
    });
    layout(hwnd);
    Ok(())
}

fn layout(hwnd: HWND) {
    let rc = util::client_rect(hwnd);
    let dpi = util::dpi_for(hwnd);
    let (top, side, bar_h) = (util::scale(PAD_TOP, dpi), util::scale(PAD_SIDE, dpi), util::scale(toolbar::HEIGHT, dpi));
    with(|n| {
        util::move_window(n.edit, side, top, (rc.right - 2 * side).max(50), (rc.bottom - top - bar_h - 6).max(50));
        util::move_window(n.bar, 0, rc.bottom - bar_h, rc.right, bar_h);
    });
}

/// Opens (creating if needed) the named note, saving any previously open note first.
pub fn open(name: &str) -> Result<(), String> {
    ensure_window()?;
    let store = Store::default_store().ok_or("notes folder is unavailable")?;
    if with(|n| n.dirty).unwrap_or(false) {
        save_now()?;
    }
    store.create_empty(name)?;
    let text = store.read(name)?;
    let order = assist::reconcile_order(&store.load_toolbar(name));
    let (hwnd, edit, bar) = with(|n| {
        n.loading = true;
        n.name = name.to_string();
        n.backup_done = false;
        n.dirty = false;
        (n.hwnd, n.edit, n.bar)
    })
    .ok_or("note window busy")?;
    richedit::set_text(edit, &text);
    toolbar::set_order(bar, order);
    util::set_text(hwnd, &format!("{name} - CmdPal"));
    reformat();
    fit_height(hwnd, edit, true);
    with(|n| n.loading = false);
    util::show(hwnd, true);
    util::foreground(hwnd);
    util::focus(edit);
    Ok(())
}

fn fit_height(hwnd: HWND, edit: HWND, first: bool) {
    let dpi = util::dpi_for(hwnd);
    let lines = richedit::line_count(edit) as i32;
    let wa = window::work_area_at(POINT { x: util::window_rect(hwnd).left, y: util::window_rect(hwnd).top });
    let chrome = util::scale(PAD_TOP + toolbar::HEIGHT + 22, dpi);
    let wanted = (lines * util::scale(26, dpi) + chrome).clamp(util::scale(260, dpi), ((wa.bottom - wa.top) * 8) / 10);
    let r = util::window_rect(hwnd);
    let cur = r.bottom - r.top;
    let grow_only = !first || with(|n| n.sized_once).unwrap_or(false);
    if wanted > cur || (first && !grow_only) {
        let top = r.top.min(wa.bottom - wanted).max(wa.top);
        util::move_window(hwnd, r.left, top, r.right - r.left, wanted);
    }
    with(|n| n.sized_once = true);
}

fn reformat() {
    let Some((edit, _)) = with(|n| (n.edit, ())) else { return };
    let text = richedit::get_text_u16(edit);
    if text.len() > MAX_FORMAT_UNITS {
        return; // very large notes stay as plain text to keep typing responsive
    }
    let sel = richedit::get_sel(edit);
    let spans = model::analyze(&text, sel);
    richedit::apply_spans(edit, &spans);
}

fn save_now() -> Result<(), String> {
    let Some((edit, name, backup)) = with(|n| (n.edit, n.name.clone(), !n.backup_done)) else { return Ok(()) };
    if name.is_empty() {
        return Ok(());
    }
    let store = Store::default_store().ok_or("notes folder is unavailable")?;
    store.save(&name, &richedit::get_text(edit), backup)?;
    with(|n| {
        n.dirty = false;
        n.backup_done = true;
    });
    Ok(())
}

/// Saves a pending edit (used on shutdown). Errors are reported, never swallowed silently.
pub fn flush() {
    if with(|n| n.dirty).unwrap_or(false) {
        if let Err(e) = save_now() {
            shell::show_error("CmdPal - could not save note", &e);
        }
    }
}

/// Closes (hides) the note after a safe save. Stays open if saving fails so no text is lost.
fn close_note() {
    util::kill_timer(with(|n| n.hwnd).unwrap_or_default(), TIMER_SAVE);
    if let Err(e) = save_now() {
        shell::show_error("CmdPal - could not save note", &format!("{e}\n\nThe note stays open so you do not lose your text."));
        return;
    }
    if let Some(h) = with(|n| n.hwnd) {
        util::show(h, false);
    }
}

pub fn current_name() -> Option<String> {
    with(|n| n.name.clone()).filter(|n| !n.is_empty())
}

/// Called when a note is deleted elsewhere: drop it without saving.
pub fn forget(name: &str) {
    let hit = with(|n| n.name == name).unwrap_or(false);
    if hit {
        with(|n| {
            n.dirty = false;
            n.name.clear();
        });
        if let Some(h) = with(|n| n.hwnd) {
            util::show(h, false);
        }
    }
}

pub fn renamed(old: &str, new: &str) {
    with(|n| {
        if n.name == old {
            n.name = new.to_string();
            util::set_text(n.hwnd, &format!("{new} - CmdPal"));
        }
    });
}

fn run_tool(index: usize) {
    let Some((edit, id)) = with(|n| (n.edit, toolbar::order().get(index).cloned())) else { return };
    let Some(tool) = id.as_deref().and_then(assist::tool) else { return };
    let (a, b) = richedit::get_sel(edit);
    let u16len = |s: &str| s.encode_utf16().count();
    match tool.kind {
        assist::Kind::Wrap(pre, post) => {
            if a == b {
                richedit::replace_sel(edit, &format!("{pre}{post}"));
                richedit::set_sel(edit, a + u16len(pre), a + u16len(pre));
            } else {
                let all = richedit::get_text_u16(edit);
                let selected = String::from_utf16_lossy(&all[a.min(all.len())..b.min(all.len())]);
                richedit::replace_sel(edit, &format!("{pre}{selected}{post}"));
                richedit::set_sel(edit, a + u16len(pre), b + u16len(pre));
            }
        }
        assist::Kind::LinePrefix(p) => {
            let start = richedit::line_start(edit, richedit::line_of_char(edit, a));
            richedit::set_sel(edit, start, start);
            richedit::replace_sel(edit, p);
            richedit::set_sel(edit, a + u16len(p), b + u16len(p));
        }
        assist::Kind::Insert(s) => richedit::replace_sel(edit, s),
    }
    util::focus(edit); // return focus to the editor
}

fn on_changed() {
    let Some((hwnd, loading)) = with(|n| (n.hwnd, n.loading)) else { return };
    if loading {
        return;
    }
    with(|n| n.dirty = true);
    util::set_timer(hwnd, TIMER_FORMAT, 90);
    util::set_timer(hwnd, TIMER_SAVE, 1500);
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        0x0083 => {
            // WM_NCCALCSIZE: no non-client area (frameless)
            if wp.0 != 0 {
                return LRESULT(0);
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        0x0084 => {
            // WM_NCHITTEST: resize on the edges, drag by the thin top strip, rest is client
            let (x, y) = ((lp.0 & 0xFFFF) as i16 as i32, ((lp.0 >> 16) & 0xFFFF) as i16 as i32);
            let r = util::window_rect(hwnd);
            let dpi = util::dpi_for(hwnd);
            let (b, strip) = (util::scale(BORDER, dpi), util::scale(PAD_TOP, dpi));
            let (l, rt, t, bt) = (x < r.left + b, x >= r.right - b, y < r.top + b, y >= r.bottom - b);
            let code = match (l, rt, t, bt) {
                (true, _, true, _) => 13,
                (_, true, true, _) => 14,
                (true, _, _, true) => 16,
                (_, true, _, true) => 17,
                (true, ..) => 10,
                (_, true, ..) => 11,
                (_, _, true, _) => 12,
                (_, _, _, true) => 15,
                _ if y < r.top + strip => 2,
                _ => 1,
            };
            LRESULT(code)
        }
        0x0024 => {
            // WM_GETMINMAXINFO
            let mi = lp.0 as *mut ::windows::Win32::UI::WindowsAndMessaging::MINMAXINFO;
            if !mi.is_null() {
                let dpi = util::dpi_for(hwnd);
                (*mi).ptMinTrackSize = POINT { x: util::scale(320, dpi), y: util::scale(200, dpi) };
            }
            LRESULT(0)
        }
        WM_SIZE => {
            layout(hwnd);
            LRESULT(0)
        }
        0x0111 => {
            // WM_COMMAND from the editor: EN_CHANGE
            if util::hiword(wp.0) == richedit::EN_CHANGE {
                on_changed();
                if let Some((h, e)) = with(|n| (n.hwnd, n.edit)) {
                    fit_height(h, e, false);
                }
            }
            LRESULT(0)
        }
        0x004E => {
            // WM_NOTIFY: EN_SELCHANGE (caret moved => delimiter visibility may change)
            let code = *((lp.0 as *const u8).add(16) as *const u32);
            if code == richedit::EN_SELCHANGE {
                if with(|n| !n.loading).unwrap_or(false) {
                    util::set_timer(hwnd, TIMER_FORMAT, 90);
                }
            }
            LRESULT(0)
        }
        0x0113 => {
            // WM_TIMER
            util::kill_timer(hwnd, wp.0);
            if wp.0 == TIMER_FORMAT {
                reformat();
            } else if wp.0 == TIMER_SAVE {
                if let Err(e) = save_now() {
                    util::set_text(hwnd, &format!("UNSAVED - {e}"));
                }
            }
            LRESULT(0)
        }
        toolbar::MSG_CLICK => {
            run_tool(wp.0);
            LRESULT(0)
        }
        toolbar::MSG_REORDER => {
            if let Some(name) = current_name() {
                if let Some(s) = Store::default_store() {
                    s.save_toolbar(&name, &toolbar::order());
                }
            }
            LRESULT(0)
        }
        0x0007 => {
            // WM_SETFOCUS: always hand focus to the editor
            if let Some(e) = with(|n| n.edit) {
                util::focus(e);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            close_note();
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe extern "system" fn edit_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let prev = NOTE.with(|n| n.try_borrow().ok().and_then(|b| b.as_ref().map(|x| x.prev_edit_proc))).unwrap_or(0);
    let ctrl = subclass::key_down(0x11);
    match msg {
        0x0100 => {
            // WM_KEYDOWN: Esc / Ctrl+W close after a safe save
            if wp.0 == 0x1B || (ctrl && wp.0 == 0x57) {
                util::post(GetParent(hwnd).unwrap_or_default(), WM_CLOSE, 0, 0);
                return LRESULT(0);
            }
        }
        0x0102 => {
            // WM_CHAR
            if wp.0 == 0x1B || wp.0 == 0x17 {
                return LRESULT(0); // swallow Esc / Ctrl+W characters
            }
            if wp.0 == 0x0D && !ctrl {
                let (a, b) = richedit::get_sel(hwnd);
                if a == b {
                    let all = richedit::get_text_u16(hwnd);
                    let ls = richedit::line_start(hwnd, richedit::line_of_char(hwnd, a));
                    if ls <= a && a <= all.len() {
                        let line = String::from_utf16_lossy(&all[ls..a]);
                        match assist::continuation(&line) {
                            assist::Continue::Insert(prefix) => {
                                richedit::replace_sel(hwnd, &format!("\r{prefix}"));
                                return LRESULT(0);
                            }
                            assist::Continue::ClearPrefix(n) => {
                                richedit::set_sel(hwnd, a.saturating_sub(n), a);
                                richedit::replace_sel(hwnd, "");
                                return LRESULT(0);
                            }
                            assist::Continue::None => {}
                        }
                    }
                }
            }
        }
        0x0302 => {
            // WM_PASTE: plain text only
            richedit::paste_text(hwnd);
            return LRESULT(0);
        }
        _ => {}
    }
    subclass::call_prev(prev, hwnd, msg, wp, lp)
}

#[allow(dead_code)]
const _UNUSED: u32 = WM_APP;
