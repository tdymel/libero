use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::{CellAlign, SortDirection, SortKey},
    column::Column,
    use_table::StateSlice,
};
use crate::{components::common::Glyph, context::IconSlot};

/// One entry of a [`Table`](super::Table)'s sort: a column, named by its header text.
///
/// ```rust
/// # use libero::components::{SortDirection, TableSort};
/// let by_age = vec![TableSort::new("Age", SortDirection::Descending)];
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TableSort {
    pub column: String,
    pub direction: SortDirection,
}

impl TableSort {
    pub fn new(column: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }
}

/// A per-row closure for a [`Table`](super::Table) prop, as `row_key`, or none
/// by default. Compares equal to any other, as a column's closures do.
///
/// ```rust
/// # use libero::components::RowFn;
/// # struct User { id: u64 }
/// let key: RowFn<User, String> = (|u: &User| u.id.to_string()).into();
/// ```
pub struct RowFn<T, R>(Option<RowClosure<T, R>>);

type RowClosure<T, R> = Rc<dyn Fn(&T) -> R>;

impl<T, R> RowFn<T, R> {
    /// `None` when unset.
    pub(super) fn call(&self, row: &T) -> Option<R> {
        self.0.as_ref().map(|f| f(row))
    }
}

impl<T, R, F: Fn(&T) -> R + 'static> From<F> for RowFn<T, R> {
    fn from(f: F) -> Self {
        Self(Some(Rc::new(f)))
    }
}

// Hand-written: a derive would demand `T: Default`, `T: Clone` and `R: Clone`.
impl<T, R> Default for RowFn<T, R> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T, R> Clone for RowFn<T, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, R> PartialEq for RowFn<T, R> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// What the non-generic body needs from a `Column<T>`, once `T` is gone.
pub(super) struct HeaderSpec {
    pub header: String,
    pub align: CellAlign,
    pub sortable: bool,
    pub row_header: bool,
}

pub(super) fn header_specs<T>(columns: &[Column<T>]) -> Vec<HeaderSpec> {
    columns
        .iter()
        .map(|column| HeaderSpec {
            header: column.header.clone(),
            align: column.align,
            sortable: column.sortable,
            row_header: column.row_header,
        })
        .collect()
}

/// `data`'s indices in display order: source order unless a column is sorted.
pub(super) fn row_order<T>(
    data: &[T],
    columns: &[Column<T>],
    active: Option<(usize, SortDirection)>,
) -> Vec<usize> {
    match active {
        Some((index, direction)) => {
            let sort_key = &columns[index].sort_key;
            let keys: Vec<SortKey> = data.iter().map(|row| sort_key(row)).collect();
            sorted_order(&keys, direction)
        }
        None => (0..data.len()).collect(),
    }
}

/// What a click on header `index` asks for: ascending, descending, then unsorted.
pub(super) fn next_sort(
    active: Option<(usize, SortDirection)>,
    index: usize,
    header: &str,
) -> Vec<TableSort> {
    let direction = match active {
        Some((column, SortDirection::Ascending)) if column == index => SortDirection::Descending,
        Some((column, SortDirection::Descending)) if column == index => return Vec::new(),
        _ => SortDirection::Ascending,
    };
    vec![TableSort::new(header, direction)]
}

/// A row's cells and its DOM key: the caller's `row_key`, else its index in
/// `data`. A cell is its text, or a caller's body.
pub(super) struct RowSpec {
    pub key: String,
    /// The row's `data-state`.
    pub states: Option<String>,
    pub attributes: Vec<Attribute>,
    pub cells: Vec<(String, Option<Element>)>,
}

/// Row indices in sorted order. Stable, so equal keys keep source order, and
/// `Empty` sinks to the bottom either way.
pub(super) fn sorted_order(keys: &[SortKey], direction: SortDirection) -> Vec<usize> {
    let mut order: Vec<usize> = (0..keys.len()).collect();
    order.sort_by(|&a, &b| {
        let (a, b) = (&keys[a], &keys[b]);
        match (a.is_empty(), b.is_empty()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => match direction {
                SortDirection::Ascending => a.compare(b),
                SortDirection::Descending => a.compare(b).reverse(),
            },
        }
    });
    order
}

