//! Pure selection/scroll logic over result rows (Up/Down/Tab/PageUp/PageDown).
use crate::features::suggestions::api::Row;

pub fn is_item(rows: &[Row], i: usize) -> bool {
    matches!(rows.get(i), Some(Row::Item(_)))
}

pub fn first_item(rows: &[Row]) -> Option<usize> {
    (0..rows.len()).find(|&i| is_item(rows, i))
}

fn items(rows: &[Row]) -> Vec<usize> {
    (0..rows.len()).filter(|&i| is_item(rows, i)).collect()
}

/// Moves `delta` items; `wrap` makes Up/Down/Tab cycle, paging clamps.
pub fn step(rows: &[Row], sel: Option<usize>, delta: isize, wrap: bool) -> Option<usize> {
    let it = items(rows);
    if it.is_empty() {
        return None;
    }
    let cur = sel.and_then(|s| it.iter().position(|&i| i == s));
    let n = it.len() as isize;
    let next = match cur {
        None => if delta >= 0 { 0 } else { n - 1 },
        Some(c) => {
            let t = c as isize + delta;
            if wrap { t.rem_euclid(n) } else { t.clamp(0, n - 1) }
        }
    };
    Some(it[next as usize])
}

pub fn row_height(rows: &[Row], i: usize, item_h: i32, header_h: i32) -> i32 {
    if matches!(rows.get(i), Some(Row::Header(_))) { header_h } else { item_h }
}

/// Number of rows (from `scroll`) that fit in `max_px`.
pub fn visible_count(rows: &[Row], scroll: usize, max_px: i32, item_h: i32, header_h: i32) -> usize {
    let (mut used, mut n) = (0, 0);
    for i in scroll..rows.len() {
        let h = row_height(rows, i, item_h, header_h);
        if used + h > max_px && n > 0 {
            break;
        }
        used += h;
        n += 1;
    }
    n
}

/// Adjusts `scroll` so `sel` (and its header, if directly above) is visible.
pub fn ensure_visible(rows: &[Row], sel: usize, mut scroll: usize, max_px: i32, item_h: i32, header_h: i32) -> usize {
    if sel < scroll {
        scroll = sel;
    }
    if sel > 0 && matches!(rows.get(sel - 1), Some(Row::Header(_))) && sel - 1 < scroll {
        scroll = sel - 1;
    }
    while scroll < sel && scroll + visible_count(rows, scroll, max_px, item_h, header_h) <= sel {
        scroll += 1;
    }
    scroll.min(rows.len().saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::contracts::{Action, Category, ResultItem};

    fn rows(n: usize) -> Vec<Row> {
        let mut v = vec![Row::Header(Category::Files)];
        for i in 0..n {
            v.push(Row::Item(ResultItem::new(format!("k{i}"), "t", Category::Files, 1, Action::OpenPath("x".into()))));
        }
        v
    }

    #[test]
    fn steps_skip_headers_and_wrap() {
        let r = rows(3);
        assert_eq!(first_item(&r), Some(1));
        assert_eq!(step(&r, Some(1), 1, true), Some(2));
        assert_eq!(step(&r, Some(3), 1, true), Some(1));
        assert_eq!(step(&r, Some(1), -1, true), Some(3));
        assert_eq!(step(&r, Some(1), 5, false), Some(3));
        assert_eq!(step(&r, None, -1, true), Some(3));
        assert_eq!(step(&[], None, 1, true), None);
    }

    #[test]
    fn scroll_keeps_selection_visible() {
        let r = rows(20);
        let s = ensure_visible(&r, 15, 0, 46 * 4 + 26, 46, 26);
        assert!(s > 0 && s <= 15);
        assert!(s + visible_count(&r, s, 46 * 4 + 26, 46, 26) > 15);
        assert_eq!(ensure_visible(&r, 1, 10, 200, 46, 26), 0); // header above selection included
    }
}
