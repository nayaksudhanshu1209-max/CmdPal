//! Settings window: section list on the left, page controls built from `logic::pages` on the
//! right. Every control edits a draft copy of the canonical `Settings`; Apply validates and saves.
use crate::core::contracts::{Registry, SharedSettings};
use crate::core::settings::{self, Settings};
use crate::features::file_search::api as files_api;
use crate::features::notes::api as notes_api;
use crate::features::settings::logic::binding::{self, FieldKind};
use crate::features::settings::logic::pages::{pages, Item};
use crate::platform::windows::gdi;
use crate::platform::windows::util::{self, loword, hiword, send, wide};
use crate::platform::windows::window::{self as win, WS_BORDER, WS_CHILD, WS_CLIPCHILDREN, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL};
use crate::platform::windows::{proc, shell};
use ::windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use ::windows::Win32::Graphics::Gdi::{HBRUSH, HFONT};
use ::windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, DestroyWindow, WM_CLOSE, WM_SIZE};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

static BUILDING: AtomicBool = AtomicBool::new(false);

const ID_SECTIONS: i32 = 10;
const ID_APPLY: i32 = 900;
const ID_CLOSE: i32 = 901;
const ID_STATUS: i32 = 902;
const LB_ADDSTRING: u32 = 0x180;
const LB_RESETCONTENT: u32 = 0x184;
const LB_SETCURSEL: u32 = 0x186;
const LB_GETCURSEL: u32 = 0x188;
const BM_GETCHECK: u32 = 0xF0;
const BM_SETCHECK: u32 = 0xF1;
const CONTENT_X: i32 = 200;
const LABEL_W: i32 = 270;

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Add,
    Update,
    Remove,
    Up,
    Down,
    Browse(usize),
}

