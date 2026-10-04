//! Where the hole goes: pure arithmetic over a measured target, tested here.

use crate::{hooks::Rect, platform::Dimensions};

/// The hole around `target`, `padding` larger on every side, clipped to the viewport.
/// A target off screen leaves a zero-size hole on the nearest edge.
pub(super) fn hole_rect(target: Rect, padding: f64, viewport: Dimensions) -> Rect {
    let left = (target.x - padding).clamp(0.0, viewport.width);
    let top = (target.y - padding).clamp(0.0, viewport.height);
    let right = (target.x + target.width + padding).clamp(0.0, viewport.width);
    let bottom = (target.y + target.height + padding).clamp(0.0, viewport.height);
    Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

/// The highlight's `style`: the hole, or a zero-size box in the middle whose shadow
/// dims everything. Every declaration on every render: a renderer never removes one.
pub(super) fn highlight_style(hole: Option<Rect>, radius: f64) -> String {
    match hole {
        Some(hole) => format!(
            "left:{}px;top:{}px;width:{}px;height:{}px;border-radius:{radius}px;",
            hole.x, hole.y, hole.width, hole.height
        ),
        None => String::from("left:50%;top:50%;width:0px;height:0px;border-radius:0px;"),
    }
}

/// Changes whenever `rect` does, so the card's popover places itself again.
pub(super) fn remeasure_key(rect: Option<Rect>) -> u64 {
    rect.map_or(0, |rect| {
        [rect.x, rect.y, rect.width, rect.height]
            .iter()
            .fold(0u64, |key, value| key.rotate_left(16) ^ value.to_bits())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Dimensions = Dimensions {
        width: 800.0,
        height: 600.0,
    };

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn the_hole_pads_the_target_on_every_side() {
        assert_eq!(
            hole_rect(rect(100.0, 50.0, 80.0, 40.0), 6.0, VIEWPORT),
            rect(94.0, 44.0, 92.0, 52.0)
        );
    }

    #[test]
    fn the_hole_stops_at_the_viewport_edges() {
        assert_eq!(
            hole_rect(rect(2.0, 590.0, 100.0, 40.0), 6.0, VIEWPORT),
            rect(0.0, 584.0, 108.0, 16.0)
        );
        assert_eq!(
            hole_rect(rect(-50.0, -50.0, 900.0, 700.0), 6.0, VIEWPORT),
            rect(0.0, 0.0, 800.0, 600.0)
        );
    }

    #[test]
    fn a_target_off_screen_leaves_an_empty_hole() {
        let hole = hole_rect(rect(100.0, 900.0, 80.0, 40.0), 6.0, VIEWPORT);
        assert_eq!((hole.width, hole.height), (92.0, 0.0));
        assert_eq!(hole.y, 600.0);
    }

    #[test]
    fn the_style_names_the_same_properties_with_or_without_a_hole() {
        let names = |style: String| {
            style
                .split(';')
                .filter_map(|declaration| {
                    declaration
                        .split_once(':')
                        .map(|(name, _)| name.to_string())
                })
                .collect::<Vec<_>>()
        };
        let with = highlight_style(Some(rect(1.0, 2.0, 3.0, 4.0)), 4.0);
        assert_eq!(
            with,
            "left:1px;top:2px;width:3px;height:4px;border-radius:4px;"
        );
        assert_eq!(names(with), names(highlight_style(None, 4.0)));
    }

    #[test]
    fn the_remeasure_key_follows_the_rect() {
        let at = remeasure_key(Some(rect(1.0, 2.0, 3.0, 4.0)));
        assert_eq!(at, remeasure_key(Some(rect(1.0, 2.0, 3.0, 4.0))));
        assert_ne!(at, remeasure_key(Some(rect(1.0, 3.0, 3.0, 4.0))));
        assert_ne!(at, remeasure_key(None));
    }
}
