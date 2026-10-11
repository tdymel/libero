//! `cell_focus`: one body cell is the tab stop, and the keys of
//! [`cell_focus`](super::cell_focus) move it.

use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    cell_focus::{FocusTarget, GridRow, HeadCell, Move, Pos, key_move, resolve, step},
    pipeline::VisibleRow,
    window::reveal_slot,
};
use crate::{
    components::common::attr,
    hooks::{ElementHandle, listener, use_id, use_silent_focus_in},
    platform::{
        ElementApi, PlatformError, arrow_target, focus_selector, infers_grid_cells, key_taken,
        rtl_target, silent_focus, typing_target, when_laid_out,
    },
};

/// Rows a PageUp or PageDown moves when the view cannot be measured.
const PAGE_ROWS: usize = 10;

/// Focus in a grid cell or a widget inside one: the filter row, the footer and the
/// leading select, detail and reorder cells keep their keys.
const GRID_CELL_FOCUS: &str = "thead th[tabindex]:focus, thead th[tabindex] :focus, \
                               tbody td[tabindex]:focus, tbody th[tabindex]:focus, \
                               tbody td[tabindex] :focus, tbody th[tabindex] :focus";

/// A cell's `"{data index}.{header index}"`, a header cell's `"h{level}.{header
/// index}"`, on Blitz only: its click, Tab and `focus()` fire no `focusin`, so the
/// silent move finds the cell by it.
const CELL_ATTR: &str = "data-cell";

/// The grid's tab stop and what moves focus to it.
#[derive(Clone, Copy)]
pub(super) struct GridFocus {
    /// Kept while focus is away, so Tab comes back to the same cell.
    pub stop: Signal<Option<FocusTarget>>,
    /// The body cell whose editor is open, with an editable column.
    pub editing: Signal<Option<FocusTarget>>,
    /// Focus the stop after the next render, revealing its slot in a window.
    request: Signal<Option<Option<Reveal>>>,
    /// Marks the stop cell as `data-cell-stop`, where a WebView focuses by selector.
    id: Signal<String>,
    table: ElementHandle,
    region: ElementHandle,
}

#[derive(Clone, Copy, PartialEq)]
struct Reveal {
    slot: usize,
    pitch: f64,
    edges: (f64, f64),
}

/// A windowed body: the row pitch, the sticky header and footer heights, and the
/// focused row the window keeps rendered.
pub(super) struct GridWindow {
    pub pitch: f64,
    pub edges: (f64, f64),
    pub focused: Signal<Option<VisibleRow>>,
}

/// What a key press reads: the header levels, then the shown rows; the shown
/// headers in display order and each row's cell widths.
pub(super) struct GridRows {
    pub order: Rc<[GridRow]>,
    pub layout: Vec<usize>,
    pub head: Rc<Vec<Vec<HeadCell>>>,
    pub spans: Rc<dyn Fn(VisibleRow) -> Vec<usize>>,
    pub window: Option<GridWindow>,
    /// Enter or Space on a header cell, by header index: its sort; Ctrl+Enter
    /// (`true`): its column menu.
    pub header: Callback<(usize, bool)>,
    /// With `selectable`: Shift+Space toggles the row, by data index.
    pub select: Option<Callback<usize>>,
    /// Whether each column is editable, by header index: Enter or F2 opens its editor.
    pub editable: Rc<[bool]>,
}

impl GridRows {
    fn row_spans(&self, row: usize) -> Vec<usize> {
        match self.order[row] {
            GridRow::Head(level) => self.head[level].iter().map(|cell| cell.span).collect(),
            GridRow::Body(row) => (self.spans)(row),
        }
    }

    /// The stop's cell, by position.
    pub fn at(&self, stop: Option<FocusTarget>) -> Option<Pos> {
        resolve(stop, &self.order, &self.layout, |row| self.row_spans(row))
    }

    /// The cell at `at`: a header cell by the level it starts at.
    pub fn target(&self, at: Pos) -> FocusTarget {
        match self.order[at.row] {
            GridRow::Head(level) => {
                let cell = self.head[level]
                    .iter()
                    .find(|cell| cell.start == at.col)
                    .copied()
                    .unwrap_or(HeadCell {
                        start: at.col,
                        span: 1,
                        level,
                    });
                FocusTarget {
                    row: GridRow::Head(cell.level),
                    col: Some(self.layout[cell.start]),
                }
            }
            row => FocusTarget {
                row,
                col: Some(self.layout[at.col]),
            },
        }
    }
}