#[derive(Clone)]
enum Kind {
    Check(&'static str),
    Edit(&'static str),
    ListBox(&'static str),
    Field(&'static str, usize),
    FieldCheck(&'static str, usize),
    ListBtn(&'static str, Op),
    PageBtn(&'static str),
    IndexStatus,
    NoteList,
    NoteName,
}

#[derive(Clone)]
struct Ctl {
    id: i32,
    hwnd: HWND,
    kind: Kind,
}

struct Ui {
    hwnd: HWND,
    sections: HWND,
    status: HWND,
    controls: Vec<Ctl>,
    content: Vec<HWND>,
    draft: Settings,
    page: usize,
    dirty: bool,
    settings: SharedSettings,
    registry: Rc<Registry>,
    font: HFONT,
}

thread_local! {
    static UI: RefCell<Option<Ui>> = RefCell::new(None);
}

fn st<R>(f: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    UI.with(|u| u.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(f)))
}

fn set_status(msg: &str) {
    if let Some(h) = st(|u| u.status) {
        util::set_text(h, msg);
    }
}

fn add(parent: HWND, font: HFONT, class: &str, text: &str, style: u32, r: (i32, i32, i32, i32), id: i32) -> HWND {
    let h = win::create(0, class, text, style | WS_CHILD | WS_VISIBLE, r, parent, id as isize).unwrap_or_default();
    util::set_font(h, font);
    h
}

pub fn open(page_id: &str, settings: SharedSettings, registry: Rc<Registry>) -> Result<(), String> {
    ensure(settings.clone(), registry)?;
    let idx = pages().iter().position(|p| p.id == page_id).unwrap_or(0);
    let (hwnd, sections) = st(|u| {
        u.draft = settings.borrow().clone();
        u.dirty = false;
        (u.hwnd, u.sections)
    })
    .ok_or("settings window busy")?;
    send(sections, LB_SETCURSEL, idx, 0);
    build_page(idx);
    set_status("");
    util::set_timer(hwnd, 1, 1000);
    util::show(hwnd, true);
    util::foreground(hwnd);
    shell::set_dialog_window(util::hwnd_to_isize(hwnd));
    Ok(())
}

fn ensure(settings: SharedSettings, registry: Rc<Registry>) -> Result<(), String> {
    if UI.with(|u| u.borrow().is_some()) {
        return Ok(());
    }
    let bg = HBRUSH(6usize as *mut std::ffi::c_void); // COLOR_WINDOW + 1
    win::register_class("CmdPalSettings", Some(proc_), Some(bg), win::app_icon())?;
    let dpi = 96;
    let wa = win::work_area_at(win::cursor_pos());
    let (w, h) = (util::scale(980, dpi).min(wa.right - wa.left - 20), util::scale(740, dpi).min(wa.bottom - wa.top - 20));
    let (x, y) = (wa.left + (wa.right - wa.left - w) / 2, wa.top + (wa.bottom - wa.top - h) / 2);
    let hwnd = win::create(0, "CmdPalSettings", "CmdPal settings", WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, (x, y, w, h), HWND::default(), 0)?;
    let font = gdi::make_font("Segoe UI", util::scale(16, util::dpi_for(hwnd)), false, false);
    let sections = add(hwnd, font, "LISTBOX", "", 1 | WS_VSCROLL | WS_BORDER | WS_TABSTOP, (12, 12, 176, 300), ID_SECTIONS);
    for p in pages() {
        send(sections, LB_ADDSTRING, 0, wide(p.title).as_ptr() as isize);
    }
    add(hwnd, font, "BUTTON", "Apply", WS_TABSTOP, (0, 0, 100, 30), ID_APPLY);
    add(hwnd, font, "BUTTON", "Close", WS_TABSTOP, (0, 0, 100, 30), ID_CLOSE);
    let status = add(hwnd, font, "STATIC", "", 0, (0, 0, 400, 22), ID_STATUS);
    let draft = settings.borrow().clone();
    UI.with(|u| {
        *u.borrow_mut() = Some(Ui { hwnd, sections, status, controls: vec![], content: vec![], draft, page: 0, dirty: false, settings, registry, font });
    });
    layout(hwnd);
    Ok(())
}

fn layout(hwnd: HWND) {
    let rc = util::client_rect(hwnd);
    let Some((sections, status)) = st(|u| (u.sections, u.status)) else { return };
    util::move_window(sections, 12, 12, 176, rc.bottom - 24);
    let by = rc.bottom - 40;
    util::move_window(status, CONTENT_X, by + 6, (rc.right - CONTENT_X - 240).max(100), 24);
    let ids = [(ID_APPLY, rc.right - 224), (ID_CLOSE, rc.right - 112)];
    for (id, x) in ids {
        if let Some(h) = child(hwnd, id) {
            util::move_window(h, x, by, 100, 30);
        }
    }
}

fn child(parent: HWND, id: i32) -> Option<HWND> {
    use ::windows::Win32::UI::WindowsAndMessaging::GetDlgItem;
    unsafe { GetDlgItem(parent, id).ok() }
}

fn build_page(idx: usize) {
    BUILDING.store(true, Ordering::SeqCst);
    let Some((hwnd, font, draft, old)) = st(|u| {
        u.page = idx;
        (u.hwnd, u.font, u.draft.clone(), std::mem::take(&mut u.content))
    }) else {
        BUILDING.store(false, Ordering::SeqCst);
        return;
    };
    for h in old {
        unsafe {
            let _ = DestroyWindow(h);
        }
    }
    let page = &pages()[idx];
    let rc = util::client_rect(hwnd);
    let cw = (rc.right - CONTENT_X - 16).max(300);
    let mut y = 12;
    let mut next = 1000;
    let mut ctls: Vec<Ctl> = vec![];
    let mut content: Vec<HWND> = vec![];
    let mut mk = |class: &str, text: &str, style: u32, r: (i32, i32, i32, i32), kind: Option<Kind>, ctls: &mut Vec<Ctl>| -> HWND {
        next += 1;
        let h = add(hwnd, font, class, text, style, r, next);
        content.push(h);
        if let Some(k) = kind {
            ctls.push(Ctl { id: next, hwnd: h, kind: k });
        }
        h
    };
    for item in &page.items {
        match *item {
            Item::Check(key, label) => {
                let h = mk("BUTTON", label, WS_TABSTOP | 3, (CONTENT_X, y, cw, 24), Some(Kind::Check(key)), &mut ctls);
                send(h, BM_SETCHECK, usize::from(binding::get_value(&draft, key) == "1"), 0);
                y += 28;
            }
            Item::Edit(key, label) => {
                mk("STATIC", label, 0, (CONTENT_X, y, cw, 20), None, &mut ctls);
                let h = mk("EDIT", "", WS_BORDER | WS_TABSTOP | 0x80, (CONTENT_X, y + 22, cw.min(420), 26), Some(Kind::Edit(key)), &mut ctls);
                util::set_text(h, &binding::get_value(&draft, key));
                y += 56;
            }
            Item::Text(t) => {
                mk("STATIC", t, 0, (CONTENT_X, y, cw, 44), None, &mut ctls);
                y += 48;
            }
            Item::IndexStatus => {
                let h = mk("STATIC", "", 0, (CONTENT_X, y, cw, 44), Some(Kind::IndexStatus), &mut ctls);
                util::set_text(h, &files_api::status_text());
                y += 48;
            }
            Item::Buttons(list) => {
                let mut x = CONTENT_X;
                for (id, label) in list {
                    mk("BUTTON", label, WS_TABSTOP, (x, y, 150, 28), Some(Kind::PageBtn(id)), &mut ctls);
                    x += 156;
                }
                y += 38;
            }
            Item::List(list, caption) => {
                mk("STATIC", caption, 0, (CONTENT_X, y, cw, 20), None, &mut ctls);
                y += 22;
                let lb = mk("LISTBOX", "", 1 | WS_VSCROLL | WS_BORDER | WS_TABSTOP, (CONTENT_X, y, cw, 118), Some(Kind::ListBox(list)), &mut ctls);
                y += 124;
                let fields = binding::fields(list);
                let rows = binding::get_rows(&draft, list);
                for r in &rows {
                    send(lb, LB_ADDSTRING, 0, wide(&binding::summary(list, r)).as_ptr() as isize);
                }
                for (i, f) in fields.iter().enumerate() {
                    match f.kind {
                        FieldKind::Bool => {
                            mk("BUTTON", f.label, WS_TABSTOP | 3, (CONTENT_X, y, cw, 24), Some(Kind::FieldCheck(list, i)), &mut ctls);
                        }
                        k => {
                            mk("STATIC", f.label, 0, (CONTENT_X, y + 3, LABEL_W, 22), None, &mut ctls);
                            let browse = k == FieldKind::Path;
                            let ew = cw - LABEL_W - 5 - if browse { 90 } else { 0 };
                            mk("EDIT", "", WS_BORDER | WS_TABSTOP | 0x80, (CONTENT_X + LABEL_W + 5, y, ew.max(100), 26), Some(Kind::Field(list, i)), &mut ctls);
                            if browse {
                                mk("BUTTON", "Browse...", WS_TABSTOP, (CONTENT_X + cw - 84, y, 84, 26), Some(Kind::ListBtn(list, Op::Browse(i))), &mut ctls);
                            }
                        }
                    }
                    y += 30;
                }
                let mut x = CONTENT_X;
                for (label, op) in [("Add", Op::Add), ("Update selected", Op::Update), ("Remove", Op::Remove), ("Move up", Op::Up), ("Move down", Op::Down)] {
                    mk("BUTTON", label, WS_TABSTOP, (x, y, 116, 28), Some(Kind::ListBtn(list, op)), &mut ctls);
                    x += 122;
                }
                y += 40;
            }
            Item::NotesManager => {
                mk("STATIC", "Your notes (stored in %APPDATA%\\CmdPal\\notes)", 0, (CONTENT_X, y, cw, 20), None, &mut ctls);
                y += 22;
                let lb = mk("LISTBOX", "", 1 | WS_VSCROLL | WS_BORDER | WS_TABSTOP, (CONTENT_X, y, cw, 200), Some(Kind::NoteList), &mut ctls);
                for n in notes_api::list_notes() {
                    send(lb, LB_ADDSTRING, 0, wide(&n.name).as_ptr() as isize);
                }
                y += 206;
                mk("STATIC", "New name (for Rename)", 0, (CONTENT_X, y + 3, LABEL_W, 22), None, &mut ctls);
                mk("EDIT", "", WS_BORDER | WS_TABSTOP | 0x80, (CONTENT_X + LABEL_W + 5, y, (cw - LABEL_W - 5).max(100), 26), Some(Kind::NoteName), &mut ctls);
                y += 36;
                let mut x = CONTENT_X;
                for (id, label) in [("note_open", "Open"), ("note_rename", "Rename"), ("note_delete", "Delete..."), ("note_refresh", "Refresh list")] {
                    mk("BUTTON", label, WS_TABSTOP, (x, y, 130, 28), Some(Kind::PageBtn(id)), &mut ctls);
                    x += 136;
                }
                y += 40;
            }
        }
    }
    let _ = y;
    // select first row of each list and fill its fields
    for c in &ctls {
        if let Kind::ListBox(list) = c.kind {
            if !binding::get_rows(&draft, list).is_empty() {
                send(c.hwnd, LB_SETCURSEL, 0, 0);
            }
        }
    }
    st(|u| {
        u.content = content;
        u.controls = ctls;
    });
    for c in st(|u| u.controls.clone()).unwrap_or_default() {
        if let Kind::ListBox(list) = c.kind {
            fill_fields(list);
        }
    }
    BUILDING.store(false, Ordering::SeqCst);
}

fn ctl_of(list: &str, f: impl Fn(&Kind) -> bool) -> Vec<Ctl> {
    st(|u| u.controls.iter().filter(|c| f(&c.kind)).cloned().collect::<Vec<Ctl>>()).unwrap_or_default().into_iter().filter(|c| match c.kind {
        Kind::Field(l, _) | Kind::FieldCheck(l, _) | Kind::ListBox(l) | Kind::ListBtn(l, _) => l == list,
        _ => true,
    }).collect()
}

fn list_box(list: &str) -> Option<HWND> {
    ctl_of(list, |k| matches!(k, Kind::ListBox(_))).first().map(|c| c.hwnd)
}

fn fill_fields(list: &str) {
    BUILDING.store(true, Ordering::SeqCst);
    let Some(lb) = list_box(list) else { return };
    let sel = send(lb, LB_GETCURSEL, 0, 0);
    let rows = st(|u| binding::get_rows(&u.draft, list)).unwrap_or_default();
    let row = (sel >= 0).then(|| rows.get(sel as usize).cloned()).flatten();
    for c in ctl_of(list, |k| matches!(k, Kind::Field(..) | Kind::FieldCheck(..))) {
        match c.kind {
            Kind::Field(_, i) => util::set_text(c.hwnd, row.as_ref().and_then(|r| r.get(i)).map(|s| s.as_str()).unwrap_or("")),
            Kind::FieldCheck(_, i) => {
                let on = row.as_ref().and_then(|r| r.get(i)).map(|v| v == "1").unwrap_or(true);
                send(c.hwnd, BM_SETCHECK, usize::from(on), 0);
            }
            _ => {}
        }
    }
    BUILDING.store(false, Ordering::SeqCst);
}

fn refresh_list(list: &'static str, select: isize) {
    let Some(lb) = list_box(list) else { return };
    let rows = st(|u| binding::get_rows(&u.draft, list)).unwrap_or_default();
    send(lb, LB_RESETCONTENT, 0, 0);
    for r in &rows {
        send(lb, LB_ADDSTRING, 0, wide(&binding::summary(list, r)).as_ptr() as isize);
    }
    let s = if rows.is_empty() { -1 } else { select.clamp(0, rows.len() as isize - 1) };
    send(lb, LB_SETCURSEL, s as usize, 0);
    fill_fields(list);
}

fn new_errors(before: &Settings, after: &Settings) -> Vec<String> {
    let old = settings::validate(before);
    settings::validate(after).into_iter().filter(|e| !old.contains(e)).collect()
}

fn list_op(list: &'static str, op: Op) {
    let fields = binding::fields(list);
    let Some(lb) = list_box(list) else { return };
    let sel = send(lb, LB_GETCURSEL, 0, 0);
    if let Op::Browse(i) = op {
        let owner = st(|u| u.hwnd).unwrap_or_default();
        if let Some(p) = proc::browse_folder(owner, "Choose a folder") {
            if let Some(c) = ctl_of(list, |k| matches!(k, Kind::Field(_, j) if *j == i)).first() {
                util::set_text(c.hwnd, &p);
            }
        }
        return;
    }
    // read the field controls
    let mut vals = vec![String::new(); fields.len()];
    for c in ctl_of(list, |k| matches!(k, Kind::Field(..) | Kind::FieldCheck(..))) {
        match c.kind {
            Kind::Field(_, i) => vals[i] = util::get_text(c.hwnd),
            Kind::FieldCheck(_, i) => vals[i] = if send(c.hwnd, BM_GETCHECK, 0, 0) == 1 { "1".into() } else { "0".into() },
            _ => {}
        }
    }
    let Some(draft) = st(|u| u.draft.clone()) else { return };
    let mut rows = binding::get_rows(&draft, list);
    let n = rows.len() as isize;
    let mut select = sel;
    match op {
        Op::Add => {
            if vals.iter().zip(&fields).all(|(v, f)| f.kind == FieldKind::Bool || v.trim().is_empty()) {
                set_status("Fill in the fields first, then click Add.");
                return;
            }
            rows.push(vals);
            select = n;
        }
        Op::Update if sel >= 0 && sel < n => rows[sel as usize] = vals,
        Op::Remove if sel >= 0 && sel < n => {
            rows.remove(sel as usize);
            select = sel.min(n - 2);
        }
        Op::Up if sel > 0 && sel < n => {
            rows.swap(sel as usize, sel as usize - 1);
            select = sel - 1;
        }
        Op::Down if sel >= 0 && sel + 1 < n => {
            rows.swap(sel as usize, sel as usize + 1);
            select = sel + 1;
        }
        _ => {
            set_status("Select an item in the list first.");
            return;
        }
    }
    let mut cand = draft.clone();
    if let Err(e) = binding::set_rows(&mut cand, list, &rows) {
        set_status(&e);
        return;
    }
    let errs = new_errors(&draft, &cand);
    if !errs.is_empty() {
        set_status(&errs.join("; "));
        return;
    }
    st(|u| {
        u.draft = cand;
        u.dirty = true;
    });
    refresh_list(list, select);
    set_status("Change not saved yet - click Apply.");
}

fn apply() {
    let Some((draft, settings, registry)) = st(|u| (u.draft.clone(), u.settings.clone(), u.registry.clone())) else { return };
    let errs = settings::validate(&draft);
    if !errs.is_empty() {
        set_status(&format!("Cannot save: {}", errs.join("; ")));
        return;
    }
    if let Some(p) = settings::settings_path() {
        if let Err(e) = settings::save_to(&p, &draft) {
            set_status(&format!("Could not write settings: {e}"));
            return;
        }
    }
    *settings.borrow_mut() = draft;
    registry.notify_settings_changed();
    st(|u| u.dirty = false);
    set_status("Saved.");
}

fn close_window() {
    let Some((hwnd, dirty)) = st(|u| (u.hwnd, u.dirty)) else { return };
    if dirty && !shell::confirm(hwnd, "CmdPal settings", "You have changes that are not applied. Close without saving?") {
        return;
    }
    util::kill_timer(hwnd, 1);
    util::show(hwnd, false);
    shell::set_dialog_window(0);
}

fn page_button(id: &'static str) {
    match id {
        "add_desktop" | "add_documents" | "add_downloads" => {
            let name = &id[4..];
            let name = format!("{}{}", &name[..1].to_uppercase(), &name[1..]);
            let Some(path) = crate::core::paths::known_folder(&name) else { return };
            let Some(mut cand) = st(|u| u.draft.clone()) else { return };
            if cand.files.roots.iter().any(|r| r.path.eq_ignore_ascii_case(&path)) {
                set_status("That folder is already in the list.");
                return;
            }
            cand.files.roots.push(settings::FileRoot { path, enabled: true });
            st(|u| {
                u.draft = cand;
                u.dirty = true;
            });
            refresh_list("roots", isize::MAX);
            set_status("Change not saved yet - click Apply.");
        }
        "pause" => {
            let paused = files_api::toggle_pause();
            set_status(if paused { "File indexing paused." } else { "File indexing resumed." });
        }
        "rebuild" => {
            files_api::rebuild();
            set_status("Rebuilding the file index in the background...");
        }
        "clear" => {
            if shell::confirm(st(|u| u.hwnd).unwrap_or_default(), "Clear file index", "Remove the stored file index? It can be rebuilt at any time.") {
                files_api::clear();
                set_status("File index cleared.");
            }
        }
        "note_open" | "note_rename" | "note_delete" | "note_refresh" => note_button(id),
        _ => {}
    }
}

fn note_button(id: &str) {
    let ctls = st(|u| u.controls.clone()).unwrap_or_default();
    let lb = ctls.iter().find(|c| matches!(c.kind, Kind::NoteList)).map(|c| c.hwnd);
    let name_edit = ctls.iter().find(|c| matches!(c.kind, Kind::NoteName)).map(|c| c.hwnd);
    let notes = notes_api::list_notes();
    let sel = lb.map(|l| send(l, LB_GETCURSEL, 0, 0)).unwrap_or(-1);
    let chosen = (sel >= 0).then(|| notes.get(sel as usize).map(|n| n.name.clone())).flatten();
    let hwnd = st(|u| u.hwnd).unwrap_or_default();
    let page = st(|u| u.page).unwrap_or(0);
    match (id, chosen) {
        ("note_refresh", _) => build_page(page),
        (_, None) => set_status("Select a note first."),
        ("note_open", Some(n)) => {
            if let Err(e) = notes_api::open_note(&n) {
                set_status(&e);
            }
        }
        ("note_rename", Some(n)) => {
            let new = name_edit.map(util::get_text).unwrap_or_default();
            match notes_api::rename_note(&n, &new) {
                Ok(_) => build_page(page),
                Err(e) => set_status(&e),
            }
        }
        ("note_delete", Some(n)) => {
            if shell::confirm(hwnd, "Delete note", &format!("Delete the note \"{n}\"? This cannot be undone.")) {
                match notes_api::delete_note(&n) {
                    Ok(()) => build_page(page),
                    Err(e) => set_status(&e),
                }
            }
        }
        _ => {}
    }
}

fn on_command(id: i32, code: u32) {
    if BUILDING.load(Ordering::SeqCst) {
        return;
    }
    match id {
        ID_APPLY if code == 0 => return apply(),
        ID_CLOSE if code == 0 => return close_window(),
        ID_SECTIONS if code == 1 => {
            if let Some(s) = st(|u| u.sections) {
                let i = send(s, LB_GETCURSEL, 0, 0);
                if i >= 0 {
                    build_page(i as usize);
                    set_status("");
                }
            }
            return;
        }
        _ => {}
    }
    let Some(ctl) = st(|u| u.controls.iter().find(|c| c.id == id).cloned()).flatten() else { return };
    match ctl.kind {
        Kind::ListBox(list) if code == 1 => fill_fields(list),
        Kind::ListBtn(list, op) if code == 0 => list_op(list, op),
        Kind::PageBtn(p) if code == 0 => page_button(p),
        Kind::Check(key) if code == 0 => set_scalar(key, if send(ctl.hwnd, BM_GETCHECK, 0, 0) == 1 { "1" } else { "0" }),
        Kind::Edit(key) if code == 0x300 => set_scalar(key, &util::get_text(ctl.hwnd)),
        _ => {}
    }
}

fn set_scalar(key: &str, value: &str) {
    let Some(draft) = st(|u| u.draft.clone()) else { return };
    let mut cand = draft.clone();
    if let Err(e) = binding::set_value(&mut cand, key, value) {
        set_status(&e);
        return;
    }
    let errs = new_errors(&draft, &cand);
    if !errs.is_empty() {
        set_status(&errs.join("; "));
        return;
    }
    st(|u| {
        u.draft = cand;
        u.dirty = true;
    });
    set_status("Change not saved yet - click Apply.");
}

unsafe extern "system" fn proc_(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        0x0111 => {
            on_command(loword(wp.0) as i32, hiword(wp.0));
            LRESULT(0)
        }
        WM_SIZE => {
            layout(hwnd);
            LRESULT(0)
        }
        0x0113 => {
            // WM_TIMER: refresh the live index status text
            if let Some(h) = st(|u| u.controls.iter().find(|c| matches!(c.kind, Kind::IndexStatus)).map(|c| c.hwnd)).flatten() {
                util::set_text(h, &files_api::status_text());
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            close_window();
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
