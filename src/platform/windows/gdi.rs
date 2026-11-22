//! Thin GDI drawing helpers (double buffering, fonts, text, rectangles).
use super::util::wide;
use ::windows::core::PCWSTR;
use ::windows::Win32::Foundation::{COLORREF, HWND, RECT, SIZE};
use ::windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, DrawTextW, FillRect, GetDC, GetTextExtentPoint32W, ReleaseDC, SelectObject,
    SetBkMode, SetTextColor, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DRAW_TEXT_FORMAT,
    HBITMAP, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, SRCCOPY, TRANSPARENT,
};

pub fn make_font(face: &str, px: i32, bold: bool, italic: bool) -> HFONT {
    let f = wide(face);
    unsafe {
        CreateFontW(
            -px,
            0,
            0,
            0,
            if bold { 700 } else { 400 },
            italic as u32,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            0,
            PCWSTR(f.as_ptr()),
        )
    }
}

pub fn delete_font(f: HFONT) {
    unsafe {
        let _ = DeleteObject(f);
    }
}

pub fn fill(hdc: HDC, r: &RECT, c: COLORREF) {
    unsafe {
        let b = CreateSolidBrush(c);
        FillRect(hdc, r, b);
        let _ = DeleteObject(b);
    }
}

pub fn rect(l: i32, t: i32, r: i32, b: i32) -> RECT {
    RECT { left: l, top: t, right: r, bottom: b }
}

pub fn draw_text(hdc: HDC, font: HFONT, s: &str, r: &RECT, color: COLORREF, flags: u32) {
    unsafe {
        SelectObject(hdc, font);
        SetTextColor(hdc, color);
        SetBkMode(hdc, TRANSPARENT);
        let mut w: Vec<u16> = s.encode_utf16().collect();
        let mut rr = *r;
        DrawTextW(hdc, &mut w, &mut rr, DRAW_TEXT_FORMAT(flags));
    }
}

pub fn text_width(hdc: HDC, font: HFONT, s: &str) -> i32 {
    unsafe {
        SelectObject(hdc, font);
        let w: Vec<u16> = s.encode_utf16().collect();
        let mut sz = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, &w, &mut sz);
        sz.cx
    }
}

// DrawText flag values
pub const DT_LEFT: u32 = 0x0000;
pub const DT_CENTER: u32 = 0x0001;
pub const DT_RIGHT: u32 = 0x0002;
pub const DT_VCENTER: u32 = 0x0004;
pub const DT_SINGLELINE: u32 = 0x0020;
pub const DT_NOPREFIX: u32 = 0x0800;
pub const DT_END_ELLIPSIS: u32 = 0x8000;

/// Off-screen buffer to avoid flicker.
pub struct Buffer {
    pub dc: HDC,
    bmp: HBITMAP,
    old: HGDIOBJ,
    w: i32,
    h: i32,
}

impl Buffer {
    pub fn new(target: HDC, w: i32, h: i32) -> Buffer {
        unsafe {
            let dc = CreateCompatibleDC(target);
            let bmp = CreateCompatibleBitmap(target, w.max(1), h.max(1));
            let old = SelectObject(dc, bmp);
            Buffer { dc, bmp, old, w, h }
        }
    }

    pub fn present(&self, target: HDC) {
        unsafe {
            let _ = BitBlt(target, 0, 0, self.w, self.h, self.dc, 0, 0, SRCCOPY);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(self.bmp);
            let _ = DeleteDC(self.dc);
        }
    }
}

/// Text width using a temporary DC of `hwnd` (for layout outside WM_PAINT).
pub fn measure_in_window(hwnd: HWND, font: HFONT, s: &str) -> i32 {
    unsafe {
        let dc = GetDC(hwnd);
        let w = text_width(dc, font, s);
        ReleaseDC(hwnd, dc);
        w
    }
}
