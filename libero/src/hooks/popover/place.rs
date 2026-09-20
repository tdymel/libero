use crate::platform::Dimensions;

use super::options::{Align, Placement, PopoverOptions, Side};

/// A box in viewport coordinates, which is what `client_offset()` and
/// `dimensions()` answer together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Where the floating box goes, in viewport coordinates, and how it got there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub x: f64,
    pub y: f64,
    pub placement: Placement,
}

/// A physical edge of the anchor: `Side::Start`/`End` resolved against the
/// direction, so the arithmetic below never asks which way the text runs.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

impl Edge {
    fn of(side: Side, rtl: bool) -> Self {
        match (side, rtl) {
            (Side::Top, _) => Edge::Top,
            (Side::Bottom, _) => Edge::Bottom,
            (Side::Start, false) | (Side::End, true) => Edge::Left,
            (Side::End, false) | (Side::Start, true) => Edge::Right,
        }
    }
}

/// The free space between the anchor and one viewport edge, less the padding.
/// Negative when the anchor's own edge is already past it.
fn room(edge: Edge, anchor: Rect, viewport: Dimensions, padding: f64) -> f64 {
    match edge {
        Edge::Top => anchor.y - padding,
        Edge::Bottom => viewport.height - (anchor.y + anchor.height) - padding,
        Edge::Left => anchor.x - padding,
        Edge::Right => viewport.width - (anchor.x + anchor.width) - padding,
    }
}

/// The cross-axis coordinate. `start`, `anchor_size` and `floating_size` are
/// all on that axis, so one function serves both orientations.
fn cross_axis(align: Align, start: f64, anchor_size: f64, floating_size: f64) -> f64 {
    match align {
        Align::Start => start,
        Align::Center => start + (anchor_size - floating_size) / 2.0,
        Align::End => start + anchor_size - floating_size,
    }
}

/// Clamps a coordinate into the viewport, leaving `padding` at both edges. A
/// box too big to fit is pinned to the near edge.
fn shift_into(start: f64, size: f64, viewport_size: f64, padding: f64) -> f64 {
    (start.min(viewport_size - padding - size)).max(padding)
}

