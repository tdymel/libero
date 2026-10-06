//! A board's geometry while a card drags by pointer: every column's list and
//! cards, in client px, measured when the drag started.

use crate::components::data_display::sortable::{Span, shift, slot_offset, target_index};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Rect {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
}

impl Rect {
    fn bottom(self) -> f64 {
        self.y + self.height
    }

    fn middle(self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// One column: its card list and its mounted cards in order, with each one's `index`.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Lane {
    pub(super) list: Rect,
    pub(super) cards: Vec<Rect>,
    pub(super) indices: Vec<usize>,
}

impl Lane {
    fn spans(&self) -> Vec<Span> {
        self.cards
            .iter()
            .map(|card| Span {
                start: card.y,
                size: card.height,
            })
            .collect()
    }
}

/// The board with card `from` of column `from_column` lifted. Slots count the
/// mounted cards; a filtered column's `index`es may skip some.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Lanes {
    pub(super) lanes: Vec<Lane>,
    pub(super) from_column: usize,
    pub(super) from: usize,
}

impl Lanes {
    fn own(&self) -> Rect {
        self.lanes[self.from_column].cards[self.from]
    }

    /// The slot of the card with `index` in `column`, `None` when it was not measured.
    pub(super) fn slot(&self, column: usize, index: usize) -> Option<usize> {
        let lane = self.lanes.get(column)?;
        lane.indices.iter().position(|&at| at == index)
    }

    /// The lifted card's `index`.
    pub(super) fn lifted_index(&self) -> usize {
        self.lanes[self.from_column].indices[self.from]
    }

    /// The `index` a card dropped in slot `target` takes: just before the next shown card,
    /// else just after the last one. In its own column, counted after its removal.
    pub(super) fn index_at(&self, target: (usize, usize)) -> usize {
        let (column, to) = target;
        let own = column == self.from_column;
        let from = self.lifted_index();
        let others: Vec<usize> = self.lanes[column]
            .indices
            .iter()
            .copied()
            .filter(|&index| !own || index != from)
            .collect();
        let shifted = |index: usize| index - usize::from(own && index > from);
        match (others.get(to), others.last()) {
            (Some(&next), _) => shifted(next),
            (None, Some(&last)) => shifted(last) + 1,
            (None, None) if own => from,
            (None, None) => 0,
        }
    }

    /// The space between two cards, from the first column holding two.
    fn gap(&self) -> f64 {
        self.lanes
            .iter()
            .find_map(|lane| match lane.cards[..] {
                [first, second, ..] => Some((second.y - first.bottom()).max(0.0)),
                _ => None,
            })
            .unwrap_or(0.0)
    }

    /// How far the cards step aside for the dragged one.
    fn pitch(&self) -> f64 {
        self.own().height + self.gap()
    }

