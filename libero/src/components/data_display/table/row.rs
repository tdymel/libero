use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    cell_focus::{FocusTarget, GridRow},
    cell_value::EditKind,
    column::Column,
    core::{CellSpec, DetailSpec, HeaderSpec, RowFn, RowSpec, body_rows, render_cells},
    detail::Details,
    edit::{CellEditor, EditCtx, EditError, Leave, RowUpdate, commit_edit},
    grid::GridFocus,
    header_groups::spanned,
    pinning::{pin_runs, span_pin},
    pipeline::VisibleRow,
    row_reorder::{ReorderSlot, RowReorder},
    selection::Selection,
    tree::TreeLevel,
};
use crate::{
    components::{
        accessibility::Announcer,
        common::{Glyph, States, attr},
    },
    context::IconSlot,
    hooks::listener,
    localization::labels::table::TableLabels,
    platform::infers_grid_cells,
    theme::TABLE_GROUP_INDENT,
};
use pictogram_icons_lucide as lucide;

/// What a body row reads besides its own row: the columns, the row props and the
/// leading columns. The same for every row of one table render.
pub(super) struct RowContext<T: 'static> {
    pub columns: Rc<Vec<Column<T>>>,
    pub headers: Rc<Vec<HeaderSpec>>,
    /// The shown headers in display order.
    pub layout: Vec<usize>,
    /// Every column a detail row spans, the leading ones too.
    pub span: usize,
    /// The column whose text names a row's checkbox and toggle.
    pub name_column: usize,
    pub row_attrs: RowFn<T, Vec<Attribute>>,
    pub row_states: RowFn<T, States>,
    pub row_detail: RowFn<T, Option<Element>>,
    pub row_has_detail: RowFn<T, bool>,
    /// With `parent_key`: opens or closes a tree row, by key and name.
    pub toggle_node: Option<Callback<(String, String, bool)>>,
    pub onrowclick: Option<EventHandler<T>>,
    pub selection: Option<Selection>,
    pub toggle: Callback<(String, bool)>,
    pub details: Option<Details>,
    pub toggle_detail: Callback<(String, bool)>,
    pub animate_details: bool,
    pub reorder: Option<RowReorder>,
    /// With `cell_focus`.
    pub grid: Option<GridFocus>,
    /// With `parent_key`: each row a treegrid stop.
    pub tree_focus: Option<GridFocus>,
    /// With `cell_focus` and a hidden column: each column's `aria-colindex`.
    pub colindex: Option<Rc<[usize]>>,
    /// Told an editable cell's commit.
    pub onrowupdate: Option<EventHandler<RowUpdate<T>>>,
    /// Whether a column is editable: the other cells are then read-only.
    pub editable: bool,
    pub announcer: Announcer,
    pub labels: TableLabels,
}

