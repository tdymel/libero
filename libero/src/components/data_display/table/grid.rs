//! `cell_focus`: one body cell is the tab stop, and the keys of
//! [`cell_focus`](super::cell_focus) move it.

use std::rc::Rc;

use dioxus::prelude::*;

use super::{
    cell_focus::{
        FocusTarget, GridRow, HeadCell, Move, Pos, TreeAct, TreeNode, key_move, resolve, step,
        tree_step,
    },
    edit::Leave,
    pipeline::VisibleRow,
    window::reveal_slot,
};
use crate::{
    components::common::attr,
    hooks::{ElementHandle, listener, use_id, use_silent_focus_in},
    platform::{
        ElementApi, PlatformError, arrow_target, focus_selector, focused_tag, infers_grid_cells,
        key_taken, meta_held, rtl_target, silent_focus, typing_target, when_laid_out,
    },
};

/// Rows a PageUp or PageDown moves when the view cannot be measured.
const PAGE_ROWS: usize = 10;

/// Focus in a grid cell or a widget inside one, or on a tree row or its toggle: the
/// filter row, the footer and the leading select, detail and reorder cells keep their keys.
const GRID_CELL_FOCUS: &str = "thead th[tabindex]:focus, thead th[tabindex] :focus, \
                               tbody td[tabindex]:focus, tbody th[tabindex]:focus, \
                               tbody td[tabindex] :focus, tbody th[tabindex] :focus, \
                               tbody > tr[aria-level]:focus, \
                               tbody > tr[aria-level] [data-node-toggle]:focus";

/// A cell's `"{data index}.{header index}"`, a header cell's `"h{level}.{header
/// index}"`, a tree row's `"{data index}"`, on Blitz only: its click, Tab and
/// `focus()` fire no `focusin`, so the silent move finds the cell by it.
const CELL_ATTR: &str = "data-cell";

/// The grid's tab stop and what moves focus to it.
#[derive(Clone, Copy)]
pub(super) struct GridFocus {
    /// Kept while focus is away, so Tab comes back to the same cell.
    pub stop: Signal<Option<FocusTarget>>,
    /// The body cell whose editor is open, with an editable column.
    pub editing: Signal<Option<FocusTarget>>,
    /// The message the open editor's last commit was refused with.
    pub invalid: Signal<Option<String>>,
    /// Moves the stop after the next render, once an editor's commit has landed.
    pending: Signal<Option<Move>>,
    /// The last render's rows, which `pending` moves over.
    rows: CopyValue<Option<Rc<GridRows>>>,
    /// The editable cell a touch began on while it was the stop: its click opens the editor.
    armed: CopyValue<Option<FocusTarget>>,
    /// Focus the stop after the next render, revealing its slot in a window.
    request: Signal<Option<Option<Reveal>>>,
    /// Marks the stop cell as `data-cell-stop`, where a WebView focuses by selector.
    id: Signal<String>,
    /// Focus is in the table, or was on a row a re-render removed: no `focusout` then.
    inside: CopyValue<bool>,
    /// A window's focused row and its slot at the last render.
    held: CopyValue<Option<(VisibleRow, usize)>>,
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
    /// Each grid row's slot in the window, which counts the group rows the arrows pass by.
    pub slots: Rc<[usize]>,
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
    /// With `parent_key`: the rows are a treegrid's.
    pub tree: Option<TreeNav>,
}

/// A treegrid's rows: each one a stop of its own, its cells too with `cells`.
pub(super) struct TreeNav {
    /// The shown body rows, in order.
    pub nodes: Vec<TreeNode>,
    /// Each row's parent by data index, a root its own.
    pub parents: Rc<[usize]>,
    /// Opens (`true`) or closes a row, by data index.
    pub toggle: Callback<(usize, bool)>,
    /// With `onrowclick`: Enter on a row, by data index.
    pub click: Option<Callback<usize>>,
    pub cells: bool,
}

impl GridRows {
    /// The stop as a target: in a tree a row of its own, or the nearest shown
    /// ancestor of a row a collapse hid; else [`Self::at`]'s cell.
    pub fn stop(&self, stop: Option<FocusTarget>) -> Option<FocusTarget> {
        let Some(tree) = &self.tree else {
            return self.at(stop).map(|at| self.target(at));
        };
        let shown = |index: usize| self.order.contains(&GridRow::Body(VisibleRow::Data(index)));
        let row_stop = |index: usize| FocusTarget {
            row: GridRow::Body(VisibleRow::Data(index)),
            col: None,
        };
        let stop = match stop {
            Some(FocusTarget {
                row: GridRow::Body(VisibleRow::Data(from)),
                col,
            }) => {
                let mut index = from;
                while !shown(index) && tree.parents.get(index).is_some_and(|&up| up != index) {
                    index = tree.parents[index];
                }
                match (shown(index), col) {
                    (false, _) => None,
                    (true, Some(_)) if index == from => stop,
                    (true, _) => return Some(row_stop(index)),
                }
            }
            stop => stop,
        };
        match stop {
            // Without cells the first row is the first stop.
            None if !tree.cells => self
                .order
                .get(self.head.len())
                .map(|&row| FocusTarget { row, col: None }),
            stop => self.at(stop).map(|at| self.target(at)),
        }
    }

