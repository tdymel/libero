use std::rc::Rc;

use dioxus::prelude::*;

use super::core::{HeaderSpec, RowSpec, body_rows};
use crate::{
    components::{common::attr, layout::Virtualize},
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

/// The body's rows: all of them, or only the ones a `ScrollArea` shows.
pub(super) enum BodyRows {
    All(Vec<RowSpec>),
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

    pub fn attributes(&self) -> Option<Attribute> {
        self.table.map(|table| listener("onmounted", table.mount()))
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
/// or the one empty row.
pub(super) fn window_attributes(head_rows: usize, rows: usize) -> Vec<Attribute> {
    vec![attr("aria-rowcount", (head_rows + rows.max(1)).to_string())]
}

/// Fixed layout, so a column keeps its width as rows come and go; one line
/// per cell and no block padding, so every row is the pitch the window assumes.
pub(super) fn windowed_sx() -> Sx {
    sx().table_layout("fixed").selector(
        "& tbody > tr > *",
        sx().box_sizing("border-box")
            .height(TABLE_ROW_HEIGHT_VAR.value())
            .padding_block("0")
            .white_space("nowrap")
            .overflow("hidden")
            .text_overflow("ellipsis"),
    )
}

/// The rows in view, `aria-rowindex`ed after the `head_rows` header rows.
pub(super) fn render_window(
    window: RowWindow,
    headers: Rc<Vec<HeaderSpec>>,
    head_rows: usize,
) -> Element {
    let RowWindow {
        order,
        row_height,
        row,
        key,
        mut focused,
        mut moves,
    } = window;
    let keep_rendered = focused().and_then(|index| order.iter().position(|&at| at == index));
    let keyed = order.clone();
    rsx! {
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
                spec.attributes.push(attr("aria-rowindex", (head_rows + position + 1).to_string()));
                body_rows(spec, &headers, 0, None)
                    .next()
                    .unwrap_or_else(VNode::empty)
            },
        }
    }
}
