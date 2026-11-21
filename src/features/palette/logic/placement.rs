//! Pure placement maths: keep the palette fully inside a monitor work area (CORE-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Horizontally centred, in the upper fifth of `work`; size is clamped to fit.
pub fn centered_upper(work: Rect, width: i32, height: i32) -> Rect {
    let ww = (work.right - work.left).max(1);
    let wh = (work.bottom - work.top).max(1);
    let (w, h) = (width.min(ww), height.min(wh));
    let left = work.left + (ww - w) / 2;
    let top = work.top + (wh - h) / 5;
    Rect {
        left,
        top,
        right: left + w,
        bottom: top + h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(inner: Rect, outer: Rect) -> bool {
        inner.left >= outer.left
            && inner.top >= outer.top
            && inner.right <= outer.right
            && inner.bottom <= outer.bottom
    }

    #[test]
    fn stays_inside_normal_and_offset_monitors() {
        for work in [
            Rect {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1040,
            },
            Rect {
                left: -1280,
                top: 100,
                right: 0,
                bottom: 1000,
            },
        ] {
            assert!(inside(centered_upper(work, 640, 72), work));
        }
    }

    #[test]
    fn clamps_when_work_area_is_tiny() {
        let work = Rect {
            left: 0,
            top: 0,
            right: 300,
            bottom: 40,
        };
        assert!(inside(centered_upper(work, 640, 72), work));
    }
}
