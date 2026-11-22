//! Thin wrapper over the system RichEdit 5.0 control (Msftedit.dll, part of Windows - no extra
//! runtime). Constants/structs are declared locally so they do not depend on crate typings.
use super::super::markdown_model::model::*;
use crate::platform::windows::util::{rgb, send, wide};
use crate::platform::windows::window::{create, WS_CHILD, WS_VISIBLE, WS_VSCROLL};
use ::windows::core::w;
use ::windows::Win32::Foundation::HWND;
use ::windows::Win32::System::LibraryLoader::LoadLibraryW;

const EM_GETSEL_EX: u32 = 0x0434;
const EM_SETSEL_EX: u32 = 0x0437;
const EM_LINEFROMCHAR_EX: u32 = 0x0436;
const EM_LINEINDEX: u32 = 0x00BB;
const EM_GETLINECOUNT: u32 = 0x00BA;
const EM_GETFIRSTVISIBLELINE: u32 = 0x00CE;
const EM_LINESCROLL: u32 = 0x00B6;
const EM_REPLACESEL: u32 = 0x00C2;
const EM_EMPTYUNDOBUFFER: u32 = 0x00CD;
const EM_SETMARGINS: u32 = 0x00D3;
const EM_SETCHARFORMAT: u32 = 0x0444;
const EM_SETEVENTMASK: u32 = 0x0445;
const EM_GETEVENTMASK: u32 = 0x043B;
const EM_SETBKGNDCOLOR: u32 = 0x0443;
const EM_PASTESPECIAL: u32 = 0x0440;
const EM_SETTEXTEX: u32 = 0x0461;
const EM_GETTEXTEX: u32 = 0x045E;
const EM_GETTEXTLENGTHEX: u32 = 0x045F;
const WM_SETREDRAW: u32 = 0x000B;
pub const ENM_CHANGE: isize = 0x1;
pub const ENM_SELCHANGE: isize = 0x80000;
pub const EN_CHANGE: u32 = 0x0300;
pub const EN_SELCHANGE: u32 = 0x0702;

const SCF_SELECTION: usize = 1;
const CFM_BOLD: u32 = 0x1;
const CFM_ITALIC: u32 = 0x2;
const CFM_UNDERLINE: u32 = 0x4;
const CFM_STRIKEOUT: u32 = 0x8;
const CFM_HIDDEN: u32 = 0x100;
const CFM_BACKCOLOR: u32 = 0x0400_0000;
const CFM_FACE: u32 = 0x2000_0000;
const CFM_COLOR: u32 = 0x4000_0000;
const CFM_SIZE: u32 = 0x8000_0000;
const CFE_AUTOBACKCOLOR: u32 = 0x0400_0000;

