use dioxus::prelude::*;

use super::cell_value::{CellAlign, SortDirection, SortKey};

/// What the non-generic body needs from a `Column<T>`, once `T` is gone.
pub(super) struct HeaderSpec {
    pub header: String,
    pub align: CellAlign,
    pub sortable: bool,
}

/// A row's cells, keyed by its position in the unsorted `data`.
pub(super) struct RowSpec {
    pub index: usize,
    pub cells: Vec<Element>,
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

fn align_attr(align: CellAlign) -> Option<&'static str> {
    (align != CellAlign::Start).then(|| align.as_str())
}

pub(super) fn render_body(
    headers: Vec<HeaderSpec>,
    rows: Vec<RowSpec>,
    mut sort: Signal<Option<(usize, SortDirection)>>,
) -> Element {
    let active = *sort.read();

    rsx! {
        thead {
            tr {
                for (index , spec) in headers.iter().enumerate() {
                    th {
                        key: "{index}",
                        scope: "col",
                        "data-align": align_attr(spec.align),
                        "data-sortable": spec.sortable.then_some(true),
                        // Only a sortable column may advertise `aria-sort`.
                        aria_sort: spec.sortable.then(|| match active {
                            Some((column, direction)) if column == index => direction.aria_value(),
                            _ => "none",
                        }),
                        if spec.sortable {
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    let next = match *sort.read() {
                                        Some((column, direction)) if column == index => {
                                            (index, direction.flipped())
                                        }
                                        _ => (index, SortDirection::default()),
                                    };
                                    sort.set(Some(next));
                                },
                                "{spec.header}"
                                {sort_arrow()}
                            }
                        } else {
                            "{spec.header}"
                        }
                    }
                }
            }
        }
        tbody {
            for row in rows {
                tr {
                    key: "{row.index}",
                    for (index , cell) in row.cells.into_iter().enumerate() {
                        td {
                            key: "{index}",
                            "data-align": headers.get(index).and_then(|spec| align_attr(spec.align)),
                            {cell}
                        }
                    }
                }
            }
        }
    }
}

/// Always rendered, so sorting a column can't change its header's width. The
/// `aria-sort` on the `th` is what fades and flips it.
fn sort_arrow() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 5v14" }
            path { d: "M6 13l6 6 6-6" }
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

    #[test]
    fn text_sorts_case_insensitively() {
        let keys = vec![SortKey::text("beta"), SortKey::text("Alpha")];

        assert_eq!(sorted_order(&keys, SortDirection::Ascending), vec![1, 0]);
    }
}
