//! Palette window: borderless always-on-top popup with a native EDIT control for the query
//! (so caret, IME, selection and long input are handled by Windows), a chip for the active bang,
//! and a custom-drawn categorised result list. All state is UI-only; behaviour lives in `logic/`
//! and in the other features via the shared registry.
use super::super::logic::placement::{self, Rect};
use super::super::logic::selection;
use super::super::logic::visibility::{Action as VAct, Visibility};
use crate::core::contracts::{Category, Query, Registry, ResultItem, SharedSettings};
use crate::core::settings;
use crate::features::suggestions::api::{build_rows, Row};
use crate::platform::windows::gdi::{self, Buffer, DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_SINGLELINE, DT_VCENTER};
use crate::platform::windows::util::{self, loword, rgb, send, WM_APP};
use crate::platform::windows::window::{self as win, WS_CHILD, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE};
use crate::platform::windows::{icons, shell, subclass};
use ::windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use ::windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use ::windows::Win32::Graphics::Gdi::{BeginPaint, CreateSolidBrush, EndPaint, SetBkColor, SetTextColor, HBRUSH, HDC, HFONT, PAINTSTRUCT};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, DestroyWindow, DrawIconEx, PostQuitMessage, SetLayeredWindowAttributes, DI_NORMAL, LAYERED_WINDOW_ATTRIBUTES_FLAGS,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicIsize, Ordering};

pub const MSG_REFRESH: u32 = WM_APP + 1;
const ID_EDIT: isize = 201;
const TIMER_FADE: usize = 1;
const EM_SETSEL: u32 = 0xB1;

const BG: (u8, u8, u8) = (32, 32, 36);
const INPUT_BG: (u8, u8, u8) = (38, 38, 44);
const SEP: (u8, u8, u8) = (62, 62, 72);
const TEXT: (u8, u8, u8) = (240, 240, 244);
const DIM: (u8, u8, u8) = (150, 150, 162);
const ACCENT: (u8, u8, u8) = (96, 165, 250);
const SELECTED: (u8, u8, u8) = (58, 64, 84);
const CHIP_BG: (u8, u8, u8) = (46, 78, 130);
const WARN: (u8, u8, u8) = (255, 176, 110);
const CAT_COLORS: [(u8, u8, u8); 8] = [(240, 170, 90), (96, 165, 250), (150, 150, 170), (110, 200, 140), (180, 130, 240), (240, 130, 160), (240, 210, 100), (120, 200, 210)];

static PREV_EDIT: AtomicIsize = AtomicIsize::new(0);

#[derive(Clone)]
struct Chip {
    id: String,
    label: String,
}

struct Fonts {
    input: HFONT,
    title: HFONT,
    sub: HFONT,
    header: HFONT,
    chip: HFONT,
}

#[derive(Clone)]
struct Ctx {
    settings: SharedSettings,
    registry: Rc<Registry>,
}

struct Ui {
    hwnd: HWND,
    edit: HWND,
    ctx: Ctx,
    rows: Vec<Row>,
    sel: Option<usize>,
    scroll: usize,
    has_query: bool,
    chip: Option<Chip>,
    chip_w: i32,
    status: Option<String>,
    fonts: Fonts,
    dpi: u32,
    vis: Visibility,
    suppress_hide: bool,
    anchor: (i32, i32),
    fade: i32,
    bg_brush: isize,
}

thread_local! {
    static UI: RefCell<Option<Ui>> = RefCell::new(None);
}

fn st<R>(f: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    UI.with(|u| u.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(f)))
}

fn st_ref<R>(f: impl FnOnce(&Ui) -> R) -> Option<R> {
    UI.with(|u| u.try_borrow().ok().and_then(|b| b.as_ref().map(f)))
}

struct M {
    w: i32,
    input_h: i32,
    item_h: i32,
    header_h: i32,
    max_body: i32,
    status_h: i32,
    pad: i32,
}

fn metrics(dpi: u32, compact: bool) -> M {
    let s = |v: i32| util::scale(v, dpi);
    if compact {
        M { w: s(640), input_h: s(44), item_h: s(32), header_h: s(20), max_body: s(32) * 9, status_h: s(22), pad: s(12) }
    } else {
        M { w: s(680), input_h: s(56), item_h: s(46), header_h: s(26), max_body: s(46) * 8, status_h: s(26), pad: s(16) }
    }
}

