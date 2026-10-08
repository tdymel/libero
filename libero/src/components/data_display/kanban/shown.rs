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

impl Shown {
    /// The `index` of the card shown at `position`; `position` itself past the last.
    pub(super) fn index_at(self, position: usize) -> usize {
        nth(&self.cards.peek(), position)
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