/// Places a floating box against its anchor; `rtl` maps logical sides and
/// aligns to physical edges. Pure, so it is testable without a renderer.
pub fn place(
    anchor: Rect,
    floating: Dimensions,
    viewport: Dimensions,
    options: &PopoverOptions,
    rtl: bool,
) -> Placed {
    let needed = |side: Side| match side.is_vertical() {
        true => floating.height + options.gap,
        false => floating.width + options.gap,
    };

    let preferred = room(
        Edge::of(options.side, rtl),
        anchor,
        viewport,
        options.padding,
    );
    let opposite = room(
        Edge::of(options.side.opposite(), rtl),
        anchor,
        viewport,
        options.padding,
    );
    // Flip only when the preferred side cannot hold the box *and* the opposite
    // holds more; too big for both, it stays and `shift` decides.
    let side = match options.flip && preferred < needed(options.side) && opposite > preferred {
        true => options.side.opposite(),
        false => options.side,
    };

    let align = options.align;
    // Along `x` an RTL start is the right edge; along `y` there is no direction.
    let across = match (align, rtl) {
        (Align::Start, true) => Align::End,
        (Align::End, true) => Align::Start,
        _ => align,
    };
    let (mut x, mut y) = match Edge::of(side, rtl) {
        Edge::Top => (
            cross_axis(across, anchor.x, anchor.width, floating.width),
            anchor.y - floating.height - options.gap,
        ),
        Edge::Bottom => (
            cross_axis(across, anchor.x, anchor.width, floating.width),
            anchor.y + anchor.height + options.gap,
        ),
        Edge::Left => (
            anchor.x - floating.width - options.gap,
            cross_axis(align, anchor.y, anchor.height, floating.height),
        ),
        Edge::Right => (
            anchor.x + anchor.width + options.gap,
            cross_axis(align, anchor.y, anchor.height, floating.height),
        ),
    };

    if options.shift {
        match side.is_vertical() {
            true => x = shift_into(x, floating.width, viewport.width, options.padding),
            false => y = shift_into(y, floating.height, viewport.height, options.padding),
        }
    }

    Placed {
        x,
        y,
        placement: Placement { side, align },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::popover::options::PopoverWidth;

    /// A 1000x800 viewport, and an anchor 200 wide and 40 tall wherever the
    /// test puts it.
    fn viewport() -> Dimensions {
        Dimensions {
            width: 1000.0,
            height: 800.0,
        }
    }

    fn anchor_at(x: f64, y: f64) -> Rect {
        Rect {
            x,
            y,
            width: 200.0,
            height: 40.0,
        }
    }

    fn dropdown() -> Dimensions {
        Dimensions {
            width: 200.0,
            height: 260.0,
        }
    }

    fn options() -> PopoverOptions {
        PopoverOptions::new(4.0, 8.0)
    }

    #[test]
    fn sits_under_the_anchor_with_the_gap() {
        let placed = place(
            anchor_at(100.0, 100.0),
            dropdown(),
            viewport(),
            &options(),
            false,
        );

        assert_eq!(placed.x, 100.0, "start-aligned, so the left edges meet");
        assert_eq!(placed.y, 144.0, "anchor bottom plus the 4px gap");
        assert_eq!(placed.placement.side, Side::Bottom);
    }

    #[test]
    fn flips_above_when_the_bottom_cannot_hold_it() {
        // 260 tall under an anchor ending at 640 needs 800; only 152 is left.
        let placed = place(
            anchor_at(100.0, 600.0),
            dropdown(),
            viewport(),
            &options(),
            false,
        );

        assert_eq!(placed.placement.side, Side::Top);
        assert_eq!(placed.y, 336.0, "anchor top, minus the box, minus the gap");
    }

    #[test]
    fn stays_below_when_neither_side_fits_but_below_has_more() {
        let tall = Dimensions {
            width: 200.0,
            height: 700.0,
        };
        let placed = place(anchor_at(100.0, 300.0), tall, viewport(), &options(), false);

        assert_eq!(
            placed.placement.side,
            Side::Bottom,
            "460 below beats 292 above, so flipping would make it worse"
        );
    }

    #[test]
    fn does_not_flip_when_flip_is_off() {
        let placed = place(
            anchor_at(100.0, 600.0),
            dropdown(),
            viewport(),
            &options().flip(false),
            false,
        );

        assert_eq!(placed.placement.side, Side::Bottom);
        assert_eq!(placed.y, 644.0, "off the bottom, which is what was asked");
    }

    #[test]
    fn shifts_a_box_back_off_the_right_edge() {
        // Start-aligned at x 900 would put the right edge at 1100.
        let placed = place(
            anchor_at(900.0, 100.0),
            dropdown(),
            viewport(),
            &options(),
            false,
        );

        assert_eq!(placed.x, 792.0, "1000 minus the 8px padding minus the box");
    }

    #[test]
    fn pins_a_too_wide_box_to_the_near_edge() {
        let wide = Dimensions {
            width: 1200.0,
            height: 100.0,
        };
        let placed = place(anchor_at(100.0, 100.0), wide, viewport(), &options(), false);

        assert_eq!(
            placed.x, 8.0,
            "it cannot fit, so it starts at the padding rather than ending there"
        );
    }

    #[test]
    fn does_not_shift_when_shift_is_off() {
        let placed = place(
            anchor_at(900.0, 100.0),
            dropdown(),
            viewport(),
            &options().shift(false),
            false,
        );

        assert_eq!(placed.x, 900.0);
    }

    #[test]
    fn centre_and_end_line_up_on_the_cross_axis() {
        let narrow = Dimensions {
            width: 100.0,
            height: 60.0,
        };
        let centred = place(
            anchor_at(400.0, 100.0),
            narrow,
            viewport(),
            &options().align(Align::Center),
            false,
        );
        let ended = place(
            anchor_at(400.0, 100.0),
            narrow,
            viewport(),
            &options().align(Align::End),
            false,
        );

        assert_eq!(centred.x, 450.0, "50px of the anchor's 200 on either side");
        assert_eq!(ended.x, 500.0, "right edges meet");
    }

    #[test]
    fn a_side_placement_measures_the_horizontal_axis() {
        let panel = Dimensions {
            width: 300.0,
            height: 100.0,
        };
        let placed = place(
            anchor_at(100.0, 100.0),
            panel,
            viewport(),
            &options().side(Side::Start),
            false,
        );

        assert_eq!(
            placed.placement.side,
            Side::End,
            "only 92 to the left, 700 to the right"
        );
        assert_eq!(placed.x, 304.0, "anchor right plus the gap");
        assert_eq!(placed.y, 100.0, "start-aligned on the vertical axis now");
    }

    /// Todo 711: under RTL `Start` is the right side, and a start-aligned
    /// dropdown lines up right edges.
    #[test]
    fn start_is_the_right_under_rtl() {
        let panel = Dimensions {
            width: 300.0,
            height: 100.0,
        };
        let start = options().side(Side::Start);
        let ltr = place(anchor_at(400.0, 100.0), panel, viewport(), &start, false);
        let rtl = place(anchor_at(400.0, 100.0), panel, viewport(), &start, true);

        assert_eq!(ltr.x, 96.0, "anchor left minus the box and the gap");
        assert_eq!(rtl.x, 604.0, "anchor right plus the gap");
        assert_eq!(rtl.placement.side, Side::Start, "reported logically");

        let narrow = Dimensions {
            width: 100.0,
            height: 60.0,
        };
        let below = place(
            anchor_at(400.0, 100.0),
            narrow,
            viewport(),
            &options(),
            true,
        );
        assert_eq!(below.x, 500.0, "right edges meet");
        assert_eq!(below.placement.align, Align::Start);
    }

    #[test]
    fn an_rtl_start_flips_to_the_end_on_the_left() {
        let panel = Dimensions {
            width: 300.0,
            height: 100.0,
        };
        let placed = place(
            anchor_at(600.0, 100.0),
            panel,
            viewport(),
            &options().side(Side::Start),
            true,
        );

        assert_eq!(
            placed.placement.side,
            Side::End,
            "192 to the right, 592 to the left"
        );
        assert_eq!(placed.x, 296.0, "anchor left minus the box and the gap");
    }

    #[test]
    fn width_is_not_the_placer_s_business() {
        // Placement never reads it, so the same options with a different width
        // land in the same spot; the hook is what turns it into CSS.
        let base = place(
            anchor_at(100.0, 100.0),
            dropdown(),
            viewport(),
            &options(),
            false,
        );
        let matched = place(
            anchor_at(100.0, 100.0),
            dropdown(),
            viewport(),
            &options().width(PopoverWidth::Match),
            false,
        );

        assert_eq!(base, matched);
    }
}
