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
            options: target.options,
            onoption: editor.onoption,
            onany: editor.onany,
            compact: true,
        }
    }
}

/// The filters row's key among the header rows, which are keyed by level.
const FILTERS_KEY: &str = "filters";

/// The header filters row: a field under each filterable shown column, by
/// header index in `cells`, and empty cells under the rest and the lead columns.
/// `rowindex` is its `aria-rowindex` in a windowed table, `colindex` each column's
/// `aria-colindex` in a grid with a hidden column.
pub(super) fn filter_row(
    headers: &[HeaderSpec],
    shown: &[usize],
    mut cells: Vec<Option<Element>>,
    (reorder, detail, select): (bool, bool, bool),
    (rowindex, colindex): (Option<usize>, Option<&[usize]>),
) -> Element {
    let cells: Vec<Element> = shown
        .iter()
        .map(|&index| {
            let pin = headers[index].pin.as_ref();
            rsx! {
                td {
                    key: "{headers[index].key}",
                    "data-filter-cell": true,
                    "data-pin": pin.map(|pin| pin.side.as_str()),
                    "data-pin-edge": pin.filter(|pin| pin.edge).map(|_| true),
                    style: pin.map(CellPin::style),
                    aria_colindex: colindex.map(|indices| indices[index].to_string()),
                    {cells.get_mut(index).and_then(Option::take)}
                }
            }
        })
        .collect();
    // The lead cells come first, one column each.
    let lead = |place: usize| colindex.map(|_| place.to_string());
    let detail_at = 1 + usize::from(reorder);
    let select_at = detail_at + usize::from(detail);
    rsx! {
        tr { key: "{FILTERS_KEY}", "data-filters": true, aria_rowindex: rowindex.map(|index| index.to_string()),
            if reorder {
                td { "data-reorder": true, aria_colindex: lead(1) }
            }
            if detail {
                td { "data-detail-toggle": true, aria_colindex: lead(detail_at) }
            }
            if select {
                td { "data-select": true, aria_colindex: lead(select_at) }
            }
            {cells.into_iter()}
        }
    }
}
