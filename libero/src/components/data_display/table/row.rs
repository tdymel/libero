use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    column::Column,
    core::{CellSpec, HeaderSpec, RowFn, RowSpec, body_rows, render_cells},
    detail::Details,
    groups::spanned,
    pinning::{pin_runs, span_pin},
    row_reorder::{ReorderSlot, RowReorder},
    selection::Selection,
};
use crate::{components::common::States, hooks::listener};

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
    pub onrowclick: Option<EventHandler<T>>,
    pub selection: Option<Selection>,
    pub toggle: Callback<(String, bool)>,
    pub details: Option<Details>,
    pub toggle_detail: Callback<(String, bool)>,
    pub animate_details: bool,
    pub reorder: Option<RowReorder>,
}

impl<T> RowContext<T> {
    /// Whether rows drawn with `other` come out the same. Closures compare equal, as a
    /// column's do: a re-declared one does not redraw every row.
    pub fn same(&self, other: &Self) -> bool {
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
        self.columns == other.columns
            && self.layout == other.layout
            && self.span == other.span
            && self.name_column == other.name_column
            && self.row_attrs == other.row_attrs
            && self.row_states == other.row_states
            && self.row_detail == other.row_detail
            && self.row_has_detail == other.row_has_detail
            && self.onrowclick == other.onrowclick
            && self.toggle == other.toggle
            && self.details == other.details
            && self.toggle_detail == other.toggle_detail
            && self.animate_details == other.animate_details
            && cells(self) == cells(other)
            && boxes(self) == boxes(other)
            && reorder(self) == reorder(other)
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
        } = state;
        let columns = &self.columns;
        let name = || {
            columns
                .get(self.name_column)
                .map(|column| (column.text)(row))
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| key.clone())
        };
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
            .map(|details| details.row_cell(key.clone(), &name(), index, open, self.toggle_detail));
        let sliding = details
            .zip(open)
            .filter(|_| self.animate_details)
            .map(|(details, open)| (details.row_id(index), open));
        let detail = match (details, open) {
            (Some(details), Some(true)) => detail.map(|body| (details.row_id(index), body)),
            _ => None,
        };
        let mut attributes = self.row_attrs.call(row).unwrap_or_default();
        if let Some(onrowclick) = self.onrowclick {
            let row = row.clone();
            attributes.push(listener("onclick", move |_: Event<MouseData>| {
                onrowclick.call(row.clone());
            }));
        }
        let select = self.selection.as_ref().map(|selection| {
            selection.row_cell(key.clone(), &name(), selected == Some(true), self.toggle)
        });
        RowSpec {
            reorder: slot.map(|slot| ReorderSlot {
                slot,
                label: name(),
            }),
            selected,
            select,
            stripe,
            toggle,
            detail,
            sliding,
            states: self
                .row_states
                .call(row)
                .and_then(|states| states.data_state()),
            attributes,
            cells,
            key,
        }
    }

    /// The row's cells, in display order.
    pub fn cells(&self, row: &T) -> Element {
        let (columns, headers) = (&self.columns, &self.headers);
        let cells = pin_runs(headers, &self.layout)
            .flat_map(|run| {
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
            .map(|(index, span, covered)| {
                let column = &columns[index];
                let (text, body) = match &column.render {
                    Some(render) => (String::new(), Some(render(row))),
                    None => ((column.text)(row), None),
                };
                CellSpec {
                    column: index,
                    text,
                    body,
                    span,
                    pin: span_pin(headers, covered),
                }
            });
        render_cells(cells, headers)
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
        RowCells::<T> { row: row.clone(), context: context.clone(), generation }
    };
    let spec = context.spec(&row, state, cells);
    rsx! {
        {body_rows(spec, context.span, context.reorder.as_ref())}
    }
}

#[derive(Props, Clone)]
struct RowCellsProps<T: Clone + PartialEq + 'static> {
    row: T,
    context: Rc<RowContext<T>>,
    generation: u64,
}

impl<T: Clone + PartialEq + 'static> PartialEq for RowCellsProps<T> {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation && self.row == other.row
    }
}

/// A row's cells, a scope of their own: a selection, detail toggle or sort redraws
/// the row around them only.
#[component]
fn RowCells<T: Clone + PartialEq + 'static>(props: RowCellsProps<T>) -> Element {
    props.context.cells(&props.row)
}
