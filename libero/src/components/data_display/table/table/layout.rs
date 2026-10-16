//! The header layout: the shown columns in place order, their pins and lead widths,
//! and the header's menus and filter cells.

use std::{collections::HashMap, rc::Rc};

use dioxus::prelude::*;

use super::super::{
    column::Column,
    column_drag::{ColumnDrag, DropPlan},
    column_menu::{ColumnMenu, MenuColumn, MenuFocus},
    column_order::{moved, order_unpinned, ranked},
    core::{ActiveSort, HeaderSpec, WidthSpec, active_sort, header_specs},
    filter_panel::PanelControl,
    filter_popover::FilterTarget,
    header_filters::HeaderFilter,
    pinning::{PinRoom, PinSide, fit_pins, has_start_pins, pin_columns, pinned_extent},
    resize::{ColumnResize, ColumnWidths, MenuWidth},
    use_table::TableState,
};
use super::{TableProps, detail_width, reorder_width, select_width};
use crate::{
    components::accessibility::Announcer, localization::labels::table::TableLabels, theme::Size,
    utils::warn,
};

/// The measures a layout fits its pins to, and the pins it warned of.
pub(super) struct Measures {
    pub area_width: Signal<Option<f64>>,
    pub lead_px: Signal<f64>,
    pub pin_widths: Signal<HashMap<String, f64>>,
    pub pin_edges: Signal<[Option<f64>; 2]>,
    pub warned: CopyValue<Vec<String>>,
}

pub(super) struct HeaderLayout {
    pub headers: Vec<HeaderSpec>,
    /// The shown columns' header indices, in place order.
    pub layout: Vec<usize>,
    pub widths: ColumnWidths,
    pub column_order: Vec<String>,
    pub lead_width: Option<String>,
    /// Whether the table follows its area's width: only a pinned one does.
    pub tracks_area: bool,
    pub pins_start: bool,
    pub pins_select: bool,
    pub pins: bool,
    /// The inline start and end a focus scroll stops inside.
    pub pinned_pads: [Option<String>; 2],
    /// The lead width to measure, while the leading columns stay pinned.
    pub lead_probe: Option<String>,
    pub active: ActiveSort,
    /// The column group header rows above the column headers.
    pub group_rows: usize,
    pub head_rows: usize,
}

/// The header specs, pinned and ordered; with `drag`, its drop plan follows them.
#[allow(clippy::too_many_arguments)]
pub(super) fn header_layout<T: Clone + PartialEq + 'static>(
    props: &TableProps<T>,
    state: TableState,
    preview: Option<(usize, f64)>,
    size: Size,
    (has_detail, has_reorder): (bool, bool),
    measures: Measures,
    drag: Option<ColumnDrag>,
) -> HeaderLayout {
    let Measures {
        area_width,
        lead_px,
        pin_widths,
        pin_edges,
        mut warned,
    } = measures;
    let widths = state.column_widths.read();
    let mut headers = header_specs(
        &props.columns,
        &props.column_defaults,
        &state.hidden_columns.read(),
        WidthSpec {
            resizable: props.resizable_columns,
            widths: &widths,
            preview,
        },
    );
    let pinned = state.pinned_columns.read();
    let leads: Vec<String> = [
        has_reorder.then(reorder_width),
        has_detail.then(detail_width),
        props.selectable.then(|| select_width(size)),
    ]
    .into_iter()
    .flatten()
    .collect();
    let lead_width = match leads.as_slice() {
        [] => None,
        [one] => Some(one.clone()),
        many => Some(format!("calc({})", many.join(" + "))),
    };
    // The leading columns stay pinned with the start pins configured, however many the room releases.
    let lead_pins = has_start_pins(&headers, &pinned);
    let tracks_area = !(pinned.start.is_empty() && pinned.end.is_empty());
    let measured = pin_widths.read();
    let fitted = fit_pins(
        &headers,
        &pinned,
        &PinRoom {
            area: area_width(),
            lead: lead_px(),
            widths: &measured,
        },
    );
    let (mut layout, unknown) =
        pin_columns(&mut headers, &fitted, lead_width.as_deref(), &measured);
    let column_order = state.column_order.read();
    order_unpinned(&headers, &mut layout, &column_order);
    if let Some(drag) = drag {
        let plan = DropPlan {
            ranked: ranked(&headers, &column_order)
                .into_iter()
                .map(|index| headers[index].header.clone())
                .collect(),
            unpinned: layout
                .iter()
                .filter(|&&index| headers[index].pin.is_none())
                .map(|&index| (index, headers[index].header.clone()))
                .collect(),
        };
        let mut current = drag.plan;
        if *current.peek() != plan {
            current.set(plan);
        }
    }
    let pins_start = lead_pins;
    let pins_select = props.selectable && pins_start;
    let pins = pins_start || headers.iter().any(|spec| spec.pin.is_some());
    let edges = pin_edges();
    let mut pinned_pads = [(PinSide::Start, edges[0]), (PinSide::End, edges[1])]
        .map(|(side, edge)| pinned_extent(&headers, side, edge, &measured));
    // Released start pins leave the leading columns, which a focus scroll still clears.
    let start_released = pins_start
        && !headers.iter().any(|spec| {
            spec.pin
                .as_ref()
                .is_some_and(|pin| pin.side == PinSide::Start)
        });
    if start_released {
        pinned_pads[0] = lead_width.clone();
    }
    let lead_probe = lead_width.clone().filter(|_| lead_pins);
    if !unknown.is_empty() && *warned.peek() != unknown {
        warn(&format!(
            "Table: pinned columns {unknown:?} sit past a pinned column without `width`, so \
             their offset is unknown and they overlap it."
        ));
        warned.set(unknown);
    }
    let active = active_sort(&headers, &state.sort.read(), props.multi_sort);
    let group_rows = layout
        .iter()
        .map(|&index| headers[index].groups.len())
        .max()
        .unwrap_or(0);
    let head_rows = group_rows + 1 + usize::from(props.header_filters);
    HeaderLayout {
        headers,
        layout,
        widths,
        column_order,
        lead_width,
        tracks_area,
        pins_start,
        pins_select,
        pins,
        pinned_pads,
        lead_probe,
        active,
        group_rows,
        head_rows,
    }
}

