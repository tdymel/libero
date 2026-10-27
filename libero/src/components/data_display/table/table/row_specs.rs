//! The body's rows from the shown order: the group row, each row's state, the grid's
//! callbacks and the rows drawn, whole or as a window.

use std::{collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use super::super::{
    cell_focus::{FocusTarget, GridRow, TreeNode},
    column::Column,
    core::{ActiveSort, HeaderSpec, UserSort, align_attr, next_sort},
    footer::aggregate_cell,
    grid::{GridFocus, GridWindow, TreeNav},
    pipeline::{GroupInfo, VisibleRow},
    row::{RowContext, RowState, TableRow},
    row_groups::{GroupCells, GroupRow, GroupValues},
    selection::{Selection, group_state},
    tree::TreeLevel,
    window::{BodyRows, RowFocus, RowWindow, window_attributes},
};
use crate::{components::common::attr, localization::labels::table::TableLabels};

/// A group row by its group's index, with the attributes its place asks for.
pub(super) type GroupRowFn = Rc<dyn Fn(usize, Vec<Attribute>) -> Element>;

/// A data row's state by its position among the shown rows and its `data` index.
pub(super) type RowStateFn = Rc<dyn Fn(usize, usize) -> RowState>;

pub(super) struct GroupSpec<'a, T> {
    pub groups: &'a Rc<[GroupInfo]>,
    pub values: Option<Vec<GroupValues>>,
    pub columns: &'a Rc<Vec<Column<T>>>,
    pub headers: &'a [HeaderSpec],
    pub layout: &'a [usize],
    pub keys: &'a Rc<[String]>,
    pub selected_rows: &'a Rc<HashSet<String>>,
    pub selection: Option<&'a Selection>,
    /// The leading select, detail and reorder columns a group row spans.
    pub leads: usize,
    pub labels: TableLabels,
    /// Opens or closes a group: its id, its label and the new state.
    pub toggle: Callback<(String, String, bool)>,
    /// Turns the rows of a group, by its id, on or off.
    pub select: Callback<(String, bool)>,
}

pub(super) fn group_row<T: 'static>(spec: GroupSpec<'_, T>) -> GroupRowFn {
    let GroupSpec {
        groups,
        values,
        columns,
        headers,
        layout,
        keys,
        selected_rows,
        selection,
        leads,
        labels,
        toggle,
        select,
    } = spec;
    let cells = GroupCells {
        leads,
        pins: layout
            .iter()
            .map(|&index| headers[index].pin.clone())
            .collect(),
        aligns: layout
            .iter()
            .map(|&index| align_attr(headers[index].align))
            .collect(),
        labels,
        toggle,
        select: selection.map(|selection| (selection.look.clone(), select)),
    };
    let (groups, columns, layout) = (groups.clone(), columns.clone(), layout.to_vec());
    let (keys, selected_rows) = (keys.clone(), selected_rows.clone());
    let selects = selection.is_some();
    Rc::new(move |at, attributes| {
        let group = &groups[at];
        let aggregates = values.as_ref().map(|values| {
            layout
                .iter()
                .map(|&index| aggregate_cell(&columns[index], values[at][index], &labels))
                .collect()
        });
        let selected = selects
            .then(|| group_state(&group.leaves, |index| selected_rows.contains(&keys[index])));
        rsx! {
            GroupRow {
                // Apart from the row keys, which are the caller's.
                key: "\u{1f}{group.id}",
                id: group.id.clone(),
                label: group.label.clone(),
                count: group.leaves.len(),
                depth: group.depth,
                open: group.open,
                aggregates: aggregates.unwrap_or_default(),
                selected,
                attributes,
                cells: cells.clone(),
            }
        }
    })
}

pub(super) struct StateSpec {
    pub key_at: Rc<dyn Fn(usize) -> String>,
    pub selects: bool,
    pub selected_rows: Rc<HashSet<String>>,
    pub mixed: Option<Rc<[bool]>>,
    pub expanded: Rc<HashSet<String>>,
    pub tree_levels: Option<Rc<[Option<TreeLevel>]>>,
    pub striped: bool,
    pub has_reorder: bool,
    pub stop: Option<FocusTarget>,
    pub editing: Option<FocusTarget>,
    /// With a paged grid: the rows on the pages before this one and on all of them.
    pub paged: Option<(usize, usize)>,
    pub head_rows: usize,
}

