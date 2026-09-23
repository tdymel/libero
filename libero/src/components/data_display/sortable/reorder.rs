//! A sortable list's geometry, on one axis in flow order: index 0 starts lowest.

/// One item's extent along the list's axis, measured when the drag started.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Span {
    pub(super) start: f64,
    pub(super) size: f64,
}

impl Span {
    pub(super) fn end(self) -> f64 {
        self.start + self.size
    }

    fn middle(self) -> f64 {
        self.start + self.size / 2.0
    }
}

/// Which item moved where, by index.
///
/// ```rust
/// # use libero::hooks::SortableMove;
/// let mut fruit = vec!["apple", "pear", "plum"];
/// SortableMove { from: 0, to: 2 }.apply(&mut fruit);
/// assert_eq!(fruit, ["pear", "plum", "apple"]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortableMove {
    pub from: usize,
    pub to: usize,
}

impl SortableMove {
    /// Moves `items[from]` to `to`, shifting the ones between. `from == to` or
    /// an index out of range does nothing.
    pub fn apply<T>(self, items: &mut Vec<T>) {
        if self.from != self.to && self.from < items.len() && self.to < items.len() {
            let item = items.remove(self.from);
            items.insert(self.to, item);
        }
    }
}

/// How far a neighbour steps aside for item `from`: its size plus the gap beside it.
fn pitch(spans: &[Span], from: usize) -> f64 {
    let own = spans[from];
    let gap = match (spans.get(from + 1), from.checked_sub(1)) {
        (Some(next), _) => next.start - own.end(),
        (None, Some(previous)) => own.start - spans[previous].end(),
        (None, None) => 0.0,
    };
    own.size + gap
}

/// The dragged item's offset, kept between the list's first and last item.
pub(super) fn clamp_offset(spans: &[Span], from: usize, offset: f64) -> f64 {
    let (Some(first), Some(last)) = (spans.first(), spans.last()) else {
        return 0.0;
    };
    let own = spans[from];
    // `max` then `min`, not `clamp`: odd measurements must not panic.
    offset
        .max(first.start - own.start)
        .min(last.end() - own.end())
}

/// Where item `from` lands, dragged by `offset`: past every neighbour whose
/// middle its leading edge crossed. Its middle would never get a tall item past
/// a short last one.
pub(super) fn target_index(spans: &[Span], from: usize, offset: f64) -> usize {
    let own = spans[from];
    let offset = clamp_offset(spans, from, offset);
    let after = spans[from + 1..]
        .iter()
        .take_while(|span| own.end() + offset > span.middle())
        .count();
    if after > 0 {
        return from + after;
    }
    let before = spans[..from]
        .iter()
        .rev()
        .take_while(|span| own.start + offset < span.middle())
        .count();
    from - before
}

/// How far item `index` steps aside while item `from` hovers over slot `to`.
pub(super) fn shift(spans: &[Span], from: usize, to: usize, index: usize) -> f64 {
    if from < to && (from + 1..=to).contains(&index) {
        -pitch(spans, from)
    } else if to < from && (to..from).contains(&index) {
        pitch(spans, from)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three 40px items with 10px gaps: 0-40, 50-90, 100-140.
    fn even() -> Vec<Span> {
        (0..3)
            .map(|i| Span {
                start: i as f64 * 50.0,
                size: 40.0,
            })
            .collect()
    }

    #[test]
    fn a_small_move_keeps_the_item_in_place() {
        assert_eq!(target_index(&even(), 0, 20.0), 0);
        assert_eq!(target_index(&even(), 2, -20.0), 2);
    }

    #[test]
    fn crossing_a_neighbours_middle_takes_its_slot() {
        // Item 0's bottom (40) passes item 1's middle (70) beyond 30.
        assert_eq!(target_index(&even(), 0, 29.0), 0);
        assert_eq!(target_index(&even(), 0, 31.0), 1);
        assert_eq!(target_index(&even(), 0, 81.0), 2);
        assert_eq!(target_index(&even(), 2, -29.0), 2);
        assert_eq!(target_index(&even(), 2, -31.0), 1);
        assert_eq!(target_index(&even(), 2, -81.0), 0);
    }

    #[test]
    fn a_drag_past_the_ends_stops_at_the_first_or_last_slot() {
        assert_eq!(target_index(&even(), 1, 1000.0), 2);
        assert_eq!(target_index(&even(), 1, -1000.0), 0);
        assert_eq!(clamp_offset(&even(), 1, 1000.0), 50.0);
        assert_eq!(clamp_offset(&even(), 1, -1000.0), -50.0);
    }

    #[test]
    fn neighbours_between_step_aside_by_the_items_size_and_gap() {
        let spans = even();
        assert_eq!(shift(&spans, 0, 2, 1), -50.0);
        assert_eq!(shift(&spans, 0, 2, 2), -50.0);
        assert_eq!(shift(&spans, 2, 0, 0), 50.0);
        assert_eq!(shift(&spans, 2, 1, 0), 0.0);
        assert_eq!(shift(&spans, 0, 1, 2), 0.0);
        assert_eq!(shift(&spans, 1, 1, 0), 0.0);
    }

    #[test]
    fn a_tall_item_moves_its_neighbours_by_its_own_height() {
        let spans = [
            Span {
                start: 0.0,
                size: 100.0,
            },
            Span {
                start: 108.0,
                size: 20.0,
            },
            Span {
                start: 136.0,
                size: 20.0,
            },
        ];
        assert_eq!(shift(&spans, 0, 1, 1), -108.0);
        // The last item has no gap after it; it uses the one before.
        assert_eq!(shift(&spans, 2, 0, 0), 28.0);
        // The tall item's bottom (100) passes the short ones' middles (118, 146).
        assert_eq!(target_index(&spans, 0, 17.0), 0);
        assert_eq!(target_index(&spans, 0, 19.0), 1);
        assert_eq!(target_index(&spans, 0, 1000.0), 2);
    }

    #[test]
    fn a_move_reorders_a_vec_and_ignores_an_index_out_of_range() {
        let mut items = vec!['a', 'b', 'c', 'd'];
        SortableMove { from: 3, to: 1 }.apply(&mut items);
        assert_eq!(items, ['a', 'd', 'b', 'c']);
        SortableMove { from: 4, to: 0 }.apply(&mut items);
        SortableMove { from: 0, to: 4 }.apply(&mut items);
        SortableMove { from: 2, to: 2 }.apply(&mut items);
        assert_eq!(items, ['a', 'd', 'b', 'c']);
    }
}