fn m_of(u: &Ui) -> M {
    metrics(u.dpi, u.ctx.settings.borrow().palette.compact_mode)
}

fn make_fonts(dpi: u32, compact: bool) -> Fonts {
    let s = |v: i32| util::scale(v, dpi);
    Fonts {
        input: gdi::make_font("Segoe UI", s(if compact { 17 } else { 20 }), false, false),
        title: gdi::make_font("Segoe UI", s(if compact { 14 } else { 15 }), false, false),
        sub: gdi::make_font("Segoe UI", s(12), false, false),
        header: gdi::make_font("Segoe UI", s(11), true, false),
        chip: gdi::make_font("Segoe UI", s(14), true, false),
    }
}

fn free_fonts(f: &Fonts) {
    for h in [f.input, f.title, f.sub, f.header, f.chip] {
        gdi::delete_font(h);
    }
}

pub fn create(settings: SharedSettings, registry: Rc<Registry>) -> Result<HWND, String> {
    let brush = unsafe { CreateSolidBrush(rgb(BG.0, BG.1, BG.2)) };
    win::register_class("CmdPalPalette", Some(proc), Some(brush), win::app_icon())?;
    let hwnd = win::create(WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED, "CmdPalPalette", "CmdPal", WS_POPUP | 0x0200_0000, (0, 0, 680, 60), HWND::default(), 0)?;
    let dpi = util::dpi_for(hwnd);
    let compact = settings.borrow().palette.compact_mode;
    let fonts = make_fonts(dpi, compact);
    let edit = win::create(0, "EDIT", "", WS_CHILD | WS_VISIBLE | 0x80, (10, 10, 400, 30), hwnd, ID_EDIT)?;
    util::set_font(edit, fonts.input);
    PREV_EDIT.store(subclass::install(edit, edit_proc), Ordering::SeqCst);
    unsafe {
        let pref: i32 = 2; // DWMWCP_ROUND (Windows 11; harmless error on Windows 10)
        let _ = DwmSetWindowAttribute(hwnd, DWMWINDOWATTRIBUTE(33), &pref as *const i32 as *const std::ffi::c_void, 4);
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LAYERED_WINDOW_ATTRIBUTES_FLAGS(2));
    }
    UI.with(|u| {
        *u.borrow_mut() = Some(Ui {
            hwnd,
            edit,
            ctx: Ctx { settings, registry },
            rows: vec![],
            sel: None,
            scroll: 0,
            has_query: false,
            chip: None,
            chip_w: 0,
            status: None,
            fonts,
            dpi,
            vis: Visibility::default(),
            suppress_hide: false,
            anchor: (0, 0),
            fade: 255,
            bg_brush: brush.0 as isize,
        });
    });
    layout_edit();
    Ok(hwnd)
}