/// The header cells' share of the grid: the stop, and with a hidden column each
/// column's `aria-colindex` by header index.
pub(super) struct HeadGrid {
    pub focus: GridFocus,
    pub stop: Option<FocusTarget>,
    pub colindex: Option<Rc<[usize]>>,
}

impl HeadGrid {
    /// The attributes of the header cell at `level` whose first column is `col`.
    pub fn cell(&self, level: usize, col: usize) -> Vec<Attribute> {
        let row = GridRow::Head(level);
        let is_stop = self.stop
            == Some(FocusTarget {
                row,
                col: Some(col),
            });
        let colindex = self.colindex.as_ref().map(|indices| indices[col]);
        self.focus.cell(row, col, is_stop, true, colindex)
    }
}

/// Each column's `aria-colindex` by header index, after `leads` leading columns: a
/// hidden one keeps a place after the shown column before it in source order.
pub(super) fn col_indices(layout: &[usize], columns: usize, leads: usize) -> Rc<[usize]> {
    // Hidden columns placed before each shown position.
    let mut gaps = vec![0; layout.len() + 1];
    for hidden in (0..columns).filter(|index| !layout.contains(index)) {
        let after = layout
            .iter()
            .enumerate()
            .filter(|&(_, &shown)| shown < hidden)
            .max_by_key(|&(_, &shown)| shown)
            .map_or(0, |(position, _)| position + 1);
        gaps[after] += 1;
    }
    let mut indices = vec![0; columns];
    let mut gap = 0;
    for (position, &index) in layout.iter().enumerate() {
        gap += gaps[position];
        indices[index] = leads + position + gap + 1;
    }
    indices.into()
}

/// Called every render, `cell_focus` or not, for a stable hook order.
pub(super) fn use_grid_focus(table: ElementHandle, region: ElementHandle) -> GridFocus {
    let stop = use_signal(|| None);
    let editing = use_signal(|| None);
    use_silent_focus_in(table, move || {
        let mut stop = stop;
        if let Some(target) = focused_cell(&table)
            && *stop.peek() != Some(target)
        {
            stop.set(Some(target));
        }
    });
    let id = use_id();
    let mut request = use_signal(|| None::<Option<Reveal>>);
    use_effect(move || {
        let Some(reveal) = request() else {
            return;
        };
        request.set(None);
        let selector = format!("[data-cell-stop='{}']", id.peek());
        let focus = move || match table.query_selector(&selector) {
            Ok(cell) => {
                let _ = cell.focus();
            }
            // A WebView queries nothing: the page focuses it.
            Err(_) => {
                let _ = focus_selector(&selector);
            }
        };
        let Some(Reveal { slot, pitch, edges }) = reveal else {
            focus();
            return;
        };
        // Once the window's new rows are laid out: Blitz lays out after effects.
        when_laid_out(move || {
            focus();
            // Blitz scrolls nothing into view on a `focus()`.
            let (offset, view) = (region.scroll_offset(), region.dimensions());
            spawn(async move {
                if let (Ok((x, top)), Ok(view)) = (offset.await, view.await)
                    && let Some(top) = reveal_slot(slot, pitch, edges, top, view.height)
                {
                    let _ = region.scroll_to(x, top);
                }
            });
        });
    });
    GridFocus {
        stop,
        editing,
        request,
        id,
        table,
        region,
    }
}

/// The body cell holding focus or a widget with it, by its [`CELL_ATTR`].
fn focused_cell(table: &ElementHandle) -> Option<FocusTarget> {
    let holds =
        |element: &dyn ElementApi| element.is_focused() || element.query_selector(":focus").is_ok();
    let row = table
        .query_selector_all("thead > tr, tbody > tr")
        .ok()?
        .into_iter()
        .find(|row| holds(row.as_ref()))?;
    let cell = row
        .query_selector_all(&format!("[{CELL_ATTR}]"))
        .ok()?
        .into_iter()
        .find(|cell| holds(cell.as_ref()))?;
    let name = cell.attribute(CELL_ATTR).ok()??;
    let (row, col) = name.split_once('.')?;
    let row = match row.strip_prefix('h') {
        Some(level) => GridRow::Head(level.parse().ok()?),
        None => GridRow::Body(VisibleRow::Data(row.parse().ok()?)),
    };
    Some(FocusTarget {
        row,
        col: Some(col.parse().ok()?),
    })
}