    /// The column under the dragged card's centre, the nearest across a gap
    /// between two; `None` beside the lists, or over a card's height above or below them.
    fn column_at(&self, dx: f64, dy: f64) -> Option<usize> {
        let (x, y) = self.own().middle();
        let (x, y) = (x + dx, y + dy);
        let lists = self.lanes.iter().map(|lane| lane.list);
        let left = lists
            .clone()
            .map(|list| list.x)
            .fold(f64::INFINITY, f64::min);
        let right = lists
            .clone()
            .map(|list| list.x + list.width)
            .fold(f64::NEG_INFINITY, f64::max);
        // A card's height of slack above and below: over the header still lands first.
        let top = lists
            .clone()
            .map(|list| list.y)
            .fold(f64::INFINITY, f64::min)
            - self.pitch();
        let bottom = lists.map(Rect::bottom).fold(f64::NEG_INFINITY, f64::max) + self.pitch();
        if !(left..=right).contains(&x) || !(top..=bottom).contains(&y) {
            return None;
        }
        let distance = |lane: &Lane| {
            let (start, end) = (lane.list.x, lane.list.x + lane.list.width);
            (start - x).max(x - end).max(0.0)
        };
        self.lanes
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| distance(a).total_cmp(&distance(b)))
            .map(|(at, _)| at)
    }

    /// Where the card lands dragged by `(dx, dy)`: a column and a slot in it, `None` off the board.
    pub(super) fn target(&self, dx: f64, dy: f64) -> Option<(usize, usize)> {
        let column = self.column_at(dx, dy)?;
        let lane = &self.lanes[column];
        if column == self.from_column {
            return Some((column, target_index(&lane.spans(), self.from, dy)));
        }
        let y = self.own().middle().1 + dy;
        let before = lane.cards.iter().take_while(|card| card.middle().1 < y);
        Some((column, before.count()))
    }

    /// How far card `index` of `column` steps down while the lifted one hovers over `target`.
    pub(super) fn step(&self, column: usize, index: usize, target: (usize, usize)) -> f64 {
        let (to_column, to) = target;
        if column == self.from_column && to_column == self.from_column {
            shift(&self.lanes[column].spans(), self.from, to, index)
        } else if column == self.from_column && index > self.from {
            -self.pitch()
        } else if column == to_column && column != self.from_column && index >= to {
            self.pitch()
        } else {
            0.0
        }
    }

    /// The room a column grows by below its cards: the lifted card's, while it hovers there from another.
    pub(super) fn room(&self, column: usize, target: (usize, usize)) -> f64 {
        if column == target.0 && column != self.from_column {
            self.pitch()
        } else {
            0.0
        }
    }

    /// How far the lifted card travels from its start to sit in `target`.
    pub(super) fn landing(&self, target: (usize, usize)) -> (f64, f64) {
        let (column, to) = target;
        let lane = &self.lanes[column];
        if column == self.from_column {
            return (0.0, slot_offset(&lane.spans(), self.from, to));
        }
        let own = self.own();
        let y = match (lane.cards.get(to), lane.cards.last()) {
            (Some(card), _) => card.y,
            (None, Some(last)) => last.bottom() + self.gap(),
            (None, None) => lane.list.y,
        };
        let x = lane.cards.first().map_or(lane.list.x, |card| card.x);
        (x - own.x, y - own.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64) -> Rect {
        Rect {
            x,
            y,
            width: 100.0,
            height: 40.0,
        }
    }

    /// Three 100px columns 20px apart, cards 40px tall with 10px gaps: 3, 1 and 0 cards.
    fn board(from_column: usize, from: usize) -> Lanes {
        let lane = |x: f64, count: usize| Lane {
            list: Rect {
                x,
                y: 0.0,
                width: 100.0,
                height: 200.0,
            },
            cards: (0..count).map(|at| rect(x, at as f64 * 50.0)).collect(),
            indices: (0..count).collect(),
        };
        Lanes {
            lanes: vec![lane(0.0, 3), lane(120.0, 1), lane(240.0, 0)],
            from_column,
            from,
        }
    }

    #[test]
    fn a_drag_inside_its_column_lands_as_in_a_sortable() {
        let lanes = board(0, 0);
        assert_eq!(lanes.target(0.0, 0.0), Some((0, 0)));
        assert_eq!(lanes.target(10.0, 60.0), Some((0, 1)));
        assert_eq!(lanes.target(-30.0, 150.0), Some((0, 2)));
        assert_eq!(lanes.step(0, 1, (0, 2)), -50.0);
        assert_eq!(lanes.step(0, 2, (0, 2)), -50.0);
        assert_eq!(lanes.landing((0, 2)), (0.0, 100.0));
        assert_eq!(lanes.room(0, (0, 2)), 0.0);
    }

    #[test]
    fn the_column_under_the_cards_centre_takes_it_and_between_columns_the_nearest() {
        let lanes = board(0, 1);
        let column = |dx, dy| lanes.target(dx, dy).map(|(column, _)| column);
        assert_eq!(column(120.0, -50.0), Some(1));
        // A centre in the 20px gap between columns goes to the nearer one.
        assert_eq!(column(59.0, 0.0), Some(0));
        assert_eq!(column(65.0, 0.0), Some(1));
        assert_eq!(column(240.0, 0.0), Some(2));
    }

    #[test]
    fn off_the_board_there_is_no_target() {
        let lanes = board(0, 1);
        // Own centre at (50, 70); the lists span x 0..340 and y 0..200, plus a 50px card each way.
        assert_eq!(lanes.target(-51.0, 0.0), None);
        assert_eq!(lanes.target(291.0, 0.0), None);
        assert_eq!(lanes.target(0.0, -120.0), Some((0, 0)));
        assert_eq!(lanes.target(0.0, -121.0), None);
        assert_eq!(lanes.target(0.0, 180.0), Some((0, 2)));
        assert_eq!(lanes.target(0.0, 181.0), None);
    }

    #[test]
    fn in_another_column_it_lands_before_the_first_card_whose_middle_is_below_its_own() {
        let lanes = board(0, 1);
        // Own middle at y 70; column 1's only card has its middle at 20.
        assert_eq!(lanes.target(120.0, -60.0), Some((1, 0)));
        assert_eq!(lanes.target(120.0, -40.0), Some((1, 1)));
        assert_eq!(lanes.target(240.0, 0.0), Some((2, 0)));
    }

    #[test]
    fn across_columns_the_source_closes_up_and_the_target_opens_a_slot() {
        let lanes = board(0, 1);
        let target = (1, 0);
        assert_eq!(lanes.step(0, 0, target), 0.0);
        assert_eq!(lanes.step(0, 2, target), -50.0);
        assert_eq!(lanes.step(1, 0, target), 50.0);
        assert_eq!(lanes.step(2, 0, target), 0.0);
        assert_eq!(lanes.room(1, target), 50.0);
        assert_eq!(lanes.room(0, target), 0.0);
    }

    #[test]
    fn with_every_card_mounted_a_slot_is_its_index() {
        let lanes = board(0, 1);
        assert_eq!(lanes.lifted_index(), 1);
        assert_eq!(lanes.index_at((0, 0)), 0);
        assert_eq!(lanes.index_at((0, 2)), 2);
        assert_eq!(lanes.index_at((1, 1)), 1);
        assert_eq!(lanes.index_at((2, 0)), 0);
    }

    /// Todo 2517: column 0 shows its cards 1, 3 and 4 of a longer list.
    #[test]
    fn a_filtered_column_lands_by_the_shown_cards_index() {
        let mut lanes = board(0, 1);
        lanes.lanes[0].indices = vec![1, 3, 4];
        assert_eq!(lanes.slot(0, 3), Some(1));
        assert_eq!(lanes.slot(0, 0), None);
        assert_eq!(lanes.lifted_index(), 3);
        // Before the first shown, after the last shown, counted with 3 taken out.
        assert_eq!(lanes.index_at((0, 0)), 1);
        assert_eq!(lanes.index_at((0, 2)), 4);
        lanes.lanes[1].indices = vec![5];
        assert_eq!(lanes.index_at((1, 0)), 5);
        assert_eq!(lanes.index_at((1, 1)), 6);
    }

    #[test]
    fn the_landing_is_the_slots_place_in_the_target_column() {
        let lanes = board(0, 1);
        assert_eq!(lanes.landing((1, 0)), (120.0, -50.0));
        assert_eq!(lanes.landing((1, 1)), (120.0, 0.0));
        assert_eq!(lanes.landing((2, 0)), (240.0, -50.0));
    }
}