impl<T> RowContext<T> {
    /// Whether rows drawn with `other` come out the same. Closures compare equal, as a
    /// column's do: a re-declared one does not redraw every row.
    pub fn same(&self, other: &Self) -> bool {
        // Every field is named: a new one fails to compile until it is compared or skipped.
        let Self {
            columns,
            headers: _,
            layout,
            span,
            name_column,
            row_attrs,
            row_states,
            row_detail,
            row_has_detail,
            toggle_node,
            onrowclick,
            selection: _,
            toggle,
            details,
            toggle_detail,
            animate_details,
            reorder: _,
            grid,
            tree_focus,
            colindex,
            onrowupdate,
            editable,
            announcer,
            labels,
        } = self;
        let cells = |context: &Self| {
            context
                .headers
                .iter()
                .map(|spec| (spec.row_header, spec.align, spec.pin.clone()))
                .collect::<Vec<_>>()
        };
        let boxes = |context: &Self| {
            context
                .selection
                .as_ref()
                .map(|selection| (selection.look.clone(), selection.labels))
        };
        let reorder = |context: &Self| {
            context.reorder.as_ref().map(|reorder| {
                (
                    reorder.disabled,
                    reorder.instructions.clone(),
                    reorder.fixed.map(|slots| slots.pitch),
                )
            })
        };
        *columns == other.columns
            && *layout == other.layout
            && *span == other.span
            && *name_column == other.name_column
            && *row_attrs == other.row_attrs
            && *row_states == other.row_states
            && *row_detail == other.row_detail
            && *row_has_detail == other.row_has_detail
            && *toggle_node == other.toggle_node
            && *onrowclick == other.onrowclick
            && *toggle == other.toggle
            && *details == other.details
            && *toggle_detail == other.toggle_detail
            && *animate_details == other.animate_details
            && cells(self) == cells(other)
            && boxes(self) == boxes(other)
            && reorder(self) == reorder(other)
            && grid.map(|grid| grid.stop) == other.grid.map(|grid| grid.stop)
            && tree_focus.map(|grid| grid.stop) == other.tree_focus.map(|grid| grid.stop)
            && *colindex == other.colindex
            && *onrowupdate == other.onrowupdate
            && *editable == other.editable
            && *announcer == other.announcer
            && *labels == other.labels
    }
}

/// One body row's own state, beside the table's [`RowContext`].
#[derive(Clone, PartialEq)]
pub(super) struct RowState {
    /// `data`'s index, which names its detail row.
    pub index: usize,
    pub key: String,
    pub stripe: bool,
    /// With `onrowreorder`: its slot among the shown rows.
    pub slot: Option<usize>,
    /// With `selectable`.
    pub selected: Option<bool>,
    /// Its key is among the open details.
    pub expanded: bool,
    /// With `cell_focus`, in the stop's row: the stop's header index.
    pub stop: Option<usize>,
    /// With `cell_focus` on a page: its `aria-rowindex`.
    pub rowindex: Option<usize>,
    /// The header index of its cell whose editor is open.
    pub editing: Option<usize>,
    /// With `parent_key`: its place in the tree.
    pub level: Option<TreeLevel>,
    /// With `parent_key`: the row itself is the stop.
    pub row_stop: bool,
}