impl GridFocus {
    /// A grid cell's attributes: one tab stop, and focus inside makes it the stop.
    /// `header` for a `th`, which reads as a column or row header.
    pub fn cell(
        &self,
        row: GridRow,
        col: usize,
        is_stop: bool,
        header: bool,
        colindex: Option<usize>,
    ) -> Vec<Attribute> {
        let mut stop = self.stop;
        let target = FocusTarget {
            row,
            col: Some(col),
        };
        let mut attributes = vec![
            attr("tabindex", if is_stop { "0" } else { "-1" }),
            listener("onfocusin", move |_: Event<FocusData>| {
                if *stop.peek() != Some(target) {
                    stop.set(Some(target));
                }
            }),
        ];
        if is_stop {
            attributes.push(attr("data-cell-stop", self.id.peek().clone()));
        }
        if silent_focus().is_some() {
            match row {
                GridRow::Head(level) => attributes.push(attr(CELL_ATTR, format!("h{level}.{col}"))),
                GridRow::Body(VisibleRow::Data(index)) => {
                    attributes.push(attr(CELL_ATTR, format!("{index}.{col}")));
                }
                GridRow::Body(VisibleRow::Group(_)) => {}
            }
        }
        if !header && !infers_grid_cells() {
            attributes.push(attr("role", "gridcell"));
        }
        if let Some(colindex) = colindex {
            attributes.push(attr("aria-colindex", colindex.to_string()));
        }
        attributes
    }

    /// The table's key handler: arrows, Home, End and the page keys move the stop.
    pub fn keydown(self, rows: GridRows) -> Attribute {
        let rows = Rc::new(rows);
        listener("onkeydown", move |event: Event<KeyboardData>| {
            if key_taken(&event)
                || typing_target(&event)
                || arrow_target(&event)
                || event.is_composing()
            {
                return;
            }
            let modifiers = event.modifiers();
            let ctrl = modifiers.ctrl() || modifiers.meta();
            if modifiers.alt() {
                return;
            }
            if self.cell_key(&rows, &event, ctrl, modifiers.shift()) {
                event.prevent_default();
                event.stop_propagation();
                return;
            }
            if modifiers.shift() {
                return;
            }
            let Some(by) = key_move(event.key(), ctrl, rtl_target(&event), 1) else {
                return;
            };
            if matches!(
                self.table.query_selector(GRID_CELL_FOCUS),
                Err(PlatformError::NotFound)
            ) {
                return;
            }
            event.prevent_default();
            // Not the scroll region's: it would scroll as well.
            event.stop_propagation();
            match by {
                Move::PageUp(_) | Move::PageDown(_) => {
                    let (rows, page) = (rows.clone(), self.page(&rows));
                    spawn(async move {
                        let page = page.await;
                        let by = match by {
                            Move::PageUp(_) => Move::PageUp(page),
                            _ => Move::PageDown(page),
                        };
                        self.go(&rows, by);
                    });
                }
                by => self.go(&rows, by),
            }
        })
    }

    /// A press on the stop cell itself, not a widget in it: Enter or Space sorts a
    /// header, Ctrl+Enter opens its menu, Enter or F2 edits an editable body cell,
    /// Shift+Space selects its row. Whether it acted. A WebView cannot tell the
    /// cell from its widget: never there.
    fn cell_key(
        &self,
        rows: &GridRows,
        event: &Event<KeyboardData>,
        ctrl: bool,
        shift: bool,
    ) -> bool {
        let space = matches!(event.key(), Key::Character(ref text) if text == " ");
        let enter = event.key() == Key::Enter;
        let f2 = event.key() == Key::F2;
        if !(space || enter || f2) {
            return false;
        }
        let on_stop = |part: &str| {
            let selector = format!("{part} [data-cell-stop='{}']:focus", self.id.peek());
            self.table.query_selector(&selector).is_ok()
        };
        let stop = rows.at(*self.stop.peek()).map(|at| rows.target(at));
        match (stop, shift, ctrl) {
            (
                Some(FocusTarget {
                    row: GridRow::Head(_),
                    col: Some(col),
                }),
                false,
                menu,
            ) if !f2 && (enter || !menu) && on_stop("thead") => {
                rows.header.call((col, menu));
                true
            }
            (
                Some(
                    target @ FocusTarget {
                        row: GridRow::Body(_),
                        col: Some(col),
                    },
                ),
                false,
                false,
            ) if (enter || f2) && rows.editable.get(col) == Some(&true) && on_stop("tbody") => {
                self.start(target);
                true
            }
            (
                Some(FocusTarget {
                    row: GridRow::Body(VisibleRow::Data(index)),
                    ..
                }),
                true,
                false,
            ) if space && on_stop("tbody") => match rows.select {
                Some(select) => {
                    select.call(index);
                    true
                }
                None => false,
            },
            _ => false,
        }
    }