pub(super) fn row_state(spec: StateSpec) -> RowStateFn {
    let StateSpec {
        key_at,
        selects,
        selected_rows,
        mixed,
        expanded,
        tree_levels,
        striped,
        has_reorder,
        stop: grid_stop,
        editing,
        paged,
        head_rows,
    } = spec;
    Rc::new(move |position, index| {
        let key = key_at(index);
        RowState {
            index,
            stripe: striped && position % 2 == 1,
            slot: has_reorder.then_some(position),
            selected: selects.then(|| selected_rows.contains(&key)),
            mixed: mixed.as_ref().is_some_and(|mixed| mixed[index]),
            expanded: expanded.contains(&key),
            stop: grid_stop
                .filter(|stop| stop.row == GridRow::Body(VisibleRow::Data(index)))
                .and_then(|stop| stop.col),
            rowindex: paged.map(|(first, _)| head_rows + first + position + 1),
            editing: editing
                .filter(|at| at.row == GridRow::Body(VisibleRow::Data(index)))
                .and_then(|at| at.col),
            level: tree_levels.as_ref().and_then(|levels| levels[index]),
            row_stop: grid_stop.is_some_and(|stop| {
                stop.col.is_none() && stop.row == GridRow::Body(VisibleRow::Data(index))
            }),
            key,
        }
    })
}

pub(super) struct TreeSpec<'a, T> {
    pub data: &'a Rc<Vec<T>>,
    pub columns: &'a Rc<Vec<Column<T>>>,
    pub keys: &'a Rc<[String]>,
    pub parents: &'a Rc<[usize]>,
    pub levels: &'a Rc<[Option<TreeLevel>]>,
    pub order: &'a [VisibleRow],
    pub onrowclick: Option<EventHandler<T>>,
    pub name_column: usize,
    /// Opens or closes a row: its key, its name and the new state.
    pub toggle: Callback<(String, String, bool)>,
    pub cells: bool,
}

/// A treegrid's navigation: each shown row with its parent and whether it opens.
pub(super) fn tree_nav<T: Clone + 'static>(spec: TreeSpec<'_, T>) -> TreeNav {
    let TreeSpec {
        data,
        columns,
        keys,
        parents,
        levels,
        order,
        onrowclick,
        name_column,
        toggle,
        cells,
    } = spec;
    let places: std::collections::HashMap<usize, usize> = order
        .iter()
        .filter_map(|row| row.data())
        .enumerate()
        .map(|(place, index)| (index, place))
        .collect();
    let nodes = order
        .iter()
        .filter_map(|row| row.data())
        .map(|index| {
            let level = levels[index];
            TreeNode {
                parent: Some(parents[index])
                    .filter(|&up| up != index)
                    .and_then(|up| places.get(&up).copied()),
                open: level
                    .filter(|level| level.expandable)
                    .map(|level| level.open),
            }
        })
        .collect();
    let click = onrowclick.map(|onrowclick| {
        let data = data.clone();
        Callback::new(move |index: usize| onrowclick.call(data[index].clone()))
    });
    let (data, columns, keys) = (data.clone(), columns.clone(), keys.clone());
    TreeNav {
        click,
        nodes,
        parents: parents.clone(),
        toggle: Callback::new(move |(index, open): (usize, bool)| {
            let name = columns
                .get(name_column)
                .map(|column| (column.text)(&data[index]))
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| keys[index].clone());
            toggle.call((keys[index].clone(), name, open));
        }),
        cells,
    }
}

/// A header cell's Enter or Space (its sort) and Ctrl+Enter (its column menu, when `menus`).
pub(super) fn header_action(
    headers: Rc<Vec<HeaderSpec>>,
    active: ActiveSort,
    sort: UserSort,
    open_menu: impl Fn(usize) + 'static,
    menus: bool,
) -> Callback<(usize, bool)> {
    Callback::new(move |(index, menu): (usize, bool)| match menu {
        false if headers[index].sortable => {
            sort.set(next_sort(&active, &headers, index, false));
        }
        true if menus => open_menu(index),
        _ => {}
    })
}

/// Shift+Space on a grid row: toggles its box, mixed or unselected on, selected off.
pub(super) fn select_action(
    key_at: Rc<dyn Fn(usize) -> String>,
    selected_rows: Rc<HashSet<String>>,
    mixed: Option<Rc<[bool]>>,
    toggle: Callback<(String, bool)>,
) -> Callback<usize> {
    Callback::new(move |index: usize| {
        let key = key_at(index);
        let mixed = mixed.as_ref().is_some_and(|mixed| mixed[index]);
        let on = mixed || !selected_rows.contains(&key);
        toggle.call((key, on));
    })
}