/// What every column's filter shares.
#[derive(Clone, Copy)]
pub(super) struct FilterBase {
    pub state: TableState,
    pub labels: TableLabels,
    pub size: Size,
    pub onfilter: Callback<()>,
}

impl FilterBase {
    /// A `filterable` column's filter.
    pub fn target<T>(self, column: &Column<T>) -> Option<FilterTarget> {
        column.filterable.then(|| FilterTarget {
            column: column.header.clone(),
            kind: column.filter_kind,
            options: column.options.clone(),
            slice: self.state.column_filters,
            labels: self.labels,
            size: self.size,
            onfilter: self.onfilter,
        })
    }
}

/// What the column menus act on.
pub(super) struct MenuSpec {
    pub columns: Rc<[MenuColumn]>,
    pub resize: Option<ColumnResize>,
    pub filter: FilterBase,
    pub panel: PanelControl,
    pub focus: MenuFocus,
    pub announcer: Announcer,
}

/// One column menu per header, by header index.
pub(super) fn column_menus<T: Clone + PartialEq + 'static>(
    props: &TableProps<T>,
    header: &HeaderLayout,
    spec: MenuSpec,
) -> Vec<Element> {
    let HeaderLayout {
        headers,
        layout,
        widths,
        column_order,
        active,
        ..
    } = header;
    let MenuSpec {
        columns,
        resize,
        filter,
        panel,
        focus,
        announcer,
    } = spec;
    let FilterBase {
        state,
        labels,
        size,
        ..
    } = filter;
    let active = Rc::new(active.clone());
    (0..headers.len())
        .map(|index| {
            // The next shown column, else the one before.
            let at = layout.iter().position(|&shown| shown == index);
            let neighbour = at.and_then(|at| {
                layout
                    .get(at + 1)
                    .or_else(|| at.checked_sub(1).and_then(|before| layout.get(before)))
                    .copied()
            });
            rsx! {
                ColumnMenu {
                    index,
                    columns: columns.clone(),
                    // A move swaps it with the next shown unpinned column: one place over.
                    moves: (
                        moved(headers, column_order, index, false).zip(at),
                        moved(headers, column_order, index, true)
                            .zip(at)
                            .map(|(next, at)| (next, at + 2)),
                    ),
                    order: state.column_order,
                    width: resize.zip(headers[index].resize).map(|(resize, limits)| MenuWidth {
                        resize,
                        limits,
                        width: widths.get(&headers[index].header).copied(),
                        announcer,
                    }),
                    active: active.clone(),
                    sort: state.sort,
                    multi_sort: props.multi_sort,
                    hidden: state.hidden_columns,
                    pinned: state.pinned_columns,
                    grouping: state.row_grouping,
                    labels,
                    size,
                    parts: props.column_menu_parts.clone(),
                    filter: filter.target(&props.columns[index]),
                    panel: props.filter_panel.then_some(panel),
                    focus,
                    neighbour,
                    announcer,
                }
            }
        })
        .collect()
}

/// The header filters row's cells, by header index: `None` for a column without a filter.
pub(super) fn filter_cells<T>(columns: &[Column<T>], filter: FilterBase) -> Vec<Option<Element>> {
    columns
        .iter()
        .map(|column| {
            filter.target(column).map(|target| {
                rsx! {
                    HeaderFilter { key: "{target.column}", target }
                }
            })
        })
        .collect()
}
