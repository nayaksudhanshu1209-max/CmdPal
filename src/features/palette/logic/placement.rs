//! Pure placement maths: keep the palette fully inside a monitor work area (CORE-04, CORE-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

fn clamp_into(work: Rect, left: i32, top: i32, w: i32, h: i32) -> Rect {
    let ww = (work.right - work.left).max(1);
    let wh = (work.bottom - work.top).max(1);
    let (w, h) = (w.min(ww), h.min(wh));
    let left = left.clamp(work.left, work.right - w);
    let top = top.clamp(work.top, work.bottom - h);
    Rect { left, top, right: left + w, bottom: top + h }
}

/// Horizontally centred, in the upper fifth of `work`; `h` is the FULL (maximum) height so the
/// top edge stays fixed while the result list grows.
pub fn centered_upper(work: Rect, width: i32, height: i32) -> Rect {
    let ww = (work.right - work.left).max(1);
    let wh = (work.bottom - work.top).max(1);
    let (w, h) = (width.min(ww), height.min(wh));
    clamp_into(work, work.left + (ww - w) / 2, work.top + (wh - h) / 5, w, h)
}

/// Top-left near the cursor, shifted so the whole palette stays on screen.
pub fn near_cursor(work: Rect, cursor: (i32, i32), width: i32, height: i32) -> Rect {
    clamp_into(work, cursor.0 - 24, cursor.1 - 16, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(inner: Rect, outer: Rect) -> bool {
        inner.left >= outer.left && inner.top >= outer.top && inner.right <= outer.right && inner.bottom <= outer.bottom
    }

    const A: Rect = Rect { left: 0, top: 0, right: 1920, bottom: 1040 };
    const B: Rect = Rect { left: -1280, top: 100, right: 0, bottom: 1000 };

    #[test]
    fn centered_stays_inside_normal_and_offset_monitors() {
        for w in [A, B] {
            assert!(inside(centered_upper(w, 680, 480), w));
        }
        assert_eq!(centered_upper(A, 680, 480).left, 620);
    }

    #[test]
    fn near_cursor_is_clamped_at_every_edge() {
        for c in [(0, 0), (1919, 1039), (1900, 10), (10, 1030), (-5000, 5000)] {
            assert!(inside(near_cursor(A, c, 680, 480), A), "{c:?}");
        }
        assert!(inside(near_cursor(B, (-10, 990), 680, 480), B));
    }

    #[test]
    fn clamps_when_work_area_is_tiny() {
        let work = Rect { left: 0, top: 0, right: 300, bottom: 40 };
        assert!(inside(centered_upper(work, 640, 72), work));
        assert!(inside(near_cursor(work, (200, 30), 640, 72), work));
    }
}