/// A windowed grid's pitch and where each grid row sits in the window.
pub(super) fn grid_window(
    pitch: f64,
    focused: Signal<Option<VisibleRow>>,
    order: &[VisibleRow],
) -> GridWindow {
    GridWindow {
        pitch,
        focused,
        slots: order
            .iter()
            .enumerate()
            .filter(|(_, row)| row.data().is_some())
            .map(|(slot, _)| slot)
            .collect(),
    }
}

pub(super) struct RowsSpec<'a, T: 'static> {
    pub order: Vec<VisibleRow>,
    pub data: Rc<Vec<T>>,
    pub groups: Rc<[GroupInfo]>,
    pub context: Rc<RowContext<T>>,
    pub generation: u64,
    pub key_at: Rc<dyn Fn(usize) -> String>,
    pub state_of: RowStateFn,
    pub group_row: GroupRowFn,
    /// The windowed body's pitch, `None` for every row drawn.
    pub row_height: Option<f64>,
    /// The rows before this page and every row, counted in `aria-rowindex`.
    pub counted: (usize, usize),
    /// With a paged grid: the same counts.
    pub paged: Option<(usize, usize)>,
    pub head_rows: usize,
    pub tree: bool,
    /// Whether the table is busy or has a footer, which the window's attributes tell.
    pub skeleton: bool,
    pub footer: bool,
    pub stop: Option<FocusTarget>,
    pub grid_focus: GridFocus,
    pub row_focus: RowFocus,
    pub attributes: &'a mut Vec<Attribute>,
}

/// The rows in the body: all drawn, or a window projecting those in view.
pub(super) fn body_rows<T: Clone + PartialEq + 'static>(spec: RowsSpec<'_, T>) -> BodyRows {
    let RowsSpec {
        order,
        data,
        groups,
        context,
        generation,
        key_at,
        state_of,
        group_row,
        row_height,
        counted,
        paged,
        head_rows,
        tree,
        skeleton,
        footer,
        stop: grid_stop,
        grid_focus,
        row_focus,
        attributes,
    } = spec;
    match row_height {
        Some(row_height) => {
            let (first, total) = counted;
            attributes.extend(window_attributes(head_rows, total, skeleton, footer));
            let order: Rc<[VisibleRow]> = order.into();
            row_focus.show(head_rows + first, order.clone());
            // Linus (TR3): a collapse above the focused row leaves it where it was.
            // A tree's stop row follows a collapse to its parent; another table's focused row.
            let focused = match tree {
                true => grid_stop
                    .filter(|_| grid_focus.inside())
                    .and_then(|stop| match stop.row {
                        GridRow::Body(row) => Some(row),
                        GridRow::Head(_) => None,
                    }),
                false => *row_focus.focused.peek(),
            };
            grid_focus.hold_in_view(focused, order.clone(), row_height);
            BodyRows::Window(RowWindow {
                order,
                first,
                total,
                row_height,
                row: Rc::new(move |position, index| {
                    let row = &data[index];
                    let state = state_of(position, index);
                    let cells = context.cells(
                        row,
                        index,
                        &state.key,
                        state.stop,
                        state.editing,
                        state.level,
                    );
                    context.spec(row, state, cells)
                }),
                group: group_row,
                key: Rc::new(move |row| match row {
                    VisibleRow::Data(index) => key_at(index),
                    VisibleRow::Group(at) => format!("\u{1f}{}", groups[at].id),
                }),
                focused: row_focus.focused,
                moves: row_focus.moves,
                reordering: row_focus.reordering,
            })
        }
        None => BodyRows::All(
            order
                .into_iter()
                .enumerate()
                .map(|(position, row)| match row {
                    VisibleRow::Data(index) => {
                        let state = state_of(position, index);
                        rsx! {
                            TableRow::<T> {
                                key: "{state.key}",
                                row: data[index].clone(),
                                state,
                                context: context.clone(),
                                generation,
                            }
                        }
                    }
                    VisibleRow::Group(at) => group_row(
                        at,
                        paged
                            .map(|(first, _)| {
                                let at = head_rows + first + position + 1;
                                vec![attr("aria-rowindex", at.to_string())]
                            })
                            .unwrap_or_default(),
                    ),
                })
                .collect(),
        ),
    }
}