impl<T: Clone + 'static> RowContext<T> {
    /// `cells` as [`Self::cells`] draws them.
    pub fn spec(&self, row: &T, state: RowState, cells: Element) -> RowSpec {
        let RowState {
            index,
            key,
            stripe,
            slot,
            selected,
            expanded,
            stop: _,
            rowindex,
            editing: _,
            level,
            row_stop,
        } = state;
        let named = self.details.is_some() || self.selection.is_some() || slot.is_some();
        let name = named.then(|| {
            self.columns
                .get(self.name_column)
                .map(|column| (column.text)(row))
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| key.clone())
        });
        let name = name.as_deref().unwrap_or_default();
        let (open, detail) = match self.row_has_detail.call(row) {
            Some(has) => {
                let open = has.then_some(expanded);
                let detail = match open {
                    Some(true) => self.row_detail.call(row).flatten(),
                    _ => None,
                };
                (open, detail)
            }
            None => {
                let detail = self.row_detail.call(row).flatten();
                (detail.as_ref().map(|_| expanded), detail)
            }
        };
        let details = self.details.as_ref();
        let toggle = details
            .map(|details| details.row_cell(key.clone(), name, index, open, self.toggle_detail));
        let detail = details.zip(open).map(|(details, open)| DetailSpec {
            id: details.row_id(index),
            open,
            body: detail.filter(|_| open),
        });
        let mut attributes = self.row_attrs.call(row).unwrap_or_default();
        if let Some(rowindex) = rowindex {
            attributes.push(attr("aria-rowindex", rowindex.to_string()));
        }
        if let Some(level) = level {
            attributes.push(attr("data-level", level.depth.to_string()));
            attributes.extend([
                attr("aria-level", (level.depth + 1).to_string()),
                attr("aria-posinset", level.posinset.to_string()),
                attr("aria-setsize", level.setsize.to_string()),
            ]);
            if level.children > 0 {
                attributes.push(attr("aria-expanded", level.open.to_string()));
            }
            if let Some(grid) = &self.tree_focus {
                attributes.extend(grid.row(index, row_stop));
            }
        }
        if let Some(onrowclick) = self.onrowclick {
            let row = row.clone();
            attributes.push(listener("onclick", move |_: Event<MouseData>| {
                onrowclick.call(row.clone());
            }));
        }
        let select = self.selection.as_ref().map(|selection| {
            selection.row_cell(key.clone(), name, selected == Some(true), self.toggle)
        });
        RowSpec {
            span: self.span,
            reorder: slot.map(|slot| ReorderSlot {
                slot,
                label: name.to_string(),
            }),
            selected,
            select,
            stripe,
            toggle,
            detail,
            sliding: self.animate_details,
            states: self
                .row_states
                .call(row)
                .and_then(|states| states.data_state()),
            attributes,
            cells,
            key,
        }
    }

    /// The row's cells as `(header index, colspan, covered shown columns)`, in display order.
    fn layout_of<'a>(&'a self, row: &'a T) -> impl Iterator<Item = (usize, usize, &'a [usize])> {
        let (columns, headers) = (&self.columns, &self.headers);
        pin_runs(headers, &self.layout).flat_map(move |run| {
            let mut at = 0;
            spanned(run, |index| {
                columns[index].col_span.as_ref().map_or(1, |span| span(row))
            })
            .into_iter()
            .map(move |(index, span)| {
                let covered = &run[at..at + span];
                at += span;
                (index, span, covered)
            })
        })
    }

    /// The widths of the row's cells, in shown columns.
    pub fn spans(&self, row: &T) -> Vec<usize> {
        self.layout_of(row).map(|(_, span, _)| span).collect()
    }

    /// The row's cells, in display order; `data[index]` is `row`, keyed `key`. With
    /// `cell_focus` the cell of header `stop` is the tab stop, and `editing`'s
    /// holds its editor. A tree row's `level` indents its first cell.
    pub fn cells(
        &self,
        row: &T,
        index: usize,
        key: &str,
        stop: Option<usize>,
        editing: Option<usize>,
        level: Option<TreeLevel>,
    ) -> Element {
        let (columns, headers) = (&self.columns, &self.headers);
        let cells = self
            .layout_of(row)
            .enumerate()
            .map(|(position, (column, span, covered))| {
                let spec = &columns[column];
                let (text, body) = match (&spec.render, self.grid) {
                    (_, Some(grid)) if editing == Some(column) && spec.edit.is_some() => (
                        String::new(),
                        Some(self.editor(row, index, key, column, grid)),
                    ),
                    (Some(render), _) => (String::new(), Some(render(row))),
                    (None, _) => ((spec.text)(row), None),
                };
                let (text, body) = match level.filter(|_| position == 0) {
                    Some(level) => {
                        let lead = self.node_lead(row, key, level);
                        // Unpadded block: Blitz misses an inline box in the padded cell (hit.rs).
                        let body = rsx! {
                            div { "data-node-cell": true, {lead} "{text}" {body} }
                        };
                        (String::new(), Some(body))
                    }
                    None => (text, body),
                };
                let mut attributes = match &self.grid {
                    Some(grid) => grid.cell(
                        GridRow::Body(VisibleRow::Data(index)),
                        column,
                        stop == Some(column),
                        headers[column].row_header,
                        self.colindex.as_ref().map(|indices| indices[column]),
                    ),
                    None if self.tree_focus.is_some()
                        && !headers[column].row_header
                        && !infers_grid_cells() =>
                    {
                        vec![attr("role", "gridcell")]
                    }
                    None => Vec::new(),
                };
                match (self.grid, spec.edit.is_some()) {
                    (Some(grid), true) => {
                        let target = FocusTarget {
                            row: GridRow::Body(VisibleRow::Data(index)),
                            col: Some(column),
                        };
                        attributes.push(listener("ondblclick", move |_: Event<MouseData>| {
                            grid.start(target);
                        }));
                        attributes.push(listener(
                            "onpointerdown",
                            move |event: Event<PointerData>| {
                                grid.touched(target, event.pointer_type() == "touch");
                            },
                        ));
                        attributes.push(listener("onclick", move |_: Event<MouseData>| {
                            grid.tapped(target);
                        }));
                        if editing == Some(column) {
                            attributes.push(attr("data-editing", "true"));
                        }
                    }
                    (Some(_), false) if self.editable => {
                        attributes.push(attr("aria-readonly", "true"));
                    }
                    _ => {}
                }
                CellSpec {
                    column,
                    text,
                    body,
                    span,
                    pin: span_pin(headers, covered),
                    attributes,
                }
            });
        render_cells(cells, headers)
    }

    /// A tree row's indent and its toggle, or a spacer the toggle's width for a leaf.
    fn node_lead(&self, row: &T, key: &str, level: TreeLevel) -> Element {
        let indent = format!(
            "margin-inline-start:calc({} * {})",
            level.depth,
            TABLE_GROUP_INDENT.value()
        );
        let Some(toggle) = self.toggle_node.filter(|_| level.children > 0) else {
            return rsx! {
                span { "data-node-spacer": true, style: indent }
            };
        };
        let name = self
            .columns
            .get(self.name_column)
            .map(|column| (column.text)(row))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| key.to_string());
        // State-neutral, as a group toggle's: `aria-expanded` tells open from closed.
        let label = (self.labels.group_toggle)(&name, level.children);
        let (key, open) = (key.to_string(), level.open);
        rsx! {
            button {
                r#type: "button",
                "data-node-toggle": true,
                // The treegrid row's arrows open and close it: no Tab stop in every row.
                tabindex: "-1",
                aria_label: label,
                aria_expanded: "{open}",
                style: indent,
                // Not a row click.
                onclick: move |event: MouseEvent| {
                    event.stop_propagation();
                    toggle((key.clone(), name.clone(), !open));
                },
                Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
            }
        }
    }

    /// The editor of `row`'s cell of header `column`: the built-in one, or the column's.
    fn editor(&self, row: &T, index: usize, key: &str, column: usize, grid: GridFocus) -> Element {
        let spec = &self.columns[column];
        let value = (spec.value_text)(row);
        let name = self
            .columns
            .get(self.name_column)
            .map(|named| (named.text)(row))
            .unwrap_or_else(|| key.to_string());
        let label = (self.labels.edit_cell)(&spec.header, &name);
        let commit = {
            let (row, columns, key) = (row.clone(), self.columns.clone(), key.to_string());
            let (onrowupdate, announcer, labels) = (self.onrowupdate, self.announcer, self.labels);
            Callback::new(move |(text, leave): (String, Leave)| {
                match commit_edit(&row, &columns[column], &text) {
                    Ok(next) => {
                        if let (Some(next), Some(onrowupdate)) = (next, onrowupdate) {
                            onrowupdate.call(RowUpdate {
                                key: key.clone(),
                                index,
                                column: columns[column].header.clone(),
                                row: next,
                            });
                        }
                        grid.finish(leave);
                        true
                    }
                    Err(error) => {
                        let message = match error {
                            EditError::Parse => labels.invalid_value.to_string(),
                            EditError::Invalid(message) => message,
                        };
                        let header = &columns[column].header;
                        if leave == Leave::Blur {
                            announcer.say((labels.edit_reverted)(header, &message));
                            grid.finish(Leave::Blur);
                            return true;
                        }
                        announcer.say((labels.edit_invalid)(header, &message));
                        let mut invalid = grid.invalid;
                        invalid.set(Some(message));
                        false
                    }
                }
            })
        };
        let cancel = Callback::new(move |()| grid.finish(Leave::Stay));
        let error = (grid.invalid)();
        let error_id = grid.error_id();
        let changed = Callback::new(move |()| {
            let mut invalid = grid.invalid;
            invalid.set(None);
        });
        let kind = spec.edit.as_ref().map(|edit| edit.kind).unwrap_or_default();
        let hint = match kind {
            EditKind::Bool => Some(self.labels.edit_hint_bool),
            EditKind::Date => Some(self.labels.edit_hint_date),
            _ => None,
        };
        let custom = spec.edit.as_ref().and_then(|edit| edit.editor.clone());
        let editor = match custom {
            Some(editor) => editor(EditCtx {
                value,
                label,
                error: error.clone(),
                error_id: error_id.clone(),
                commit: Callback::new(move |text: String| {
                    commit.call((text, Leave::Stay));
                }),
                changed,
                cancel,
            }),
            None => rsx! {
                CellEditor {
                    value,
                    label,
                    kind,
                    cancel_label: self.labels.cancel_edit,
                    error: error.clone(),
                    error_id: error_id.clone(),
                    hint,
                    commit,
                    cancel,
                    changed,
                }
            },
        };
        rsx! {
            {editor}
            if let Some(message) = error {
                div { id: error_id, "data-cell-error": true, "{message}" }
            }
        }
    }
}

