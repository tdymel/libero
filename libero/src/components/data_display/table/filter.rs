use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    column::Column,
    column_filter::{CellTest, FilterLogic, passes_all},
    use_table::StateSlice,
};
use crate::{
    components::{accessibility::Announcer, form::TextField},
    hooks::{use_debounced_callback, use_id},
    localization::TableLabels,
    sx::sx,
    theme::Size,
};

/// The quick filter's search field. Once typing settles, it announces how many
/// rows are left (WCAG 4.1.3).
#[component]
pub(super) fn QuickFilter(
    slice: StateSlice<String>,
    /// The rows the current text keeps.
    results: usize,
    announcer: Announcer,
    labels: TableLabels,
    size: Size,
    /// The caption's or the table's `aria-labelledby` ids, which tell this
    /// field from another table's.
    caption: Option<String>,
    /// Else the table's `aria-label`, read out as the field's description.
    table_label: Option<String>,
) -> Element {
    let label_id = use_id();
    let described_by = caption.or_else(|| table_label.as_ref().map(|_| label_id()));
    let mut latest = use_hook(|| CopyValue::new(results));
    latest.set(results);
    let announce = use_debounced_callback(
        move |()| announcer.say((labels.results)(*latest.peek())),
        500,
    );
    rsx! {
        if let Some(text) = table_label {
            span { id: label_id, hidden: true, "{text}" }
        }
        TextField {
            r#type: "search",
            label: labels.search,
            size,
            value: slice.read(),
            aria_describedby: described_by,
            sx: sx().max_width("20rem").margin("0 0 8px"),
            oninput: move |text| {
                slice.set(text);
                announce(());
            },
        }
    }
}

/// A quick-filter query as its lowercased words; empty keeps every row.
pub(super) fn query_words(query: &str) -> Vec<String> {
    query.split_whitespace().map(str::to_lowercase).collect()
}

/// Each row's `searched` cell texts, lowercased and joined by a newline, which
/// no query word holds, so a word never matches across two cells.
pub(super) fn searched_texts<T>(
    data: &[T],
    columns: &[Column<T>],
    searched: &[usize],
) -> Vec<String> {
    data.iter()
        .map(|row| {
            let cells: Vec<String> = searched
                .iter()
                .map(|&index| (columns[index].text)(row))
                .collect();
            cells.join("\n").to_lowercase()
        })
        .collect()
}

/// Which rows the quick filter keeps, by index: every word occurs in the row's
/// [`searched_texts`].
pub(super) fn quick_filter(texts: &[String], words: &[String]) -> Vec<bool> {
    texts
        .iter()
        .map(|text| words.iter().all(|word| text.contains(word.as_str())))
        .collect()
}

/// [`quick_filter`] and the column filters across renders: only a change of
/// rows, columns or filters reruns them, so a sort or page change does not.
/// The lowercased cell texts outlive a query change, so typing skips them.
pub(super) struct FilteredRows<T> {
    input: Option<FilterInput<T>>,
    kept: Rc<Vec<bool>>,
    texts: Option<(TextInput<T>, Vec<String>)>,
}

type TextInput<T> = (Rc<Vec<T>>, Vec<Column<T>>, Vec<usize>);

type FilterInput<T> = (
    Rc<Vec<T>>,
    Vec<Column<T>>,
    Vec<usize>,
    Vec<String>,
    Vec<(usize, CellTest)>,
    FilterLogic,
);

impl<T> Default for FilteredRows<T> {
    fn default() -> Self {
        Self {
            input: None,
            kept: Rc::default(),
            texts: None,
        }
    }
}

fn same_rows<T: PartialEq>(cached: &Rc<Vec<T>>, data: &Rc<Vec<T>>) -> bool {
    Rc::ptr_eq(cached, data) || cached == data
}

impl<T: PartialEq> FilteredRows<T> {
    /// `None` when the query and the tests keep every row.
    pub fn kept(
        &mut self,
        data: &Rc<Vec<T>>,
        columns: &[Column<T>],
        searched: &[usize],
        words: &[String],
        tests: &[(usize, CellTest)],
        logic: FilterLogic,
    ) -> Option<Rc<Vec<bool>>> {
        if words.is_empty() && tests.is_empty() {
            self.input = None;
            self.kept = Rc::default();
            return None;
        }
        let fresh =
            self.input
                .as_ref()
                .is_some_and(|(rows, cols, cells, query, checks, joined)| {
                    query == words
                        && checks == tests
                        && *joined == logic
                        && cells == searched
                        && cols.as_slice() == columns
                        && same_rows(rows, data)
                });
        if !fresh {
            let mut kept = match words.is_empty() {
                true => vec![true; data.len()],
                false => quick_filter(self.texts(data, columns, searched), words),
            };
            if !tests.is_empty() {
                for (keep, row) in kept.iter_mut().zip(data.iter()) {
                    *keep = *keep && passes_all(row, columns, tests, logic);
                }
            }
            self.kept = Rc::new(kept);
            self.input = Some((
                data.clone(),
                columns.to_vec(),
                searched.to_vec(),
                words.to_vec(),
                tests.to_vec(),
                logic,
            ));
        }
        Some(self.kept.clone())
    }