/// The sorted column, keyed by header text so reordering can't move it.
/// With duplicate headers the first wins. One column sorts: the first entry
/// naming a sortable header.
pub(super) fn active_sort(
    headers: &[HeaderSpec],
    sort: &[TableSort],
) -> Option<(usize, SortDirection)> {
    sort.iter().find_map(|sort| {
        headers
            .iter()
            .position(|spec| spec.sortable && spec.header == sort.column)
            .map(|index| (index, sort.direction))
    })
}

fn align_attr(align: CellAlign) -> Option<&'static str> {
    (align != CellAlign::Start).then(|| align.as_str())
}

/// The caption's text and the id the scroll region names itself by.
pub(super) struct CaptionSpec {
    pub text: String,
    pub id: String,
}

/// Everything inside the `<table>`, with `T` gone.
pub(super) struct BodySpec {
    pub caption: Option<CaptionSpec>,
    pub headers: Vec<HeaderSpec>,
    pub rows: Vec<RowSpec>,
    /// The empty row's body: the caller's `empty`, else the localized text.
    pub empty: Element,
    pub active: Option<(usize, SortDirection)>,
    pub sort: StateSlice<Vec<TableSort>>,
}

pub(super) fn render_body(body: BodySpec) -> Element {
    let BodySpec {
        caption,
        headers,
        rows,
        empty,
        active,
        sort,
    } = body;
    let columns = headers.len().max(1);
    let empty = rows.is_empty().then_some(empty);
    rsx! {
        if let Some(spec) = caption {
            caption { id: spec.id, "{spec.text}" }
        }
        thead {
            tr {
                for (index , spec) in headers.iter().enumerate() {
                    th {
                        key: "{index}",
                        scope: "col",
                        "data-align": align_attr(spec.align),
                        "data-sortable": spec.sortable.then_some(true),
                        // On the sorted column only (APG): a "none" on every
                        // other one is read out as "not sorted" at each.
                        aria_sort: match active {
                            Some((column, direction)) if column == index => Some(direction.aria_value()),
                            _ => None,
                        },
                        if spec.sortable {
                            button {
                                r#type: "button",
                                onclick: {
                                    let header = spec.header.clone();
                                    move |_| sort.set(next_sort(active, index, &header))
                                },
                                "{spec.header}"
                                // Always rendered, so sorting can't change the width;
                                // `aria-sort` shows and flips it.
                                Glyph { slot: IconSlot::ArrowDown, icon: lucide::arrow_down::outlined }
                            }
                        } else {
                            "{spec.header}"
                        }
                    }
                }
            }
        }
        tbody {
            if let Some(empty) = empty {
                tr { "data-empty": true,
                    td { colspan: "{columns}", {empty} }
                }
            }
            for row in rows {
                tr {
                    key: "{row.key}",
                    "data-state": row.states,
                    ..row.attributes,
                    for (index , (text , body)) in row.cells.into_iter().enumerate() {
                        if headers.get(index).is_some_and(|spec| spec.row_header) {
                            th {
                                key: "{index}",
                                scope: "row",
                                "data-align": headers.get(index).and_then(|spec| align_attr(spec.align)),
                                "{text}"
                                {body}
                            }
                        } else {
                            td {
                                key: "{index}",
                                "data-align": headers.get(index).and_then(|spec| align_attr(spec.align)),
                                // Text inline: a nested node per cell costs ~1 us a sort.
                                "{text}"
                                {body}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::column::column, *};

    fn keys(values: &[Option<f64>]) -> Vec<SortKey> {
        values
            .iter()
            .map(|value| value.map_or(SortKey::Empty, SortKey::num))
            .collect()
    }

    #[test]
    fn sorts_numbers_both_ways() {
        let keys = keys(&[Some(3.0), Some(1.0), Some(2.0)]);

        assert_eq!(sorted_order(&keys, SortDirection::Ascending), vec![1, 2, 0]);
        assert_eq!(
            sorted_order(&keys, SortDirection::Descending),
            vec![0, 2, 1]
        );
    }

    #[test]
    fn empty_keys_sink_in_both_directions() {
        let keys = keys(&[None, Some(2.0), Some(1.0)]);

        assert_eq!(sorted_order(&keys, SortDirection::Ascending), vec![2, 1, 0]);
        assert_eq!(
            sorted_order(&keys, SortDirection::Descending),
            vec![1, 2, 0]
        );
    }

    #[test]
    fn equal_keys_keep_source_order() {
        let keys = keys(&[Some(1.0), Some(1.0), Some(1.0)]);

        assert_eq!(sorted_order(&keys, SortDirection::Ascending), vec![0, 1, 2]);
        assert_eq!(
            sorted_order(&keys, SortDirection::Descending),
            vec![0, 1, 2]
        );
    }

    fn headers(names: &[&str]) -> Vec<HeaderSpec> {
        names
            .iter()
            .map(|name| HeaderSpec {
                header: name.to_string(),
                align: CellAlign::Start,
                sortable: true,
                row_header: false,
            })
            .collect()
    }

    #[test]
    fn the_sort_follows_its_header_not_its_position() {
        let sort = [TableSort::new("Age", SortDirection::Descending)];

        let active = active_sort(&headers(&["Name", "Age"]), &sort);
        assert_eq!(active, Some((1, SortDirection::Descending)));

        let reordered = active_sort(&headers(&["Age", "Name"]), &sort);
        assert_eq!(reordered, Some((0, SortDirection::Descending)));

        assert_eq!(active_sort(&headers(&["Name"]), &sort), None);
    }

    #[test]
    fn duplicate_headers_sort_the_first_sortable_match() {
        let sort = [TableSort::new("Age", SortDirection::Ascending)];
        let mut specs = headers(&["Age", "Name", "Age"]);

        assert_eq!(active_sort(&specs, &sort).map(|a| a.0), Some(0));
        specs[0].sortable = false;
        assert_eq!(active_sort(&specs, &sort).map(|a| a.0), Some(2));
    }

    #[test]
    fn the_first_entry_naming_a_sortable_header_sorts() {
        let sort = [
            TableSort::new("Height", SortDirection::Ascending),
            TableSort::new("Name", SortDirection::Descending),
            TableSort::new("Age", SortDirection::Ascending),
        ];

        let active = active_sort(&headers(&["Age", "Name"]), &sort);
        assert_eq!(active, Some((1, SortDirection::Descending)));
        assert_eq!(active_sort(&headers(&["Age"]), &[]), None);
    }

    #[test]
    fn text_sorts_case_insensitively() {
        let keys = vec![SortKey::text("beta"), SortKey::text("Alpha")];

        assert_eq!(sorted_order(&keys, SortDirection::Ascending), vec![1, 0]);
    }

    #[test]
    fn a_header_click_cycles_ascending_descending_unsorted() {
        let asc = next_sort(None, 1, "Age");
        assert_eq!(asc, [TableSort::new("Age", SortDirection::Ascending)]);

        let desc = next_sort(Some((1, SortDirection::Ascending)), 1, "Age");
        assert_eq!(desc, [TableSort::new("Age", SortDirection::Descending)]);

        assert!(next_sort(Some((1, SortDirection::Descending)), 1, "Age").is_empty());
    }

    #[test]
    fn another_header_starts_ascending() {
        let next = next_sort(Some((0, SortDirection::Descending)), 1, "Age");

        assert_eq!(next, [TableSort::new("Age", SortDirection::Ascending)]);
    }

    fn ages() -> (Vec<u32>, Vec<Column<u32>>) {
        let columns = vec![
            column("Name").value(|age: &u32| format!("n{age}")),
            column("Age").value(|age: &u32| *age).sortable(),
        ];
        (vec![30, 10, 20], columns)
    }

    #[test]
    fn rows_keep_source_order_until_sorted() {
        let (data, columns) = ages();

        assert_eq!(row_order(&data, &columns, None), vec![0, 1, 2]);
        assert_eq!(
            row_order(&data, &columns, Some((1, SortDirection::Ascending))),
            vec![1, 2, 0]
        );
    }

    #[test]
    fn a_sort_on_an_unknown_or_unsortable_header_is_inactive() {
        let (_, columns) = ages();
        let headers = header_specs(&columns);

        let unknown = [TableSort::new("Height", SortDirection::Ascending)];
        assert_eq!(active_sort(&headers, &unknown), None);
        let unsortable = [TableSort::new("Name", SortDirection::Ascending)];
        assert_eq!(active_sort(&headers, &unsortable), None);
    }
}
