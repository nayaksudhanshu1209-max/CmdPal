//! Pure Markdown analysis for the live editor. Input is the editor text as UTF-16 (the unit
//! the RichEdit control indexes by); output is a list of non-overlapping styled spans.
//! Delimiters are `HIDDEN` unless the selection touches their construct (then `DIM`).

pub const BOLD: u32 = 1 << 0;
pub const ITALIC: u32 = 1 << 1;
pub const STRIKE: u32 = 1 << 2;
pub const CODE: u32 = 1 << 3;
pub const DIM: u32 = 1 << 4;
pub const HIDDEN: u32 = 1 << 5;
pub const LINK: u32 = 1 << 6;
pub const QUOTE: u32 = 1 << 7;
pub const RULE: u32 = 1 << 8;
pub const DONE: u32 = 1 << 9;
pub const TABLE: u32 = 1 << 10;
pub const BLOCKCODE: u32 = 1 << 11;
pub const IMAGE: u32 = 1 << 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub flags: u32,
    pub heading: u8,
}

type Sel = (usize, usize);

fn eq(t: &[u16], i: usize, s: &str) -> bool {
    let b = s.as_bytes();
    i + b.len() <= t.len() && b.iter().enumerate().all(|(k, &c)| t[i + k] == c as u16)
}

fn is_space(c: u16) -> bool {
    c == b' ' as u16 || c == b'\t' as u16
}

fn is_digit(c: u16) -> bool {
    (b'0' as u16..=b'9' as u16).contains(&c)
}

fn touches(sel: Sel, start: usize, end: usize) -> bool {
    sel.0 <= end && sel.1 >= start
}

fn push(out: &mut Vec<Span>, start: usize, end: usize, flags: u32, heading: u8) {
    if end > start && (flags != 0 || heading != 0) {
        out.push(Span { start, end, flags, heading });
    }
}

fn delim(out: &mut Vec<Span>, s: usize, e: usize, show: bool, heading: u8) {
    push(out, s, e, if show { DIM } else { HIDDEN }, heading);
}

pub fn analyze(t: &[u16], sel: Sel) -> Vec<Span> {
    let mut out = vec![];
    let mut in_fence = false;
    let mut start = 0;
    while start <= t.len() {
        let mut end = start;
        while end < t.len() && t[end] != 0x0D && t[end] != 0x0A {
            end += 1;
        }
        line(t, start, end, sel, &mut in_fence, &mut out);
        if end >= t.len() {
            break;
        }
        start = end + 1;
    }
    out
}

fn line(t: &[u16], start: usize, end: usize, sel: Sel, in_fence: &mut bool, out: &mut Vec<Span>) {
    if start == end {
        return;
    }
    let caret_on_line = touches(sel, start, end);
    let mut ws = start;
    while ws < end && is_space(t[ws]) {
        ws += 1;
    }
    if ws >= end {
        return;
    }
    if eq(t, ws, "```") || eq(t, ws, "~~~") {
        *in_fence = !*in_fence;
        push(out, start, end, BLOCKCODE | DIM, 0);
        return;
    }
    if *in_fence {
        push(out, start, end, BLOCKCODE, 0);
        return;
    }
    if t[ws] == b'|' as u16 {
        push(out, start, end, TABLE, 0);
        return;
    }
    let trimmed: Vec<u16> = t[start..end].iter().copied().filter(|&c| !is_space(c)).collect();
    if trimmed.len() >= 3 && [b'-', b'*', b'_'].iter().any(|&m| trimmed.iter().all(|&c| c == m as u16)) {
        push(out, start, end, RULE, 0);
        return;
    }
    let mut pos = start;
    let mut flags = 0u32;
    let mut heading = 0u8;
    let mut n = 0;
    while ws + n < end && t[ws + n] == b'#' as u16 && n < 6 {
        n += 1;
    }
    if n >= 1 && ws + n < end && t[ws + n] == b' ' as u16 {
        let m_end = ws + n + 1;
        delim(out, ws, m_end, caret_on_line, n as u8);
        pos = m_end;
        heading = n as u8;
        flags |= BOLD;
    } else if t[ws] == b'>' as u16 {
        let m_end = ws + 1 + usize::from(ws + 1 < end && t[ws + 1] == b' ' as u16);
        delim(out, ws, m_end, caret_on_line, 0);
        pos = m_end;
        flags |= QUOTE;
    } else {
        let mut m_end = None;
        if (eq(t, ws, "- ") || eq(t, ws, "* ") || eq(t, ws, "+ ")) && ws + 2 <= end {
            m_end = Some(ws + 2);
        } else {
            let mut d = ws;
            while d < end && is_digit(t[d]) {
                d += 1;
            }
            if d > ws && eq(t, d, ". ") {
                m_end = Some(d + 2);
            }
        }
        if let Some(mut me) = m_end {
            if eq(t, me, "[ ] ") {
                push(out, ws, me + 4, DIM, 0);
                me += 4;
            } else if eq(t, me, "[x] ") || eq(t, me, "[X] ") {
                push(out, ws, me + 4, DIM, 0);
                me += 4;
                flags |= DONE;
            } else {
                push(out, ws, me, DIM, 0);
            }
            pos = me.min(end);
        }
    }
    inline(t, pos, end, flags, heading, sel, out);
}

fn find_close(t: &[u16], from: usize, to: usize, d: &str) -> Option<usize> {
    let dl = d.len();
    let mut j = from + 1; // content must be non-empty
    while j + dl <= to {
        if t[j] == b'\\' as u16 {
            j += 2;
            continue;
        }
        if eq(t, j, d) && !is_space(t[j - 1]) {
            if dl == 1 && j + 1 < to && t[j + 1] == t[j] {
                j += 2;
                continue;
            }
            return Some(j);
        }
        j += 1;
    }
    None
}