    fn texts(&mut self, data: &Rc<Vec<T>>, columns: &[Column<T>], searched: &[usize]) -> &[String] {
        let fresh = self.texts.as_ref().is_some_and(|((rows, cols, cells), _)| {
            cells == searched && cols.as_slice() == columns && same_rows(rows, data)
        });
        if !fresh {
            let texts = searched_texts(data, columns, searched);
            self.texts = Some(((data.clone(), columns.to_vec(), searched.to_vec()), texts));
        }
        self.texts.as_ref().map_or(&[], |(_, texts)| texts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::data_display::table::column::column;

    #[derive(PartialEq)]
    struct User {
        name: &'static str,
        city: &'static str,
        age: u32,
    }

    const USERS: [User; 3] = [
        User {
            name: "Ada Lovelace",
            city: "London",
            age: 36,
        },
        User {
            name: "Alan Turing",
            city: "London",
            age: 41,
        },
        User {
            name: "Grace Hopper",
            city: "New York",
            age: 85,
        },
    ];

    fn columns() -> Vec<Column<User>> {
        vec![
            column("Name").value(|u: &User| u.name),
            column("City").value(|u: &User| u.city),
            column("Age").value(|u: &User| u.age),
        ]
    }

    fn kept(query: &str, searched: &[usize]) -> Vec<bool> {
        quick_filter(
            &searched_texts(&USERS, &columns(), searched),
            &query_words(query),
        )
    }

    #[test]
    fn words_split_on_whitespace_and_lowercase() {
        assert_eq!(query_words("  Ada  LONDON "), ["ada", "london"]);
        assert!(query_words("   ").is_empty());
    }

    #[test]
    fn a_word_matches_any_searched_column_ignoring_case() {
        assert_eq!(kept("LONDON", &[0, 1, 2]), [true, true, false]);
        assert_eq!(kept("85", &[0, 1, 2]), [false, false, true]);
    }

    #[test]
    fn every_word_must_match_but_in_any_column() {
        assert_eq!(kept("london ada", &[0, 1, 2]), [true, false, false]);
    }

    #[test]
    fn a_word_never_spans_two_cells() {
        assert_eq!(kept("lacelon", &[0, 1]), [false, false, false]);
        assert_eq!(kept("lovelace", &[0, 1]), [true, false, false]);
    }

    #[test]
    fn unsearched_columns_never_match() {
        assert_eq!(kept("london", &[0, 2]), [false, false, false]);
    }

    #[test]
    fn the_cache_keeps_its_result_until_the_query_changes() {
        let data = Rc::new(Vec::from(USERS));
        let columns = columns();
        let mut filtered = FilteredRows::default();
        let words = query_words("york");
        let first = filtered
            .kept(&data, &columns, &[0, 1], &words, &[], FilterLogic::And)
            .unwrap();
        let again = filtered
            .kept(&data, &columns, &[0, 1], &words, &[], FilterLogic::And)
            .unwrap();
        assert!(Rc::ptr_eq(&first, &again));
        let other = filtered
            .kept(
                &data,
                &columns,
                &[0, 1],
                &query_words("ada"),
                &[],
                FilterLogic::And,
            )
            .unwrap();
        assert_eq!(*other, [true, false, false]);
        assert!(
            filtered
                .kept(&data, &columns, &[0, 1], &[], &[], FilterLogic::And)
                .is_none()
        );
    }

    #[test]
    fn column_filters_and_the_quick_filter_both_apply() {
        use super::super::column_filter::{ColumnFilter, FilterOperator, cell_tests};
        let data = Rc::new(Vec::from(USERS));
        let columns = columns();
        let tests = cell_tests(
            &columns,
            &[ColumnFilter::new("Age", FilterOperator::LessThan, "50")],
        );
        let mut filtered = FilteredRows::default();
        let only_tests = filtered
            .kept(&data, &columns, &[0, 1], &[], &tests, FilterLogic::And)
            .unwrap();
        assert_eq!(*only_tests, [true, true, false]);
        let both = filtered
            .kept(
                &data,
                &columns,
                &[0, 1],
                &query_words("alan"),
                &tests,
                FilterLogic::And,
            )
            .unwrap();
        assert_eq!(*both, [false, true, false]);
    }
}