pub fn destroy(hwnd: HWND) {
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

pub fn set_status(_hwnd: HWND, msg: &str) {
    st(|u| u.status = Some(msg.to_string()));
    resize();
    redraw();
}

pub fn settings_changed(_hwnd: HWND) {
    rebuild_fonts();
    layout_edit();
    refresh(None, true);
}

fn redraw() {
    if let Some(h) = st_ref(|u| u.hwnd) {
        util::invalidate(h);
    }
}

fn rebuild_fonts() {
    let Some((dpi, compact, edit)) = st_ref(|u| (u.dpi, u.ctx.settings.borrow().palette.compact_mode, u.edit)) else { return };
    let nf = make_fonts(dpi, compact);
    util::set_font(edit, nf.input);
    if let Some(old) = st(|u| std::mem::replace(&mut u.fonts, nf)) {
        free_fonts(&old);
    }
}

/// Re-reads the window DPI (after moving between monitors) and rebuilds fonts if it changed.
fn ensure_dpi() {
    let Some((hwnd, cur)) = st_ref(|u| (u.hwnd, u.dpi)) else { return };
    let now = util::dpi_for(hwnd);
    if now != cur {
        st(|u| u.dpi = now);
        rebuild_fonts();
        layout_edit();
    }
}

fn layout_edit() {
    let Some((hwnd, edit, chip, chip_font, m)) = st_ref(|u| (u.hwnd, u.edit, u.chip.clone(), u.fonts.chip, m_of(u))) else { return };
    let chip_w = chip.map(|c| gdi::measure_in_window(hwnd, chip_font, &c.label) + util::scale(22, util::dpi_for(hwnd))).unwrap_or(0);
    st(|u| u.chip_w = chip_w);
    let x = m.pad + if chip_w > 0 { chip_w + util::scale(10, util::dpi_for(hwnd)) } else { 0 };
    let eh = (m.input_h * 6 / 10).max(20);
    util::move_window(edit, x, (m.input_h - eh) / 2, (m.w - x - m.pad).max(60), eh);
}

fn geometry(u: &Ui) -> Vec<(usize, i32, i32)> {
    let m = m_of(u);
    let n = selection::visible_count(&u.rows, u.scroll, m.max_body, m.item_h, m.header_h);
    let mut y = m.input_h + 1;
    let mut v = vec![];
    for i in u.scroll..(u.scroll + n).min(u.rows.len()) {
        let h = selection::row_height(&u.rows, i, m.item_h, m.header_h);
        v.push((i, y, h));
        y += h;
    }
    v
}

fn body_height(u: &Ui) -> i32 {
    let m = m_of(u);
    let rows_h: i32 = geometry(u).iter().map(|g| g.2).sum();
    let rows_h = if u.rows.is_empty() && u.has_query { m.item_h } else { rows_h };
    m.input_h + 1 + rows_h + if u.status.is_some() { m.status_h } else { 0 }
}

fn resize() {
    let Some((hwnd, x, y, w, h)) = st_ref(|u| (u.hwnd, u.anchor.0, u.anchor.1, m_of(u).w, body_height(u))) else { return };
    util::move_window(hwnd, x, y, w, h);
}

fn refresh(keep: Option<String>, clear_status: bool) {
    let Some((edit, chip, ctx)) = st_ref(|u| (u.edit, u.chip.clone(), u.ctx.clone())) else { return };
    let q = Query { text: util::get_text(edit), bang: chip.map(|c| c.id) };
    let rows = build_rows(&ctx.registry, &ctx.settings.borrow(), &q);
    let has_query = !q.text.trim().is_empty() || q.bang.is_some();
    st(|u| {
        let m = m_of(u);
        u.sel = keep.and_then(|k| rows.iter().position(|r| matches!(r, Row::Item(i) if i.key == k))).or_else(|| selection::first_item(&rows));
        u.scroll = 0;
        if let Some(s) = u.sel {
            u.scroll = selection::ensure_visible(&rows, s, 0, m.max_body, m.item_h, m.header_h);
        }
        u.rows = rows;
        u.has_query = has_query;
        if clear_status {
            u.status = None;
        }
    });
    resize();
    redraw();
}

fn on_text_changed() {
    let Some((edit, chip_set, registry)) = st_ref(|u| (u.edit, u.chip.is_some(), u.ctx.registry.clone())) else { return };
    if !chip_set {
        let text = util::get_text(edit);
        if let Some(m) = registry.match_bang(&text) {
            // alias + Space becomes a chip; the rest of the text is preserved
            st(|u| u.chip = Some(Chip { id: m.id, label: m.label }));
            layout_edit();
            util::set_text(edit, &m.rest); // re-enters here with the chip set -> refreshes
            let n = m.rest.encode_utf16().count();
            send(edit, EM_SETSEL, n, n as isize);
            return;
        }
    }
    refresh(None, true);
}

fn selected_item() -> Option<ResultItem> {
    st_ref(|u| match u.sel.and_then(|i| u.rows.get(i)) {
        Some(Row::Item(it)) => Some(it.clone()),
        _ => None,
    })
    .flatten()
}

fn move_sel(delta: isize, wrap: bool) {
    st(|u| {
        let m = m_of(u);
        u.sel = selection::step(&u.rows, u.sel, delta, wrap);
        if let Some(s) = u.sel {
            u.scroll = selection::ensure_visible(&u.rows, s, u.scroll, m.max_body, m.item_h, m.header_h);
        }
    });
    resize();
    redraw();
}

fn apply_vis(a: VAct, hwnd: HWND) {
    if a == VAct::Hide {
        util::show(hwnd, false);
    }
}

fn hide() {
    if let Some((a, h)) = st(|u| (u.vis.on_escape(), u.hwnd)) {
        apply_vis(a, h);
    }
}

fn activate() {
    let Some(item) = selected_item() else { return };
    let Some((hwnd, ctx)) = st_ref(|u| (u.hwnd, u.ctx.clone())) else { return };
    st(|u| u.suppress_hide = true);
    let go = match &item.confirm {
        Some(text) => shell::confirm(hwnd, "CmdPal - confirm", text),
        None => true,
    };
    let outcome = if go { Some(ctx.registry.dispatch(&item.action)) } else { None };
    st(|u| u.suppress_hide = false);
    match outcome {
        Some(o) if o.hide_palette => {
            if let Some((a, h)) = st(|u| (u.vis.on_command_done(), u.hwnd)) {
                apply_vis(a, h);
            }
        }
        Some(o) => {
            st(|u| u.status = o.status);
            resize();
            redraw();
        }
        None => {}
    }
}

fn hide_selected_result() {
    let Some(item) = selected_item() else { return };
    let Some(ctx) = st_ref(|u| u.ctx.clone()) else { return };
    if !item.hideable {
        st(|u| u.status = Some("This result cannot be hidden".into()));
        resize();
        redraw();
        return;
    }
    ctx.settings.borrow_mut().exclusions.push(item.key.clone());
    if let Some(p) = settings::settings_path() {
        let _ = settings::save_to(&p, &ctx.settings.borrow());
    }
    refresh(None, true);
}

fn backspace_chip() -> bool {
    let Some((edit, chip)) = st_ref(|u| (u.edit, u.chip.clone())) else { return false };
    if chip.is_some() && util::get_text(edit).is_empty() {
        st(|u| u.chip = None);
        layout_edit();
        refresh(None, true);
        return true;
    }
    false
}

fn on_key(vk: u32) -> bool {
    let (ctrl, shift) = (subclass::key_down(0x11), subclass::key_down(0x10));
    match vk {
        0x1B => {
            hide();
            true
        }
        0x28 => {
            move_sel(1, true);
            true
        }
        0x26 => {
            move_sel(-1, true);
            true
        }
        0x09 => {
            move_sel(if shift { -1 } else { 1 }, true);
            true
        }
        0x22 => {
            move_sel(6, false);
            true
        }
        0x21 => {
            move_sel(-6, false);
            true
        }
        0x0D => {
            activate();
            true
        }
        0x08 => backspace_chip(),
        0x2E if ctrl => {
            hide_selected_result();
            true
        }
        _ => false,
    }
}

unsafe extern "system" fn edit_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        0x0100 if on_key(wp.0 as u32) => return LRESULT(0), // WM_KEYDOWN
        0x0102 if matches!(wp.0, 0x0D | 0x09 | 0x1B) => return LRESULT(0), // swallow beeps
        _ => {}
    }
    subclass::call_prev(PREV_EDIT.load(Ordering::SeqCst), hwnd, msg, wp, lp)
}

