//! The body's rows from `data`: keys, filter mask, sorted order, then the page.

use std::{ops::Range, rc::Rc};

/// A body row in display order; group rows join it as their own kind (epic 1156).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum VisibleRow {
    /// A row of `data`, by its index there.
    Data(usize),
}

impl VisibleRow {
    /// The `data` index of a data row.
    pub fn data(self) -> Option<usize> {
        match self {
            Self::Data(index) => Some(index),
        }
    }
}

/// The last rows, whether they were keyed by `row_key`, and their keys.
pub(super) type KnownRows<T> = Option<(Rc<Vec<T>>, bool, Rc<[String]>)>;

/// The rows and their keys, the `known` ones while `data` stays equal: the sort and
/// filter caches then match it by pointer, and no key is built again (todo 1983).
pub(super) fn keyed_rows<T: PartialEq>(
    known: &KnownRows<T>,
    data: Vec<T>,
    keyed: bool,
    key: impl Fn(&T) -> Option<String>,
) -> (Rc<Vec<T>>, Rc<[String]>) {
    match known {
        Some((rows, was_keyed, keys)) if *was_keyed == keyed && **rows == data => {
            (rows.clone(), keys.clone())
        }
        _ => {
            let keys = data
                .iter()
                .enumerate()
                .map(|(index, row)| key(row).unwrap_or_else(|| index.to_string()))
                .collect();
            (Rc::new(data), keys)
        }
    }
}

/// The keys a select-all reaches: every row the filter `kept`, on any page.
pub(super) fn selection_scope(keys: &Rc<[String]>, kept: Option<&[bool]>) -> Rc<[String]> {
    match kept {
        Some(kept) => keys
            .iter()
            .zip(kept)
            .filter(|(_, kept)| **kept)
            .map(|(key, _)| key.clone())
            .collect(),
        None => keys.clone(),
    }
}

/// The sorted `order` of `data` indices as rows, less the ones the filter did not keep.
pub(super) fn visible_rows(order: Vec<usize>, kept: Option<&[bool]>) -> Vec<VisibleRow> {
    order
        .into_iter()
        .filter(|&index| kept.is_none_or(|kept| kept[index]))
        .map(VisibleRow::Data)
        .collect()
}

/// Keeps the rows of one page, `shown` by position.
pub(super) fn cut_page(rows: &mut Vec<VisibleRow>, shown: Range<usize>) {
    rows.truncate(shown.end);
    rows.drain(..shown.start.min(rows.len()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(row: &&str) -> Option<String> {
        Some(row.to_uppercase())
    }

    #[test]
    fn rows_are_keyed_by_row_key_else_by_index() {
        let (_, keys) = keyed_rows(&None, vec!["a", "b"], true, key);
        assert_eq!(&*keys, ["A", "B"]);
        let (_, keys) = keyed_rows(&None, vec!["a", "b"], false, |_| None);
        assert_eq!(&*keys, ["0", "1"]);
    }

    #[test]
    fn equal_rows_reuse_the_known_rows_and_keys() {
        let (rows, keys) = keyed_rows(&None, vec!["a", "b"], true, key);
        let known = Some((rows.clone(), true, keys.clone()));
        let (again, again_keys) = keyed_rows(&known, vec!["a", "b"], true, |_| unreachable!());
        assert!(Rc::ptr_eq(&rows, &again) && Rc::ptr_eq(&keys, &again_keys));
        let (changed, _) = keyed_rows(&known, vec!["a", "c"], true, key);
        assert!(!Rc::ptr_eq(&rows, &changed));
        let (_, unkeyed) = keyed_rows(&known, vec!["a", "b"], false, |_| None);
        assert_eq!(&*unkeyed, ["0", "1"]);
    }

    #[test]
    fn the_selection_scope_is_the_kept_rows_keys() {
        let keys: Rc<[String]> = ["a", "b", "c"].map(String::from).into();
        assert!(Rc::ptr_eq(&selection_scope(&keys, None), &keys));
        assert_eq!(
            &*selection_scope(&keys, Some(&[true, false, true])),
            ["a", "c"]
        );
    }

    #[test]
    fn visible_rows_follow_the_sort_and_skip_filtered_rows() {
        assert_eq!(
            visible_rows(vec![2, 0, 1], None),
            [2, 0, 1].map(VisibleRow::Data)
        );
        assert_eq!(
            visible_rows(vec![2, 0, 1], Some(&[true, false, true])),
            [2, 0].map(VisibleRow::Data)
        );
    }

    #[test]
    fn a_page_keeps_its_slice_of_the_rows() {
        let rows = || (0..5).map(VisibleRow::Data).collect::<Vec<_>>();
        let mut page = rows();
        cut_page(&mut page, 2..4);
        assert_eq!(page, [2, 3].map(VisibleRow::Data));
        let mut last = rows();
        cut_page(&mut last, 4..6);
        assert_eq!(last, [VisibleRow::Data(4)]);
        let mut past = rows();
        cut_page(&mut past, 10..12);
        assert!(past.is_empty());
    }
}
