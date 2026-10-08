//! The cards a column shows. A filtered column's `index`es skip, but its sortable
//! (keyboard and move buttons) counts positions from 0: this maps between the two.

use std::collections::BTreeMap;

use dioxus::prelude::*;

/// Each mounted card's `index`, by a key of the column's own.
#[derive(Clone, Copy)]
pub(super) struct Shown {
    cards: Signal<BTreeMap<usize, usize>>,
    /// The `index`es run exactly `0..n`: positions are the `index`es.
    dense: Memo<bool>,
    next_key: CopyValue<usize>,
}

fn is_dense(cards: &BTreeMap<usize, usize>) -> bool {
    let mut indices: Vec<usize> = cards.values().copied().collect();
    indices.sort_unstable();
    indices.iter().enumerate().all(|(at, &index)| at == index)
}

fn rank(cards: &BTreeMap<usize, usize>, index: usize) -> usize {
    cards.values().filter(|&&other| other < index).count()
}

fn nth(cards: &BTreeMap<usize, usize>, position: usize) -> usize {
    let mut indices: Vec<usize> = cards.values().copied().collect();
    indices.sort_unstable();
    indices.get(position).copied().unwrap_or(position)
}

/// The `index` a card dropped in slot `to` among the shown `indices` takes: just before the
/// next shown card, else just after the last one. With `lifted`, its own `index`, the others
/// count after its removal.
pub(super) fn drop_index(indices: &[usize], lifted: Option<usize>, to: usize) -> usize {
    let others: Vec<usize> = indices
        .iter()
        .copied()
        .filter(|&index| Some(index) != lifted)
        .collect();
    let shifted = |index: usize| index - usize::from(lifted.is_some_and(|from| index > from));
    match (others.get(to), others.last(), lifted) {
        (Some(&next), ..) => shifted(next),
        (None, Some(&last), _) => shifted(last) + 1,
        (None, None, Some(from)) => from,
        (None, None, None) => 0,
    }
}

impl Shown {
    /// The `index`es of a move from `position` to slot `to` among the shown cards, landing
    /// where the pointer drag drops it.
    pub(super) fn reorder(self, position: usize, to: usize) -> (usize, usize) {
        let cards = self.cards.peek();
        let mut indices: Vec<usize> = cards.values().copied().collect();
        indices.sort_unstable();
        let from = nth(&cards, position);
        (from, drop_index(&indices, Some(from), to))
    }
}

pub(super) fn use_shown() -> Shown {
    let cards = use_signal(BTreeMap::new);
    let dense = use_memo(move || is_dense(&cards.read()));
    let next_key = use_hook(|| CopyValue::new(0));
    Shown {
        cards,
        dense,
        next_key,
    }
}

/// Registers a card with `index` in `shown`, and returns its position among the shown cards.
/// While the `index`es run `0..n` that is `index`, in the render it changes.
pub(super) fn use_shown_card(shown: Shown, index: usize) -> usize {
    let key = use_hook(|| {
        let mut next = shown.next_key;
        let key = *next.peek();
        next.set(key + 1);
        key
    });
    let mut cards = shown.cards;
    let mut placed = use_signal(|| None::<usize>);
    use_effect(use_reactive!(|index| {
        cards.write().insert(key, index);
        placed.set(Some(index));
    }));
    use_drop(move || {
        if let Ok(mut cards) = cards.try_write() {
            cards.remove(&key);
        }
    });
    let ranked = use_memo(move || placed().map(|at| rank(&cards.read(), at)));
    match (shown.dense)() {
        false => ranked().unwrap_or(index),
        true => index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shown(indices: &[usize]) -> BTreeMap<usize, usize> {
        indices.iter().copied().enumerate().collect()
    }

    #[test]
    fn indices_from_0_without_a_gap_are_dense_in_any_order() {
        assert!(is_dense(&shown(&[])));
        assert!(is_dense(&shown(&[2, 0, 1])));
        assert!(!is_dense(&shown(&[1, 2])));
        assert!(!is_dense(&shown(&[0, 2])));
    }

    /// Todo 2635: a later move lands before the next shown card, as the pointer drop does.
    #[test]
    fn a_drop_lands_before_the_next_shown_card_or_after_the_last() {
        let shown = [1, 4, 6];
        // Card 1 over slot 1 passes 4 and stops before 6: past the hidden 5.
        assert_eq!(drop_index(&shown, Some(1), 1), 5);
        assert_eq!(drop_index(&shown, Some(1), 2), 6);
        assert_eq!(drop_index(&shown, Some(6), 0), 1);
        assert_eq!(drop_index(&shown, Some(4), 0), 1);
        assert_eq!(drop_index(&shown, None, 3), 7);
        assert_eq!(drop_index(&[3], Some(3), 0), 3);
        assert_eq!(drop_index(&[], None, 0), 0);
        assert_eq!(drop_index(&[0, 1, 2], Some(0), 1), 1);
    }

    #[test]
    fn a_position_counts_the_shown_cards_before_it() {
        let cards = shown(&[3, 1, 2]);

        assert_eq!(rank(&cards, 1), 0);
        assert_eq!(rank(&cards, 3), 2);
        assert_eq!(nth(&cards, 0), 1);
        assert_eq!(nth(&cards, 2), 3);
        assert_eq!(nth(&cards, 5), 5);
    }
}