fn show_palette() {
    let Some((hwnd, edit, animations)) = st_ref(|u| (u.hwnd, u.edit, u.ctx.settings.borrow().palette.animations)) else { return };
    st(|u| {
        u.chip = None;
        u.status = None;
    });
    layout_edit();
    util::set_text(edit, "");
    refresh(None, true);
    // two-pass placement: move first so the window picks up the target monitor's DPI
    let cursor = win::cursor_pos();
    let wa = win::work_area_at(cursor);
    util::move_window(hwnd, wa.left, wa.top, 10, 10);
    ensure_dpi();
    let (near, m) = st_ref(|u| (u.ctx.settings.borrow().palette.position_near_cursor, m_of(u))).unwrap_or((false, metrics(96, false)));
    let work = Rect { left: wa.left, top: wa.top, right: wa.right, bottom: wa.bottom };
    let full_h = m.input_h + 1 + m.max_body + m.status_h;
    let r = if near { placement::near_cursor(work, (cursor.x, cursor.y), m.w, full_h) } else { placement::centered_upper(work, m.w, full_h) };
    st(|u| u.anchor = (r.left, r.top));
    resize();
    unsafe {
        let a: u8 = if animations { 0 } else { 255 };
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), a, LAYERED_WINDOW_ATTRIBUTES_FLAGS(2));
    }
    st(|u| u.fade = if animations { 0 } else { 255 });
    util::show(hwnd, true);
    util::foreground(hwnd);
    util::focus(edit);
    if animations {
        util::set_timer(hwnd, TIMER_FADE, 16);
    }
}

