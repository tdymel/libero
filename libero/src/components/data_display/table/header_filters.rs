use dioxus::prelude::*;

use super::{
    column_filter::filter_of,
    core::HeaderSpec,
    filter_popover::{FilterTarget, FilterValue, use_filter_editor},
    pinning::CellPin,
};

/// One cell's field in the header filters row: edits its column's filter
/// value with the column's current operator.
#[component]
pub(super) fn HeaderFilter(target: FilterTarget) -> Element {
    let editor = use_filter_editor(&target);
    let filter = filter_of(&target.slice.read(), &target.column).cloned();
    rsx! {
        FilterValue {
            column: target.column,
            kind: target.kind,
            labels: target.labels,
            size: target.size,
            filter,
            draft: editor.draft.read().clone(),
            ontext: editor.ontext,
            onpick: editor.onpick,
            onday: editor.onday,
        }
    }
}

/// The filters row's key among the header rows, which are keyed by level.
const FILTERS_KEY: &str = "filters";

/// The header filters row: a field under each filterable shown column, by
/// header index in `cells`, and empty cells under the rest and the lead columns.
/// `rowindex` is its `aria-rowindex` in a windowed table.
pub(super) fn filter_row(
    headers: &[HeaderSpec],
    shown: &[usize],
    mut cells: Vec<Option<Element>>,
    reorder: bool,
    detail: bool,
    select: bool,
    rowindex: Option<usize>,
) -> Element {
    let cells: Vec<Element> = shown
        .iter()
        .map(|&index| {
            let pin = headers[index].pin.as_ref();
            rsx! {
                td {
                    key: "{index}",
                    "data-filter-cell": true,
                    "data-pin": pin.map(|pin| pin.side.as_str()),
                    "data-pin-edge": pin.filter(|pin| pin.edge).map(|_| true),
                    style: pin.map(CellPin::style),
                    {cells.get_mut(index).and_then(Option::take)}
                }
            }
        })
        .collect();
    rsx! {
        tr { key: "{FILTERS_KEY}", "data-filters": true, aria_rowindex: rowindex.map(|index| index.to_string()),
            if reorder {
                td { "data-reorder": true }
            }
            if detail {
                td { "data-detail-toggle": true }
            }
            if select {
                td { "data-select": true }
            }
            {cells.into_iter()}
        }
    }
}
