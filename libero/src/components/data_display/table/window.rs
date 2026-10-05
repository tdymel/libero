use std::rc::Rc;

use dioxus::prelude::*;

use super::super::sortable::SortableMove;
use super::{
    core::{HeaderSpec, RowSpec, body_rows},
    row_reorder::RowReorder,
};
use crate::{
    components::{
        common::attr,
        layout::{RowsInTable, Virtualize},
    },
    hooks::{ElementHandle, listener, use_silent_focus_within},
    platform::{ElementApi, next_task},
    sx::{Sx, sx},
    theme::CssVar,
};

/// Rows kept beyond each edge while a row holds focus: a held Tab outruns the
/// scroll's re-render by a few rows, and must find the next one there (todo 1420).
const FOCUSED_OVERSCAN: usize = 24;

/// Every row's height in a windowed body.
pub(super) const TABLE_ROW_HEIGHT_VAR: CssVar = CssVar::new("--lsx-table-row-height");

/// The body's rows: all of them, each its own scope, or only the ones a `ScrollArea` shows.
pub(super) enum BodyRows {
    All(Vec<Element>),
    Window(RowWindow),
}

impl BodyRows {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::All(rows) => rows.is_empty(),
            Self::Window(window) => window.order.is_empty(),
        }
    }
}

/// A windowed body: the shown rows' `data` indices, projected as they scroll
/// in. No detail rows: they would break the one row height the window assumes.
pub(super) struct RowWindow {
    pub order: Rc<[usize]>,
    /// The rows on the pages before this one, counted in `aria-rowindex`.
    pub first: usize,
    pub row_height: f64,
    /// A row's spec by its position among the shown rows and its `data` index.
    pub row: Rc<dyn Fn(usize, usize) -> RowSpec>,
    /// A row's key by `data` index, so its node follows it through a sort.
    pub key: Rc<dyn Fn(usize) -> String>,
    /// The `data` index of the row holding focus, kept rendered out of view.
    pub focused: Signal<Option<usize>>,
    pub moves: CopyValue<u64>,
}

/// The row holding focus in a windowed body, by `data` index.
#[derive(Clone, Copy)]
pub(super) struct RowFocus {
    pub focused: Signal<Option<usize>>,
    /// Bumped by each row `focusin`: a `focusout` clears `focused` only if none followed.
    pub moves: CopyValue<u64>,
    /// The header rows and the shown rows' `data` indices, as last rendered.
    shown: CopyValue<(usize, Rc<[usize]>)>,
    /// The table, on Blitz only: its Tab and `focus()` fire no `focusin`.
    table: Option<ElementHandle>,
}

pub(super) fn use_row_focus() -> RowFocus {
    let focused = use_signal(|| None::<usize>);
    let shown = use_hook(|| CopyValue::new((0, Rc::<[usize]>::from([]))));
    let moves = use_hook(|| CopyValue::new(0u64));
    let table = use_silent_focus_within(move |table, inside| {
        let at = inside.then(|| focused_row(&table, &shown.peek())).flatten();
        let mut focused = focused;
        if *focused.peek() != at {
            focused.set(at);
        }
    });
    RowFocus {
        focused,
        moves,
        shown,
        table,
    }
}

impl RowFocus {
    /// Remembers the window's rows, so a silent move maps back to a `data` index.
    pub fn show(mut self, head_rows: usize, order: Rc<[usize]>) {
        self.shown.set((head_rows, order));
    }

    /// Follows the focused row through a move of `data`, before the scroll brings the
    /// window to its slot: kept by its old index, it unmounted and lost the focus (todo 2233).
    pub fn follow(mut self, step: SortableMove) {
        let Some(at) = *self.focused.peek() else {
            return;
        };
        let next = moved_index(at, step);
        if next != at {
            self.focused.set(Some(next));
        }
    }

    pub fn attributes(&self) -> Option<Attribute> {
        self.table.map(|table| listener("onmounted", table.mount()))
    }
}

/// Where index `at` lands once `step` moved one item, shifting the ones between.
fn moved_index(at: usize, SortableMove { from, to }: SortableMove) -> usize {
    match at {
        _ if at == from => to,
        _ if from < at && at <= to => at - 1,
        _ if to <= at && at < from => at + 1,
        _ => at,
    }
}

/// The `data` index of the body row holding focus, by its `aria-rowindex`.
fn focused_row(table: &ElementHandle, (head_rows, order): &(usize, Rc<[usize]>)) -> Option<usize> {
    let row = table
        .query_selector_all("tbody > tr")
        .ok()?
        .into_iter()
        .find(|row| row.is_focused() || row.query_selector(":focus").is_ok())?;
    let index: usize = row.attribute("aria-rowindex").ok()??.parse().ok()?;
    order.get(index.checked_sub(head_rows + 1)?).copied()
}