fn draw(u: &Ui, dc: HDC, w: i32, total_h: i32) {
    let m = m_of(u);
    let full = gdi::rect(0, 0, w, total_h);
    gdi::fill(dc, &full, rgb(BG.0, BG.1, BG.2));
    gdi::fill(dc, &gdi::rect(0, 0, w, m.input_h), rgb(INPUT_BG.0, INPUT_BG.1, INPUT_BG.2));
    if let Some(c) = &u.chip {
        let ch = util::scale(28, u.dpi);
        let r = gdi::rect(m.pad, (m.input_h - ch) / 2, m.pad + u.chip_w, (m.input_h - ch) / 2 + ch);
        gdi::fill(dc, &r, rgb(CHIP_BG.0, CHIP_BG.1, CHIP_BG.2));
        gdi::draw_text(dc, u.fonts.chip, &c.label, &r, rgb(255, 255, 255), gdi::DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    }
    gdi::fill(dc, &gdi::rect(0, m.input_h, w, m.input_h + 1), rgb(SEP.0, SEP.1, SEP.2));
    let compact = u.ctx.settings.borrow().palette.compact_mode;
    let geo = geometry(u);
    for (i, y, h) in &geo {
        match &u.rows[*i] {
            Row::Header(c) => {
                let r = gdi::rect(m.pad, *y, w - m.pad, y + h);
                gdi::draw_text(dc, u.fonts.header, &c.title().to_uppercase(), &r, rgb(DIM.0, DIM.1, DIM.2), DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
            }
            Row::Item(it) => {
                if u.sel == Some(*i) {
                    gdi::fill(dc, &gdi::rect(util::scale(8, u.dpi), *y, w - util::scale(8, u.dpi), y + h), rgb(SELECTED.0, SELECTED.1, SELECTED.2));
                }
                let icon_sz = util::scale(16, u.dpi);
                let (ix, iy) = (m.pad, y + (h - icon_sz) / 2);
                let hicon = it.icon_path.as_deref().and_then(icons::get);
                match hicon {
                    Some(hi) => unsafe {
                        let _ = DrawIconEx(dc, ix, iy, hi, icon_sz, icon_sz, 0, HBRUSH::default(), DI_NORMAL);
                    },
                    None => {
                        let c = CAT_COLORS[(it.category as usize) % CAT_COLORS.len()];
                        gdi::fill(dc, &gdi::rect(ix, iy, ix + icon_sz, iy + icon_sz), rgb(c.0, c.1, c.2));
                    }
                }
                let tx = m.pad + icon_sz + util::scale(12, u.dpi);
                let flags = DT_LEFT | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS;
                if compact {
                    let mid = tx + (w - tx - m.pad) * 6 / 10;
                    gdi::draw_text(dc, u.fonts.title, &it.title, &gdi::rect(tx, *y, mid, y + h), rgb(TEXT.0, TEXT.1, TEXT.2), flags | DT_VCENTER);
                    gdi::draw_text(dc, u.fonts.sub, &it.subtitle, &gdi::rect(mid + 8, *y, w - m.pad, y + h), rgb(DIM.0, DIM.1, DIM.2), flags | DT_VCENTER | DT_RIGHT);
                } else {
                    let half = h / 2;
                    gdi::draw_text(dc, u.fonts.title, &it.title, &gdi::rect(tx, y + util::scale(4, u.dpi), w - m.pad, y + half + 2), rgb(TEXT.0, TEXT.1, TEXT.2), flags | DT_VCENTER);
                    gdi::draw_text(dc, u.fonts.sub, &it.subtitle, &gdi::rect(tx, y + half, w - m.pad, y + h - 2), rgb(DIM.0, DIM.1, DIM.2), flags | DT_VCENTER);
                }
            }
        }
    }
    let rows_end = geo.last().map(|g| g.1 + g.2).unwrap_or(m.input_h + 1);
    if u.rows.is_empty() && u.has_query {
        gdi::draw_text(dc, u.fonts.title, "No results", &gdi::rect(m.pad, rows_end, w - m.pad, rows_end + m.item_h), rgb(DIM.0, DIM.1, DIM.2), DT_LEFT | DT_VCENTER | DT_SINGLELINE);
    }
    if let Some(s) = &u.status {
        let y0 = if u.rows.is_empty() && u.has_query { rows_end + m.item_h } else { rows_end };
        gdi::draw_text(dc, u.fonts.sub, s, &gdi::rect(m.pad, y0, w - m.pad, y0 + m.status_h), rgb(WARN.0, WARN.1, WARN.2), DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS);
    }
}

fn paint(hwnd: HWND) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        let rc = util::client_rect(hwnd);
        let buf = Buffer::new(hdc, rc.right, rc.bottom);
        st_ref(|u| draw(u, buf.dc, rc.right, rc.bottom));
        buf.present(hdc);
        let _ = EndPaint(hwnd, &ps);
    }
}

