//! Window-fit geometry (`WIN-11`), pure so it can be tested without a window.

/// Share of the monitor height a window still at its starting size takes.
const FIRST_OPEN_HEIGHT: f64 = 0.85;
/// Share of the monitor any window may take.
const MAXIMUM: f64 = 0.95;
pub const MINIMUM: (f64, f64) = (400.0, 500.0);

/// The page-area size matching `aspect` (width ÷ height), in logical px.
///
/// 1. Keep the current height, unless the window was never sized: then start from
///    85% of the monitor.
/// 2. Shrink to fit 95% of the monitor; `chrome` (the title bar) takes vertical room.
/// 3. Grow to the minimum. Both steps keep the aspect, so the minimum wins on a
///    monitor too small for it.
pub fn content_size(
    aspect: f64,
    current_height: f64,
    never_sized: bool,
    chrome: f64,
    monitor: (f64, f64),
) -> Option<(i32, i32)> {
    if !aspect.is_finite() || aspect <= 0.0 || monitor.0 <= 0.0 || monitor.1 <= 0.0 {
        return None;
    }
    let mut height = if never_sized {
        monitor.1 * FIRST_OPEN_HEIGHT
    } else {
        current_height
    };
    if height <= 0.0 {
        return None;
    }
    let mut width = height * aspect;

    let max_width = monitor.0 * MAXIMUM;
    let max_height = (monitor.1 * MAXIMUM - chrome).max(1.0);
    let shrink = (max_width / width).min(max_height / height).min(1.0);
    width *= shrink;
    height *= shrink;

    let grow = (MINIMUM.0 / width).max(MINIMUM.1 / height).max(1.0);
    width *= grow;
    height *= grow;

    Some((width.round() as i32, height.round() as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (f64, f64) = (1920.0, 1080.0);

    #[test]
    fn keeps_the_height_of_a_sized_window_and_corrects_the_width() {
        assert_eq!(
            content_size(0.5, 800.0, false, 0.0, SCREEN),
            Some((400, 800))
        );
    }

    #[test]
    fn a_never_sized_window_starts_from_most_of_the_monitor() {
        let (_, h) = content_size(0.66, 600.0, true, 0.0, SCREEN).unwrap();
        assert_eq!(h, (1080.0 * 0.85_f64).round() as i32);
    }

    #[test]
    fn matches_the_aspect() {
        for aspect in [0.66, 1.0, 1.5] {
            let (w, h) = content_size(aspect, 700.0, false, 0.0, SCREEN).unwrap();
            assert!((w as f64 / h as f64 - aspect).abs() < 0.005, "{aspect}");
        }
    }

    #[test]
    fn shrinks_to_fit_a_narrow_monitor_keeping_the_aspect() {
        // 1800 × 900 doesn't fit 95% of 1400 px.
        assert_eq!(
            content_size(2.0, 900.0, false, 0.0, (1400.0, 1080.0)),
            Some((1330, 665))
        );
    }

    #[test]
    fn leaves_room_for_the_title_bar() {
        let (_, h) = content_size(0.5, 2000.0, false, 40.0, SCREEN).unwrap();
        assert_eq!(h, (1080.0 * 0.95 - 40.0_f64).round() as i32);
    }

    #[test]
    fn grows_to_the_minimum_keeping_the_aspect() {
        assert_eq!(
            content_size(0.5, 300.0, false, 0.0, SCREEN),
            Some((400, 800))
        );
        assert_eq!(
            content_size(2.0, 100.0, false, 0.0, SCREEN),
            Some((1000, 500))
        );
    }

    #[test]
    fn rejects_nonsense() {
        assert_eq!(content_size(0.0, 800.0, false, 0.0, SCREEN), None);
        assert_eq!(content_size(f64::NAN, 800.0, false, 0.0, SCREEN), None);
        assert_eq!(content_size(f64::INFINITY, 800.0, false, 0.0, SCREEN), None);
        assert_eq!(content_size(1.0, 800.0, false, 0.0, (0.0, 1080.0)), None);
        assert_eq!(content_size(1.0, 0.0, false, 0.0, SCREEN), None);
    }
}
