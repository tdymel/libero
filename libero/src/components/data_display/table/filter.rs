use std::rc::Rc;

use dioxus::prelude::*;

use super::{column::Column, use_table::StateSlice};
use crate::{
    components::{accessibility::Announcer, form::TextField},
    hooks::use_debounced_callback,
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
    /// The caption's id, which tells this field from another table's.
    caption: Option<String>,
) -> Element {
    let mut latest = use_hook(|| CopyValue::new(results));
    latest.set(results);
    let announce = use_debounced_callback(
        move |()| announcer.say((labels.results)(*latest.peek())),
        500,
    );
    rsx! {
        TextField {
            r#type: "search",
            label: labels.search,
            size,
            value: slice.read(),
            aria_describedby: caption,
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

/// Which of `data`'s rows the quick filter keeps, by index: every word occurs
/// in the cell text of one of the `searched` columns, ignoring case.
pub(super) fn quick_filter<T>(
    data: &[T],
    columns: &[Column<T>],
    searched: &[usize],
    words: &[String],
) -> Vec<bool> {
    data.iter()
        .map(|row| {
            let texts: Vec<String> = searched
                .iter()
                .map(|&index| (columns[index].text)(row).to_lowercase())
                .collect();
            words
                .iter()
                .all(|word| texts.iter().any(|text| text.contains(word.as_str())))
        })
        .collect()
}

/// [`quick_filter`] across renders: only a change of rows, columns or query
/// reruns it, so a sort or page change does not.
pub(super) struct FilteredRows<T> {
    input: Option<FilterInput<T>>,
    kept: Rc<Vec<bool>>,
}

type FilterInput<T> = (Rc<Vec<T>>, Vec<Column<T>>, Vec<usize>, Vec<String>);

impl<T> Default for FilteredRows<T> {
    fn default() -> Self {
        Self {
            input: None,
            kept: Rc::default(),
        }
    }
}

impl<T: PartialEq> FilteredRows<T> {
    /// `None` when the query keeps every row.
    pub fn kept(
        &mut self,
        data: &Rc<Vec<T>>,
        columns: &[Column<T>],
        searched: &[usize],
        words: &[String],
    ) -> Option<Rc<Vec<bool>>> {
        if words.is_empty() {
            *self = Self::default();
            return None;
        }
        let fresh = self
            .input
            .as_ref()
            .is_some_and(|(rows, cols, cells, query)| {
                query == words
                    && cells == searched
                    && cols.as_slice() == columns
                    && (Rc::ptr_eq(rows, data) || rows == data)
            });
        if !fresh {
            self.kept = Rc::new(quick_filter(data, columns, searched, words));
            self.input = Some((
                data.clone(),
                columns.to_vec(),
                searched.to_vec(),
                words.to_vec(),
            ));
        }
        Some(self.kept.clone())
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
        quick_filter(&USERS, &columns(), searched, &query_words(query))
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
    fn unsearched_columns_never_match() {
        assert_eq!(kept("london", &[0, 2]), [false, false, false]);
    }

    #[test]
    fn the_cache_keeps_its_result_until_the_query_changes() {
        let data = Rc::new(Vec::from(USERS));
        let columns = columns();
        let mut filtered = FilteredRows::default();
        let words = query_words("york");
        let first = filtered.kept(&data, &columns, &[0, 1], &words).unwrap();
        let again = filtered.kept(&data, &columns, &[0, 1], &words).unwrap();
        assert!(Rc::ptr_eq(&first, &again));
        let other = filtered
            .kept(&data, &columns, &[0, 1], &query_words("ada"))
            .unwrap();
        assert_eq!(*other, [true, false, false]);
        assert!(filtered.kept(&data, &columns, &[0, 1], &[]).is_none());
    }
}
