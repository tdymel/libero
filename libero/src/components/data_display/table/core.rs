use dioxus::prelude::*;

use super::cell_value::{CellAlign, SortDirection, SortKey};
use crate::components::common::ArrowDownIcon;

/// What the non-generic body needs from a `Column<T>`, once `T` is gone.
pub(super) struct HeaderSpec {
    pub header: String,
    pub align: CellAlign,
    pub sortable: bool,
    pub row_header: bool,
}

/// A row's cells, keyed by its position in the unsorted `data`. A cell is its
/// text, or a caller's body and no text.
pub(super) struct RowSpec {
    pub index: usize,
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

/// The column the sort sits on now. Keyed by header text, so reordering or
/// hiding other columns can't move it; with duplicate headers the first wins.
pub(super) fn active_sort(
    headers: &[HeaderSpec],
    sort: Option<&(String, SortDirection)>,
) -> Option<(usize, SortDirection)> {
    let (header, direction) = sort?;
    headers
        .iter()
        .position(|spec| spec.sortable && spec.header == *header)
        .map(|index| (index, *direction))
}

fn align_attr(align: CellAlign) -> Option<&'static str> {
    (align != CellAlign::Start).then(|| align.as_str())
}

/// The caption's text and the id the scroll region names itself by.
pub(super) struct CaptionSpec {
    pub text: String,
    pub id: String,
}

pub(super) fn render_body(
    caption: Option<CaptionSpec>,
    headers: Vec<HeaderSpec>,
    rows: Vec<RowSpec>,
    empty: Option<Element>,
    active: Option<(usize, SortDirection)>,
    mut sort: Signal<Option<(String, SortDirection)>>,
) -> Element {
    let columns = headers.len().max(1);
    let empty = empty.filter(|_| rows.is_empty());
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
                                    move |_| {
                                        let direction = match active {
                                            Some((column, direction)) if column == index => {
                                                direction.flipped()
                                            }
                                            _ => SortDirection::default(),
                                        };
                                        sort.set(Some((header.clone(), direction)));
                                    }
                                },
                                "{spec.header}"
                                // Always rendered, so sorting a column can't
                                // change its header's width. The `th`'s
                                // `aria-sort` is what shows and flips it.
                                ArrowDownIcon {}
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
                    key: "{row.index}",
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
    use super::*;

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
        let sort = ("Age".to_string(), SortDirection::Descending);

        let active = active_sort(&headers(&["Name", "Age"]), Some(&sort));
        assert_eq!(active, Some((1, SortDirection::Descending)));

        let reordered = active_sort(&headers(&["Age", "Name"]), Some(&sort));
        assert_eq!(reordered, Some((0, SortDirection::Descending)));

        assert_eq!(active_sort(&headers(&["Name"]), Some(&sort)), None);
    }

    #[test]
    fn duplicate_headers_sort_the_first_sortable_match() {
        let sort = ("Age".to_string(), SortDirection::Ascending);
        let mut specs = headers(&["Age", "Name", "Age"]);

        assert_eq!(active_sort(&specs, Some(&sort)).map(|a| a.0), Some(0));
        specs[0].sortable = false;
        assert_eq!(active_sort(&specs, Some(&sort)).map(|a| a.0), Some(2));
    }

    #[test]
    fn text_sorts_case_insensitively() {
        let keys = vec![SortKey::text("beta"), SortKey::text("Alpha")];

        assert_eq!(sorted_order(&keys, SortDirection::Ascending), vec![1, 0]);
    }
}
