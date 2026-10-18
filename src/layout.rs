//! Where pages go in the page view (`WIN-6`).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Pages scaled to one height, side by side and touching, the row fitted into
/// `area` keeping its aspect and centred.
pub fn place(area: (f64, f64), pages: &[(f64, f64)]) -> Vec<Rect> {
    let (aw, ah) = area;
    if aw <= 0.0 || ah <= 0.0 || pages.iter().any(|&(w, h)| w <= 0.0 || h <= 0.0) {
        return Vec::new();
    }
    let aspect: f64 = pages.iter().map(|&(w, h)| w / h).sum();
    let height = ah.min(aw / aspect);
    let mut x = (aw - height * aspect) / 2.0;
    let y = (ah - height) / 2.0;
    pages
        .iter()
        .map(|&(w, h)| {
            let width = height * w / h;
            let rect = Rect {
                x,
                y,
                width,
                height,
            };
            x += width;
            rect
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn one_page_fits_and_centres() {
        assert_eq!(
            place((1000.0, 1000.0), &[(500.0, 1000.0)]),
            [r(250.0, 0.0, 500.0, 1000.0)]
        );
        assert_eq!(
            place((1000.0, 1000.0), &[(2000.0, 1000.0)]),
            [r(0.0, 250.0, 1000.0, 500.0)]
        );
        // Small pages scale up.
        assert_eq!(
            place((400.0, 400.0), &[(10.0, 10.0)]),
            [r(0.0, 0.0, 400.0, 400.0)]
        );
    }

    #[test]
    fn two_pages_share_a_height_and_meet_in_the_middle() {
        let rects = place((1200.0, 1000.0), &[(500.0, 1000.0), (1000.0, 1000.0)]);
        // Combined aspect 1.5 -> 1200 × 800.
        assert_eq!(
            rects,
            [r(0.0, 100.0, 400.0, 800.0), r(400.0, 100.0, 800.0, 800.0)]
        );
        let rects = place((2000.0, 1000.0), &[(500.0, 1000.0), (500.0, 1000.0)]);
        assert_eq!(rects[0].x + rects[0].width, 1000.0);
        assert_eq!(rects[1].x, 1000.0);
    }

    #[test]
    fn degenerate_inputs_place_nothing() {
        assert!(place((0.0, 100.0), &[(1.0, 1.0)]).is_empty());
        assert!(place((100.0, 100.0), &[(0.0, 1.0)]).is_empty());
        assert!(place((100.0, 100.0), &[]).is_empty());
    }
}
