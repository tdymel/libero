//! Auto-scroll for a drag held near a scrolling box's side edge.

/// How far from a scrolling box's side edge a drag starts scrolling it, in px.
const EDGE: f64 = 48.0;

/// The most a box scrolls per auto-scroll tick, at its very edge.
const EDGE_STEP: f64 = 16.0;

/// The px a drag at client `x` scrolls a box spanning `start..start + width`
/// by per tick: negative near its left edge, positive near its right, faster
/// the closer, 0 elsewhere.
pub(crate) fn edge_scroll_step(x: f64, start: f64, width: f64) -> f64 {
    let depth = |distance: f64| ((EDGE - distance) / EDGE).clamp(0.0, 1.0);
    let (left, right) = (depth(x - start), depth(start + width - x));
    // At least a px, so a pointer just inside the zone still moves.
    let step = |depth: f64| (EDGE_STEP * depth).max(1.0);
    match (left > 0.0, right > 0.0) {
        (true, _) => -step(left),
        (_, true) => step(right),
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_near_a_side_edge_scrolls_that_way_faster_the_closer() {
        let step = |x: f64| edge_scroll_step(x, 100.0, 600.0);
        assert_eq!(step(400.0), 0.0);
        assert_eq!(step(148.0), 0.0);
        assert_eq!(step(100.0), -EDGE_STEP);
        assert_eq!(step(124.0), -EDGE_STEP / 2.0);
        assert_eq!(step(700.0), EDGE_STEP);
        // Past the edge, the pointer off the box, still the full step.
        assert_eq!(step(760.0), EDGE_STEP);
        assert_eq!(step(147.9), -1.0);
    }
}