    /// The tree row stop `target`'s place among the body rows.
    fn tree_row(&self, target: FocusTarget) -> Option<usize> {
        match target {
            FocusTarget {
                row: row @ GridRow::Body(_),
                col: None,
            } => self.order.iter().position(|&at| at == row),
            _ => None,
        }
        .map(|at| at - self.head.len())
    }

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
    let invalid = use_signal(|| None);
    let pending = use_signal(|| None);
    let rows = use_hook(|| CopyValue::new(None));
    let armed = use_hook(|| CopyValue::new(None));
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
    let grid = GridFocus {
        stop,
        editing,
        invalid,
        pending,
        rows,
        armed,
        request,
        id,
        inside: use_hook(|| CopyValue::new(false)),
        held: use_hook(|| CopyValue::new(None)),
        table,
        region,
    };
    use_effect(move || {
        let Some(by) = pending() else {
            return;
        };
        let mut pending = pending;
        pending.set(None);
        match rows.peek().clone() {
            Some(rows) => grid.go(&rows, by),
            None => grid.request_focus(),
        }
    });
    grid
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
        .find(|cell| holds(cell.as_ref()));
    let name = match cell {
        Some(cell) => cell.attribute(CELL_ATTR).ok()??,
        // A tree row, or a widget in it outside a cell.
        None => row.attribute(CELL_ATTR).ok()??,
    };
    let Some((row, col)) = name.split_once('.') else {
        return Some(FocusTarget {
            row: GridRow::Body(VisibleRow::Data(name.parse().ok()?)),
            col: None,
        });
    };
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

    /// The treegrid's own listeners: whether focus is in it.
    pub fn tree_listeners(self) -> [Attribute; 2] {
        let (mut on, mut off) = (self.inside, self.inside);
        [
            listener("onfocusin", move |_: Event<FocusData>| on.set(true)),
            listener("onfocusout", move |_: Event<FocusData>| off.set(false)),
        ]
    }

    /// Once a collapse or filter hid the stop's row, the `shown` stop takes its place,
    /// and the focus too when it was on the hidden row.
    pub fn follow_shown(self, shown: Option<FocusTarget>) {
        let held = *self.stop.peek();
        let hid = held.is_some_and(|at| matches!(at.row, GridRow::Body(_)))
            && shown.is_some()
            && held.map(|at| at.row) != shown.map(|at| at.row);
        if !hid {
            return;
        }
        let (mut stop, mut request, inside) = (self.stop, self.request, self.inside);
        // After the render: no write to a signal it read.
        spawn(async move {
            stop.set(shown);
            if *inside.peek() {
                request.set(Some(None));
            }
        });
    }

    /// Whether focus is in the treegrid, as its [`Self::tree_listeners`] last heard.
    pub fn inside(self) -> bool {
        *self.inside.peek()
    }

    /// In a window, rows opened or closed above the focused row move it by slots:
    /// the view scrolls as far, so the row stays where it was. `focused` is its row and slot.
    pub fn hold_in_view(self, focused: Option<(VisibleRow, usize)>, pitch: f64) {
        let mut held = self.held;
        let last = *held.peek();
        held.set(focused);
        let (Some((row, slot)), Some((was, from))) = (focused, last) else {
            return;
        };
        if row != was || slot == from {
            return;
        }
        let shift = (slot as f64 - from as f64) * pitch;
        let (region, offset) = (self.region, self.region.scroll_offset());
        spawn(async move {
            if let Ok((x, top)) = offset.await {
                let _ = region.scroll_to(x, (top + shift).max(0.0));
            }
        });
    }

    /// A tree row's attributes, a stop of its own as [`Self::cell`]'s are.
    pub fn row(&self, index: usize, is_stop: bool) -> Vec<Attribute> {
        let mut stop = self.stop;
        let row = GridRow::Body(VisibleRow::Data(index));
        let mut attributes = vec![
            attr("tabindex", if is_stop { "0" } else { "-1" }),
            // A focused cell's `focusin` bubbles here after it made itself the stop.
            listener("onfocusin", move |_: Event<FocusData>| {
                let held = *stop.peek();
                if held.is_none_or(|held| held.row != row) {
                    stop.set(Some(FocusTarget { row, col: None }));
                }
            }),
        ];
        if is_stop {
            attributes.push(attr("data-cell-stop", self.id.peek().clone()));
        }
        if silent_focus().is_some() {
            attributes.push(attr(CELL_ATTR, index.to_string()));
        }
        attributes
    }

