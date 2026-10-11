//! `cell_focus`: one body cell is the tab stop, and the keys of
//! [`cell_focus`](super::cell_focus) move it.

use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    cell_focus::{FocusTarget, Move, Pos, key_move, resolve, step},
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

/// Focus in a body cell or a widget inside one: the header, the footer and the
/// leading select, detail and reorder cells keep their keys.
const BODY_CELL_FOCUS: &str = "tbody td[tabindex]:focus, tbody th[tabindex]:focus, \
                               tbody td[tabindex] :focus, tbody th[tabindex] :focus";

/// A cell's `"{data index}.{header index}"`, on Blitz only: its click, Tab and
/// `focus()` fire no `focusin`, so the silent move finds the cell by it.
const CELL_ATTR: &str = "data-cell";

/// The grid's tab stop and what moves focus to it.
#[derive(Clone, Copy)]
pub(super) struct GridFocus {
    /// Kept while focus is away, so Tab comes back to the same cell.
    pub stop: Signal<Option<FocusTarget>>,
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

/// What a key press reads: the shown rows, the shown headers in display order and
/// each row's cell widths.
pub(super) struct GridRows {
    pub order: Rc<[VisibleRow]>,
    pub layout: Vec<usize>,
    pub spans: Rc<dyn Fn(VisibleRow) -> Vec<usize>>,
    pub window: Option<GridWindow>,
}

impl GridRows {
    /// The stop's cell, by position.
    pub fn at(&self, stop: Option<FocusTarget>) -> Option<Pos> {
        resolve(stop, &self.order, &self.layout, |row| {
            (self.spans)(self.order[row])
        })
    }
}

/// Called every render, `cell_focus` or not, for a stable hook order.
pub(super) fn use_grid_focus(table: ElementHandle, region: ElementHandle) -> GridFocus {
    let stop = use_signal(|| None);
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
        .query_selector_all("tbody > tr")
        .ok()?
        .into_iter()
        .find(|row| holds(row.as_ref()))?;
    let cell = row
        .query_selector_all(&format!("[{CELL_ATTR}]"))
        .ok()?
        .into_iter()
        .find(|cell| holds(cell.as_ref()))?;
    let name = cell.attribute(CELL_ATTR).ok()??;
    let (index, col) = name.split_once('.')?;
    Some(FocusTarget {
        row: VisibleRow::Data(index.parse().ok()?),
        col: Some(col.parse().ok()?),
    })
}

impl GridFocus {
    /// A body cell's attributes: one tab stop, and focus inside makes it the stop.
    pub fn cell(&self, row: VisibleRow, col: usize, is_stop: bool, header: bool) -> Vec<Attribute> {
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
        if let (Some(_), Some(index)) = (silent_focus(), row.data()) {
            attributes.push(attr(CELL_ATTR, format!("{index}.{col}")));
        }
        if !header && !infers_grid_cells() {
            attributes.push(attr("role", "gridcell"));
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
            if modifiers.shift() || modifiers.alt() {
                return;
            }
            let ctrl = modifiers.ctrl() || modifiers.meta();
            let Some(by) = key_move(event.key(), ctrl, rtl_target(&event), 1) else {
                return;
            };
            if matches!(
                self.table.query_selector(BODY_CELL_FOCUS),
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

    /// Moves the stop and asks for focus on it after the render.
    fn go(mut self, rows: &GridRows, by: Move) {
        let Some(at) = rows.at(*self.stop.peek()) else {
            return;
        };
        let to = step(at, by, rows.order.len(), |row| {
            (rows.spans)(rows.order[row])
        });
        let row = rows.order[to.row];
        let target = FocusTarget {
            row,
            col: Some(rows.layout[to.col]),
        };
        if *self.stop.peek() != Some(target) {
            self.stop.set(Some(target));
        }
        let reveal = rows.window.as_ref().map(|window| {
            let mut focused = window.focused;
            // Kept rendered while out of view, so the focus finds it.
            if *focused.peek() != Some(row) {
                focused.set(Some(row));
            }
            Reveal {
                slot: to.row,
                pitch: window.pitch,
                edges: window.edges,
            }
        });
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
