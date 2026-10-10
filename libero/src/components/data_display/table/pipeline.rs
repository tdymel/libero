//! The body's rows from `data`: keys, filter mask, sorted order, groups, then the page.

use std::{collections::HashMap, ops::Range, rc::Rc};

/// A body row in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum VisibleRow {
    /// A row of `data`, by its index there.
    Data(usize),
    /// A group row, by its index in the [`GroupInfo`] list of [`group_rows`].
    Group(usize),
}

impl VisibleRow {
    /// The `data` index of a data row.
    pub fn data(self) -> Option<usize> {
        match self {
            Self::Data(index) => Some(index),
            Self::Group(_) => None,
        }
    }
}

/// A group of rows sharing their grouped columns' texts.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct GroupInfo {
    /// Its texts, from the outermost grouped column in.
    pub path: Vec<String>,
    /// 0 for the outermost groups.
    pub depth: usize,
    /// Its text, or the empty-group label.
    pub label: String,
    /// Its rows' `data` indices, nested groups' too, in sorted order.
    pub leaves: Vec<usize>,
    pub id: String,
    pub open: bool,
}

/// A group's id: its path joined by `/`, a `/` or `\` in a text escaped by `\`.
pub(super) fn group_id(path: &[String]) -> String {
    let escape = |text: &String| text.replace('\\', "\\\\").replace('/', "\\/");
    path.iter().map(escape).collect::<Vec<_>>().join("/")
}

/// The `leaves` nested into groups, one level per entry of `texts` (each by `data`
/// index). Groups follow first appearance, so a sort orders them; an `open` one
/// shows its rows.
pub(super) fn group_rows(
    leaves: &[VisibleRow],
    texts: &[Vec<String>],
    open: &dyn Fn(&str) -> bool,
    empty: &str,
) -> (Vec<VisibleRow>, Vec<GroupInfo>) {
    let mut rows = Vec::with_capacity(leaves.len());
    let mut groups = Vec::new();
    let leaves: Vec<usize> = leaves.iter().filter_map(|row| row.data()).collect();
    let mut nest = Nest {
        texts,
        open,
        empty,
        rows: &mut rows,
        groups: &mut groups,
        path: Vec::new(),
    };
    nest.level(leaves);
    (rows, groups)
}

struct Nest<'a> {
    texts: &'a [Vec<String>],
    open: &'a dyn Fn(&str) -> bool,
    empty: &'a str,
    rows: &'a mut Vec<VisibleRow>,
    groups: &'a mut Vec<GroupInfo>,
    path: Vec<String>,
}

impl Nest<'_> {
    fn level(&mut self, leaves: Vec<usize>) {
        let depth = self.path.len();
        let Some(texts) = self.texts.get(depth) else {
            self.rows.extend(leaves.into_iter().map(VisibleRow::Data));
            return;
        };
        let mut found: Vec<(&str, Vec<usize>)> = Vec::new();
        let mut at: HashMap<&str, usize> = HashMap::new();
        for index in leaves {
            let text = texts[index].as_str();
            let slot = *at.entry(text).or_insert_with(|| {
                found.push((text, Vec::new()));
                found.len() - 1
            });
            found[slot].1.push(index);
        }
        for (text, leaves) in found {
            self.path.push(text.to_string());
            let id = group_id(&self.path);
            let open = (self.open)(&id);
            let label = match text.is_empty() {
                true => self.empty.to_string(),
                false => text.to_string(),
            };
            self.rows.push(VisibleRow::Group(self.groups.len()));
            self.groups.push(GroupInfo {
                path: self.path.clone(),
                depth,
                label,
                leaves: leaves.clone(),
                id,
                open,
            });
            if open {
                self.level(leaves);
            }
            self.path.pop();
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

    fn texts(levels: &[&[&str]]) -> Vec<Vec<String>> {
        levels
            .iter()
            .map(|level| level.iter().map(|text| text.to_string()).collect())
            .collect()
    }

    fn data(order: &[usize]) -> Vec<VisibleRow> {
        order.iter().copied().map(VisibleRow::Data).collect()
    }

    #[test]
    fn groups_follow_first_appearance_and_open_ones_show_their_rows() {
        let texts = texts(&[&["b", "a", "b", "a"]]);
        let (rows, groups) = group_rows(&data(&[0, 1, 2, 3]), &texts, &|id| id == "b", "-");
        use VisibleRow::*;
        assert_eq!(rows, [Group(0), Data(0), Data(2), Group(1)]);
        assert_eq!(groups[0].leaves, [0, 2]);
        assert_eq!((groups[1].label.as_str(), groups[1].open), ("a", false));
        assert_eq!(groups[1].leaves, [1, 3]);
    }

    #[test]
    fn a_new_sort_reorders_the_groups_and_their_rows() {
        let texts = texts(&[&["b", "a", "b", "a"]]);
        let (rows, groups) = group_rows(&data(&[3, 2, 1, 0]), &texts, &|_| true, "-");
        use VisibleRow::*;
        assert_eq!(
            rows,
            [Group(0), Data(3), Data(1), Group(1), Data(2), Data(0)]
        );
        assert_eq!(groups[0].label, "a");
    }

    #[test]
    fn nested_groups_carry_their_path_and_depth() {
        let texts = texts(&[&["x", "x", "y"], &["1", "2", "1"]]);
        let (rows, groups) = group_rows(&data(&[0, 1, 2]), &texts, &|id| id != "y", "-");
        use VisibleRow::*;
        assert_eq!(
            rows,
            [Group(0), Group(1), Data(0), Group(2), Data(1), Group(3)]
        );
        assert_eq!(groups[2].path, ["x", "2"]);
        assert_eq!((groups[2].depth, groups[2].id.as_str()), (1, "x/2"));
        // A closed group lists no nested group rows, but counts every row.
        assert_eq!((groups.len(), groups[3].leaves.len()), (4, 1));
    }

    #[test]
    fn an_empty_text_is_one_group_under_the_empty_label() {
        let texts = texts(&[&["", "a", ""]]);
        let (_, groups) = group_rows(&data(&[0, 1, 2]), &texts, &|_| false, "(empty)");
        assert_eq!(groups[0].label, "(empty)");
        assert_eq!((groups[0].id.as_str(), groups[0].leaves.len()), ("", 2));
    }

    #[test]
    fn group_ids_escape_the_separator() {
        let path = |texts: &[&str]| texts.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        assert_eq!(group_id(&path(&["a/b", "c"])), "a\\/b/c");
        assert_ne!(group_id(&path(&["a/b"])), group_id(&path(&["a", "b"])));
        assert_eq!(group_id(&path(&["a\\"])), "a\\\\");
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