    /// Opens the editor of the body cell `target`, which becomes the stop.
    pub fn start(mut self, target: FocusTarget) {
        if *self.stop.peek() != Some(target) {
            self.stop.set(Some(target));
        }
        self.editing.set(Some(target));
    }

    /// Closes the editor; `refocus` focuses its cell again, as the field goes with the
    /// focus. Not after a blur, whose focus went elsewhere.
    pub fn finish(mut self, refocus: bool) {
        self.editing.set(None);
        if refocus {
            self.request.set(Some(None));
        }
    }

    /// Moves the stop and asks for focus on it after the render.
    fn go(mut self, rows: &GridRows, by: Move) {
        let Some(mut at) = rows.at(*self.stop.peek()) else {
            return;
        };
        let from = rows.target(at);
        let spans = |row: usize| rows.row_spans(row);
        let mut target = from;
        // A column's header spans the levels under its neighbours' groups: past it.
        for _ in 0..rows.order.len() {
            let to = step(at, by, rows.order.len(), spans);
            let moved = to != at;
            at = to;
            target = rows.target(at);
            let vertical = matches!(
                by,
                Move::Up | Move::Down | Move::PageUp(_) | Move::PageDown(_)
            );
            if target != from || !moved || !vertical {
                break;
            }
        }
        if *self.stop.peek() != Some(target) {
            self.stop.set(Some(target));
        }
        let head = rows.head.len();
        let reveal = match (rows.window.as_ref(), target.row) {
            (Some(window), GridRow::Body(row)) => {
                let mut focused = window.focused;
                // Kept rendered while out of view, so the focus finds it.
                if *focused.peek() != Some(row) {
                    focused.set(Some(row));
                }
                Some(Reveal {
                    slot: at.row - head,
                    pitch: window.pitch,
                    edges: window.edges,
                })
            }
            _ => None,
        };
        self.request.set(Some(reveal));
    }

    /// The rows one view shows: the region's height over a row's. Reads start here:
    /// natively a spawned task cannot read the document.
    fn page(self, rows: &GridRows) -> impl Future<Output = usize> + 'static {
        let view = self.region.dimensions();
        let window = rows
            .window
            .as_ref()
            .map(|window| (window.pitch, window.edges));
        let row = window.is_none().then(|| {
            self.table
                .query_selector("tbody > tr")
                .map(|row| row.dimensions())
        });
        async move {
            let Ok(view) = view.await else {
                return PAGE_ROWS;
            };
            let (pitch, (head, foot)) = match (window, row) {
                (Some(at), _) => at,
                (None, Some(Ok(row))) => match row.await {
                    Ok(row) => (row.height, (0.0, 0.0)),
                    Err(_) => return PAGE_ROWS,
                },
                _ => return PAGE_ROWS,
            };
            if pitch <= 0.0 {
                return PAGE_ROWS;
            }
            (((view.height - head - foot) / pitch).floor() as usize).max(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Columns 0 to 3 shown as 2, 0, 3 after one lead: 1 sits after 0.
    #[test]
    fn a_hidden_column_leaves_a_gap_after_its_source_neighbour() {
        assert_eq!(&*col_indices(&[2, 0, 3], 4, 1), &[3, 0, 2, 5]);
        assert_eq!(&*col_indices(&[1, 2], 3, 0), &[0, 2, 3]);
    }
}