#[derive(Props, Clone)]
pub(super) struct TableRowProps<T: Clone + PartialEq + 'static> {
    row: T,
    state: RowState,
    context: Rc<RowContext<T>>,
    /// Bumped when the context changes, which then counts in place of it.
    generation: u64,
}

impl<T: Clone + PartialEq + 'static> PartialEq for TableRowProps<T> {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation && self.state == other.state && self.row == other.row
    }
}

/// A body row and its detail row, its own scope: a table render that leaves a row,
/// its state and the context as they were skips it (todo 1981).
#[component]
pub(super) fn TableRow<T: Clone + PartialEq + 'static>(props: TableRowProps<T>) -> Element {
    let TableRowProps {
        row,
        state,
        context,
        generation,
    } = props;
    let cells = rsx! {
        RowCells::<T> {
            row: row.clone(),
            index: state.index,
            row_key: state.key.clone(),
            stop: state.stop,
            editing: state.editing,
            level: state.level,
            context: context.clone(),
            generation,
        }
    };
    let spec = context.spec(&row, state, cells);
    rsx! {
        {body_rows(spec, context.reorder.as_ref())}
    }
}

#[derive(Props, Clone)]
struct RowCellsProps<T: Clone + PartialEq + 'static> {
    row: T,
    index: usize,
    row_key: String,
    stop: Option<usize>,
    editing: Option<usize>,
    level: Option<TreeLevel>,
    context: Rc<RowContext<T>>,
    generation: u64,
}

impl<T: Clone + PartialEq + 'static> PartialEq for RowCellsProps<T> {
    fn eq(&self, other: &Self) -> bool {
        // The index and key only name a grid cell's focus target and edit.
        self.generation == other.generation
            && self.row == other.row
            && self.stop == other.stop
            && self.editing == other.editing
            && self.level == other.level
            && (self.context.grid.is_none()
                || (self.index == other.index && self.row_key == other.row_key))
    }
}

/// A row's cells, a scope of their own: a selection, detail toggle or sort redraws
/// the row around them only.
#[component]
fn RowCells<T: Clone + PartialEq + 'static>(props: RowCellsProps<T>) -> Element {
    let RowCellsProps {
        row,
        index,
        row_key,
        stop,
        editing,
        level,
        context,
        ..
    } = &props;
    context.cells(row, *index, row_key, *stop, *editing, *level)
}