fn inline(t: &[u16], from: usize, to: usize, flags: u32, heading: u8, sel: Sel, out: &mut Vec<Span>) {
    let mut i = from;
    let mut seg = from;
    while i < to {
        if t[i] == b'\\' as u16 {
            i += 2;
            continue;
        }
        if t[i] == b'`' as u16 {
            if let Some(j) = (i + 2..to).find(|&j| t[j] == b'`' as u16) {
                let show = touches(sel, i, j + 1);
                push(out, seg, i, flags, heading);
                delim(out, i, i + 1, show, heading);
                push(out, i + 1, j, flags | CODE, heading);
                delim(out, j, j + 1, show, heading);
                i = j + 1;
                seg = i;
                continue;
            }
        }
        let img = t[i] == b'!' as u16 && i + 1 < to && t[i + 1] == b'[' as u16;
        if img || t[i] == b'[' as u16 {
            let ls = i + if img { 2 } else { 1 };
            if let Some(cb) = (ls..to).find(|&j| t[j] == b']' as u16) {
                if cb + 1 < to && t[cb + 1] == b'(' as u16 {
                    if let Some(cp) = (cb + 2..to).find(|&j| t[j] == b')' as u16) {
                        let show = touches(sel, i, cp + 1);
                        push(out, seg, i, flags, heading);
                        delim(out, i, ls, show, heading);
                        push(out, ls, cb, flags | LINK | if img { IMAGE } else { 0 }, heading);
                        delim(out, cb, cp + 1, show, heading);
                        i = cp + 1;
                        seg = i;
                        continue;
                    }
                }
            }
        }
        let (d, f): (&str, u32) = if eq(t, i, "**") {
            ("**", BOLD)
        } else if eq(t, i, "__") {
            ("__", BOLD)
        } else if eq(t, i, "~~") {
            ("~~", STRIKE)
        } else if t[i] == b'*' as u16 {
            ("*", ITALIC)
        } else if t[i] == b'_' as u16 && (i == from || !(t[i - 1] < 128 && (t[i - 1] as u8).is_ascii_alphanumeric())) {
            ("_", ITALIC)
        } else {
            i += 1;
            continue;
        };
        let dl = d.len();
        let opens = i + dl < to && !is_space(t[i + dl]);
        if opens {
            if let Some(j) = find_close(t, i + dl, to, d) {
                let show = touches(sel, i, j + dl);
                push(out, seg, i, flags, heading);
                delim(out, i, i + dl, show, heading);
                inline(t, i + dl, j, flags | f, heading, sel, out);
                delim(out, j, j + dl, show, heading);
                i = j + dl;
                seg = i;
                continue;
            }
        }
        i += dl.max(1);
    }
    push(out, seg, to, flags, heading);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }
    fn at(s: &str, c: usize) -> Vec<(usize, usize, u32)> {
        analyze(&u(s), (c, c)).iter().map(|s| (s.start, s.end, s.flags)).collect()
    }

    #[test]
    fn bold_delimiters_hidden_then_dim_when_caret_inside() {
        let s = "a **bold** b";
        assert_eq!(at(s, 100), vec![(2, 4, HIDDEN), (4, 8, BOLD), (8, 10, HIDDEN)]);
        assert_eq!(at(s, 5), vec![(2, 4, DIM), (4, 8, BOLD), (8, 10, DIM)]);
        assert_eq!(at(s, 10), vec![(2, 4, DIM), (4, 8, BOLD), (8, 10, DIM)]);
        assert_eq!(at(s, 11), vec![(2, 4, HIDDEN), (4, 8, BOLD), (8, 10, HIDDEN)]);
    }

    #[test]
    fn headings_quotes_tasks_lists() {
        let h = analyze(&u("## Title\rtext"), (50, 50));
        assert_eq!(h[0], Span { start: 0, end: 3, flags: HIDDEN, heading: 2 });
        assert_eq!(h[1], Span { start: 3, end: 8, flags: BOLD, heading: 2 });
        assert_eq!(at("> quoted", 100), vec![(0, 2, HIDDEN), (2, 8, QUOTE)]);
        assert_eq!(at("- [x] done", 100), vec![(0, 6, DIM), (6, 10, DONE)]);
        assert_eq!(at("1. item", 100), vec![(0, 3, DIM)]);
    }

    #[test]
    fn fences_tables_rules() {
        let sp = at("```\rcode\r```\rafter", 100);
        assert_eq!(sp, vec![(0, 3, BLOCKCODE | DIM), (4, 8, BLOCKCODE), (9, 12, BLOCKCODE | DIM)]);
        assert_eq!(at("| a | b |", 100), vec![(0, 9, TABLE)]);
        assert_eq!(at("---", 100), vec![(0, 3, RULE)]);
    }

    #[test]
    fn links_images_code_strike() {
        assert_eq!(at("[hi](http://x.y)", 100), vec![(0, 1, HIDDEN), (1, 3, LINK), (3, 16, HIDDEN)]);
        assert_eq!(at("![alt](p.png)", 100)[1].2, LINK | IMAGE);
        assert_eq!(at("`x`", 100), vec![(0, 1, HIDDEN), (1, 2, CODE), (2, 3, HIDDEN)]);
        assert_eq!(at("~~gone~~", 100)[1].2, STRIKE);
    }

    #[test]
    fn nested_emphasis_and_unmatched() {
        let sp = at("**a *b* c**", 100);
        assert!(sp.iter().any(|s| s.2 == BOLD | ITALIC));
        assert!(at("**unclosed", 100).is_empty());
        assert!(at("2 * 3 * 4", 100).is_empty());
        assert!(at("snake_case_name", 100).is_empty());
    }
}