pub const BG: (u8, u8, u8) = (30, 30, 34);
const TEXT: (u8, u8, u8) = (232, 232, 236);
const DIMC: (u8, u8, u8) = (110, 110, 122);
const LINKC: (u8, u8, u8) = (110, 170, 250);
const QUOTEC: (u8, u8, u8) = (165, 165, 178);
const CODEC: (u8, u8, u8) = (240, 175, 125);
const CODEBG: (u8, u8, u8) = (50, 50, 58);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CharFormat2 {
    cb_size: u32,
    mask: u32,
    effects: u32,
    height: i32,
    offset: i32,
    text_color: u32,
    charset: u8,
    pitch: u8,
    face: [u16; 32],
    weight: u16,
    spacing: i16,
    back_color: u32,
    lcid: u32,
    reserved: u32,
    style: i16,
    kerning: u16,
    underline_type: u8,
    animation: u8,
    rev_author: u8,
    underline_color: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CharRange {
    min: i32,
    max: i32,
}

#[repr(C)]
struct SetTextEx {
    flags: u32,
    codepage: u32,
}

#[repr(C)]
struct GetTextEx {
    cb: u32,
    flags: u32,
    codepage: u32,
    default_char: *const u8,
    used_default: *mut i32,
}

#[repr(C)]
struct GetTextLengthEx {
    flags: u32,
    codepage: u32,
}

fn col(c: (u8, u8, u8)) -> u32 {
    rgb(c.0, c.1, c.2).0
}

fn set_face(c: &mut CharFormat2, name: &str) {
    c.face = [0; 32];
    for (i, u) in name.encode_utf16().take(31).enumerate() {
        c.face[i] = u;
    }
}

pub fn base_format() -> CharFormat2 {
    let mut c: CharFormat2 = unsafe { std::mem::zeroed() };
    c.cb_size = std::mem::size_of::<CharFormat2>() as u32;
    c.mask = CFM_SIZE | CFM_COLOR | CFM_FACE | CFM_BOLD | CFM_ITALIC | CFM_UNDERLINE | CFM_STRIKEOUT | CFM_HIDDEN | CFM_BACKCOLOR;
    c.effects = CFE_AUTOBACKCOLOR;
    c.height = 220;
    c.text_color = col(TEXT);
    set_face(&mut c, "Segoe UI");
    c
}

pub fn format_for(flags: u32, heading: u8) -> CharFormat2 {
    let mut c = base_format();
    if heading > 0 {
        c.height = match heading {
            1 => 340,
            2 => 290,
            3 => 255,
            _ => 235,
        };
    }
    if flags & BOLD != 0 {
        c.effects |= 0x1;
    }
    if flags & (ITALIC | QUOTE | IMAGE) != 0 {
        c.effects |= 0x2;
    }
    if flags & STRIKE != 0 {
        c.effects |= 0x8;
    }
    if flags & LINK != 0 {
        c.effects |= 0x4;
        c.text_color = col(LINKC);
    }
    if flags & QUOTE != 0 {
        c.text_color = col(QUOTEC);
    }
    if flags & (CODE | BLOCKCODE | TABLE) != 0 {
        set_face(&mut c, "Consolas");
        c.height = 200;
        c.text_color = col(CODEC);
    }
    if flags & (CODE | BLOCKCODE) != 0 {
        c.effects &= !CFE_AUTOBACKCOLOR;
        c.back_color = col(CODEBG);
    }
    if flags & TABLE != 0 {
        c.text_color = col(TEXT);
    }
    if flags & (DIM | RULE) != 0 {
        c.text_color = col(DIMC);
    }
    if flags & DONE != 0 {
        c.effects |= 0x8;
        c.text_color = col(DIMC);
    }
    if flags & HIDDEN != 0 {
        c.effects |= 0x100;
    }
    c
}

pub fn load_library() -> bool {
    unsafe { LoadLibraryW(w!("Msftedit.dll")).is_ok() }
}

pub fn create_edit(parent: HWND, id: isize) -> Result<HWND, String> {
    // ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN | ES_NOHIDESEL
    let style = WS_CHILD | WS_VISIBLE | WS_VSCROLL | 0x4 | 0x40 | 0x1000 | 0x100;
    let h = create(0, "RICHEDIT50W", "", style, (0, 0, 100, 100), parent, id)?;
    send(h, EM_SETBKGNDCOLOR, 0, col(BG) as isize);
    send(h, EM_SETMARGINS, 3, (8 | (8 << 16)) as isize);
    send(h, EM_SETEVENTMASK, 0, ENM_CHANGE | ENM_SELCHANGE);
    let base = base_format();
    send(h, 0x0444, 4, &base as *const CharFormat2 as isize); // SCF_ALL
    Ok(h)
}

pub fn set_text(h: HWND, s: &str) {
    let norm = s.replace("\r\n", "\r").replace('\n', "\r");
    let w = wide(&norm);
    let st = SetTextEx { flags: 0, codepage: 1200 };
    send(h, EM_SETTEXTEX, &st as *const SetTextEx as usize, w.as_ptr() as isize);
    send(h, EM_EMPTYUNDOBUFFER, 0, 0);
}

/// Editor text as UTF-16 with `\r` line breaks (the unit selection indexes use).
pub fn get_text_u16(h: HWND) -> Vec<u16> {
    let gl = GetTextLengthEx { flags: 8, codepage: 1200 };
    let len = send(h, EM_GETTEXTLENGTHEX, &gl as *const GetTextLengthEx as usize, 0).max(0) as usize;
    if len == 0 {
        return vec![];
    }
    let mut buf = vec![0u16; len + 2];
    let g = GetTextEx { cb: (buf.len() * 2) as u32, flags: 0, codepage: 1200, default_char: std::ptr::null(), used_default: std::ptr::null_mut() };
    let n = send(h, EM_GETTEXTEX, &g as *const GetTextEx as usize, buf.as_mut_ptr() as isize).max(0) as usize;
    buf.truncate(n.min(len + 1));
    buf
}

/// Editor text as a normal string with `\n` line breaks (for saving).
pub fn get_text(h: HWND) -> String {
    String::from_utf16_lossy(&get_text_u16(h)).replace("\r\n", "\n").replace('\r', "\n")
}

pub fn get_sel(h: HWND) -> (usize, usize) {
    let mut r = CharRange::default();
    send(h, EM_GETSEL_EX, 0, &mut r as *mut CharRange as isize);
    (r.min.max(0) as usize, r.max.max(0) as usize)
}

pub fn set_sel(h: HWND, a: usize, b: usize) {
    let r = CharRange { min: a as i32, max: b as i32 };
    send(h, EM_SETSEL_EX, 0, &r as *const CharRange as isize);
}

pub fn replace_sel(h: HWND, s: &str) {
    let w = wide(s);
    send(h, EM_REPLACESEL, 1, w.as_ptr() as isize);
}

pub fn line_of_char(h: HWND, ch: usize) -> usize {
    send(h, EM_LINEFROMCHAR_EX, 0, ch as isize).max(0) as usize
}

pub fn line_start(h: HWND, line: usize) -> usize {
    send(h, EM_LINEINDEX, line, 0).max(0) as usize
}

pub fn line_count(h: HWND) -> usize {
    send(h, EM_GETLINECOUNT, 0, 0).max(1) as usize
}

pub fn paste_text(h: HWND) {
    send(h, EM_PASTESPECIAL, 13, 0); // CF_UNICODETEXT only: never paste foreign formatting
}

/// Re-applies live Markdown formatting. Selection, scroll position and undo are preserved.
pub fn apply_spans(h: HWND, spans: &[Span]) {
    let sel = get_sel(h);
    let first = send(h, EM_GETFIRSTVISIBLELINE, 0, 0);
    let mask = send(h, EM_GETEVENTMASK, 0, 0);
    send(h, EM_SETEVENTMASK, 0, 0);
    send(h, WM_SETREDRAW, 0, 0);
    let base = base_format();
    send(h, EM_SETCHARFORMAT, 4, &base as *const CharFormat2 as isize); // SCF_ALL
    for s in spans {
        set_sel(h, s.start, s.end);
        let f = format_for(s.flags, s.heading);
        send(h, EM_SETCHARFORMAT, SCF_SELECTION, &f as *const CharFormat2 as isize);
    }
    set_sel(h, sel.0, sel.1);
    let now = send(h, EM_GETFIRSTVISIBLELINE, 0, 0);
    send(h, EM_LINESCROLL, 0, first - now);
    send(h, WM_SETREDRAW, 1, 0);
    crate::platform::windows::util::invalidate(h);
    send(h, EM_SETEVENTMASK, 0, mask);
}