    /// The table's key handler: arrows, Home, End and the page keys move the stop.
    pub fn keydown(self, rows: GridRows) -> Attribute {
        let rows = Rc::new(rows);
        let mut stored = self.rows;
        stored.set(Some(rows.clone()));
        listener("onkeydown", move |event: Event<KeyboardData>| {
            if key_taken(&event)
                || typing_target(&event)
                || arrow_target(&event)
                || event.is_composing()
            {
                return;
            }
            let modifiers = event.modifiers();
            let ctrl = modifiers.ctrl() || meta_held(modifiers);
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
    /// Shift+Space selects its row. Whether it acted.
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
            match self.table.query_selector(&selector) {
                Ok(_) => true,
                // A WebView mirrors the focus on focusin, before keydown: a `td` or `th` is a cell.
                Err(PlatformError::Unsupported) => {
                    matches!(focused_tag().as_deref(), Some("TD" | "TH" | "TR"))
                }
                Err(_) => false,
            }
        };
        let stop = rows.stop(*self.stop.peek());
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
            // Enter on a tree row: what a click on it does, no more.
            (
                Some(FocusTarget {
                    row: GridRow::Body(VisibleRow::Data(index)),
                    col: None,
                }),
                false,
                false,
            ) if enter && on_stop("tbody") => {
                match rows.tree.as_ref().and_then(|tree| tree.click) {
                    Some(click) => {
                        click.call(index);
                        true
                    }
                    None => false,
                }
            }
            _ => false,
        }
    }

    /// Opens the editor of the body cell `target`, which becomes the stop.
    pub fn start(mut self, target: FocusTarget) {
        if *self.stop.peek() != Some(target) {
            self.stop.set(Some(target));
        }
        self.invalid.set(None);
        self.editing.set(Some(target));
    }

    /// A touch on the editable cell `target` began: remembers it when it was the stop.
    pub fn touched(mut self, target: FocusTarget, touch: bool) {
        let held =
            touch && *self.stop.peek() == Some(target) && *self.editing.peek() != Some(target);
        self.armed.set(held.then_some(target));
    }

    /// The click on the editable cell `target`: a second tap on the stop opens its editor.
    pub fn tapped(mut self, target: FocusTarget) {
        if self.armed.take() == Some(target) {
            self.start(target);
        }
    }

    /// Closes the editor and `leave`s: a refocus of its cell, as the field goes with the
    /// focus, or a move of the stop. Nothing after a blur, whose focus went elsewhere.
    pub fn finish(mut self, leave: Leave) {
        self.editing.set(None);
        self.invalid.set(None);
        match leave {
            Leave::Blur => {}
            Leave::Stay => self.request_focus(),
            Leave::To(by) => self.pending.set(Some(by)),
        }
    }

    /// The id of the open editor's error message.
    pub fn error_id(&self) -> String {
        format!("{}-edit-error", self.id.peek())
    }

    fn request_focus(mut self) {
        self.request.set(Some(None));
    }

    /// Moves the stop and asks for focus on it after the render.
    fn go(mut self, rows: &GridRows, by: Move) {
        let Some((at, target)) = self.tree_go(rows, by).or_else(|| self.cell_go(rows, by)) else {
            return;
        };
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
                let body = at.row - head;
                Some(Reveal {
                    slot: window.slots.get(body).copied().unwrap_or(body),
                    pitch: window.pitch,
                    edges: window.edges,
                })
            }
            _ => None,
        };
        self.request.set(Some(reveal));
    }

    /// A treegrid's move from a row stop, or Left from a row's first cell to the row.
    /// `None` leaves it to the cells; a toggle or no move, to nothing.
    fn tree_go(self, rows: &GridRows, by: Move) -> Option<(Pos, FocusTarget)> {
        let tree = rows.tree.as_ref()?;
        let stop = rows.stop(*self.stop.peek())?;
        let head = rows.head.len();
        let row_at = |body: usize| {
            let row = rows.order[head + body];
            (
                Pos {
                    row: head + body,
                    col: 0,
                },
                FocusTarget { row, col: None },
            )
        };
        let Some(body) = rows.tree_row(stop) else {
            // A body cell in the first column: Left takes the row.
            let at = rows.at(Some(stop))?;
            return match (stop.row, by) {
                (GridRow::Body(_), Move::Left) if at.col == 0 => Some(row_at(at.row - head)),
                _ => None,
            };
        };
        match tree_step(body, by, &tree.nodes, tree.cells) {
            TreeAct::Row(to) => Some(row_at(to)),
            TreeAct::FirstCell => {
                let at = Pos {
                    row: head + body,
                    col: 0,
                };
                Some((at, rows.target(at)))
            }
            TreeAct::Toggle(open) => {
                if let GridRow::Body(VisibleRow::Data(index)) = stop.row {
                    tree.toggle.call((index, open));
                }
                // Focus stays, and the stop with it.
                Some(row_at(body))
            }
            TreeAct::Stay => Some(row_at(body)),
        }
    }

    /// A move between cells.
    fn cell_go(self, rows: &GridRows, by: Move) -> Option<(Pos, FocusTarget)> {
        let mut at = rows.at(rows.stop(*self.stop.peek()))?;
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
        Some((at, target))
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