/// The rows the DOM leaves out still count: the header rows plus every body row,
/// or the one empty row; the hidden skeleton rows count none (todo 1421).
pub(super) fn window_attributes(head_rows: usize, rows: usize, skeleton: bool) -> Vec<Attribute> {
    let body = if skeleton { 0 } else { rows.max(1) };
    vec![attr("aria-rowcount", (head_rows + body).to_string())]
}

/// A windowed table's least width: its sized columns, and a floor for each other one.
pub(super) const TABLE_MIN_WIDTH_VAR: CssVar = CssVar::new("--lsx-table-min-width");

/// The floor of a windowed column with no `width`.
const UNSIZED_FLOOR: &str = "8rem";

/// The least width of a windowed table: fixed layout splits the room among the unsized
/// columns, which would shrink to nothing on a narrow screen rather than scroll (todo 2014).
pub(super) fn windowed_min_width(
    lead: Option<&str>,
    headers: &[HeaderSpec],
    shown: &[usize],
) -> String {
    let parts: Vec<&str> = lead
        .into_iter()
        .chain(
            shown
                .iter()
                .map(|&index| headers[index].width.as_deref().unwrap_or(UNSIZED_FLOOR)),
        )
        .collect();
    format!("max(100%, calc({}))", parts.join(" + "))
}

/// Fixed layout, so a column keeps its width as rows come and go; one line
/// per cell and no block padding, so every row is the pitch the window assumes.
pub(super) fn windowed_sx() -> Sx {
    sx().table_layout("fixed")
        .min_width(TABLE_MIN_WIDTH_VAR.value())
        .selector(
            "& tbody > tr > *",
            sx().box_sizing("border-box")
                .height(TABLE_ROW_HEIGHT_VAR.value())
                .padding_block("0")
                .white_space("nowrap")
                .overflow("hidden")
                .text_overflow("ellipsis"),
        )
}

/// The scroll top that shows slot `to` whole below a `head` px sticky header,
/// in a view `view` px tall scrolled to `top`; `None` when it shows already.
pub(super) fn reveal_slot(to: usize, pitch: f64, head: f64, top: f64, view: f64) -> Option<f64> {
    let (start, end) = (to as f64 * pitch, (to + 1) as f64 * pitch);
    if start < top {
        Some(start)
    } else if head + end > top + view {
        Some(head + end - view)
    } else {
        None
    }
}

/// The rows in view, `aria-rowindex`ed after the `head_rows` header rows.
pub(super) fn render_window(
    window: RowWindow,
    head_rows: usize,
    reorder: Option<RowReorder>,
) -> Element {
    let RowWindow {
        order,
        first,
        row_height,
        row,
        key,
        mut focused,
        mut moves,
    } = window;
    let keep_rendered = focused().and_then(|index| order.iter().position(|&at| at == index));
    let keyed = order.clone();
    rsx! {
        RowsInTable {}
        Virtualize {
            count: order.len(),
            item_size: row_height,
            overscan: keep_rendered.map(|_| FOCUSED_OVERSCAN),
            keep_rendered,
            item_key: move |position: usize| key(keyed[position]),
            item: move |position: usize| {
                let index = order[position];
                let mut spec = row(position, index);
                spec.attributes.push(listener("onfocusin", move |_: Event<FocusData>| {
                    *moves.write() += 1;
                    if *focused.peek() != Some(index) {
                        focused.set(Some(index));
                    }
                }));
                // A task later: Tab to the next row is no moment without a focused row,
                // whose shrunk overscan would drop the rows above it and jump the scroll.
                spec.attributes.push(listener("onfocusout", move |_: Event<FocusData>| {
                    let left = *moves.peek();
                    spawn(async move {
                        next_task().await;
                        if *moves.peek() == left && *focused.peek() == Some(index) {
                            focused.set(None);
                        }
                    });
                }));
                spec.attributes.push(attr("aria-rowindex", (head_rows + first + position + 1).to_string()));
                body_rows(spec, 0, reorder.as_ref())
                    .next()
                    .unwrap_or_else(VNode::empty)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 40px slots under a 42px header in a 200px view.
    #[test]
    fn a_slot_is_revealed_below_the_header_or_left_alone() {
        assert_eq!(reveal_slot(3, 40.0, 42.0, 0.0, 200.0), Some(2.0));
        assert_eq!(reveal_slot(2, 40.0, 42.0, 0.0, 200.0), None);
        assert_eq!(reveal_slot(5, 40.0, 42.0, 400.0, 200.0), Some(200.0));
        assert_eq!(reveal_slot(199, 40.0, 42.0, 0.0, 200.0), Some(7842.0));
    }

    #[test]
    fn an_index_follows_a_move_of_its_list() {
        let down = SortableMove { from: 1, to: 3 };
        let up = SortableMove { from: 3, to: 1 };
        assert_eq!(
            [0, 1, 2, 3, 4].map(|at| moved_index(at, down)),
            [0, 3, 1, 2, 4]
        );
        assert_eq!(
            [0, 1, 2, 3, 4].map(|at| moved_index(at, up)),
            [0, 2, 3, 1, 4]
        );
    }
}