fn row_at(y: i32) -> Option<usize> {
    st_ref(|u| geometry(u).into_iter().find(|(_, ry, h)| y >= *ry && y < ry + h).map(|(i, _, _)| i)).flatten()
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        0x0312 => {
            // WM_HOTKEY
            if let Some((a, h)) = st(|u| (u.vis.on_hotkey(), u.hwnd)) {
                if a == VAct::Show {
                    show_palette();
                } else {
                    apply_vis(a, h);
                }
            }
            LRESULT(0)
        }
        0x0006 => {
            // WM_ACTIVATE
            if loword(wp.0) == 0 {
                if let Some((a, h)) = st(|u| if u.suppress_hide { (VAct::Nothing, u.hwnd) } else { (u.vis.on_focus_lost(), u.hwnd) }) {
                    apply_vis(a, h);
                }
            }
            LRESULT(0)
        }
        0x0111 => {
            // WM_COMMAND: EN_CHANGE from the query box
            if util::hiword(wp.0) == 0x0300 {
                on_text_changed();
            }
            LRESULT(0)
        }
        0x0133 => {
            // WM_CTLCOLOREDIT
            let hdc = HDC(wp.0 as *mut std::ffi::c_void);
            let _ = SetTextColor(hdc, rgb(TEXT.0, TEXT.1, TEXT.2));
            let _ = SetBkColor(hdc, rgb(INPUT_BG.0, INPUT_BG.1, INPUT_BG.2));
            LRESULT(st_ref(|u| u.bg_brush).unwrap_or(0))
        }
        0x0014 => LRESULT(1), // WM_ERASEBKGND
        0x000F => {
            paint(hwnd);
            LRESULT(0)
        }
        0x0201 => {
            // WM_LBUTTONDOWN: select and run
            let y = ((lp.0 >> 16) & 0xFFFF) as i16 as i32;
            if let Some(i) = row_at(y) {
                if selection::is_item(&st_ref(|u| u.rows.clone()).unwrap_or_default(), i) {
                    st(|u| u.sel = Some(i));
                    activate();
                }
            }
            LRESULT(0)
        }
        0x0205 => {
            // WM_RBUTTONUP: hide this result
            let y = ((lp.0 >> 16) & 0xFFFF) as i16 as i32;
            if let Some(i) = row_at(y) {
                if selection::is_item(&st_ref(|u| u.rows.clone()).unwrap_or_default(), i) {
                    st(|u| u.sel = Some(i));
                    hide_selected_result();
                }
            }
            LRESULT(0)
        }
        0x020A => {
            // WM_MOUSEWHEEL
            let delta = (wp.0 >> 16) as u16 as i16 as i32;
            st(|u| {
                let n = u.rows.len().saturating_sub(1);
                u.scroll = if delta > 0 { u.scroll.saturating_sub(3) } else { (u.scroll + 3).min(n) };
            });
            resize();
            redraw();
            LRESULT(0)
        }
        0x0113 => {
            // WM_TIMER: short fade-in
            if wp.0 == TIMER_FADE {
                let a = st(|u| {
                    u.fade = (u.fade + 64).min(255);
                    u.fade
                })
                .unwrap_or(255);
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), a as u8, LAYERED_WINDOW_ATTRIBUTES_FLAGS(2));
                if a >= 255 {
                    util::kill_timer(hwnd, TIMER_FADE);
                }
            }
            LRESULT(0)
        }
        MSG_REFRESH => {
            // results changed on a background thread: keep selection, never touch the query text
            let key = selected_item().map(|i| i.key);
            refresh(key, false);
            LRESULT(0)
        }
        0x0010 => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        0x0002 => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

#[allow(dead_code)]
fn _category_unused(_: Category) {}
