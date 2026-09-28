use std::rc::Rc;

use dioxus::prelude::*;

use super::core::{HeaderSpec, RowSpec, body_rows};
use crate::{
    components::{common::attr, layout::Virtualize},
    hooks::listener,
    sx::{Sx, sx},
    theme::CssVar,
};

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
}

/// The rows the DOM leaves out still count: the header rows plus every body row.
pub(super) fn window_attributes(head_rows: usize, rows: usize) -> Vec<Attribute> {
    vec![attr("aria-rowcount", (head_rows + rows).to_string())]
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
    } = window;
    let keep_rendered = focused().and_then(|index| order.iter().position(|&at| at == index));
    let keyed = order.clone();
    rsx! {
        Virtualize {
            count: order.len(),
            item_size: row_height,
            keep_rendered,
            item_key: move |position: usize| key(keyed[position]),
            item: move |position: usize| {
                let index = order[position];
                let mut spec = row(position, index);
                spec.attributes.push(listener("onfocusin", move |_: Event<FocusData>| {
                    focused.set(Some(index));
                }));
                spec.attributes.push(listener("onfocusout", move |_: Event<FocusData>| {
                    if focused.peek().is_some_and(|at| at == index) {
                        focused.set(None);
                    }
                }));
                spec.attributes.push(attr("aria-rowindex", (head_rows + position + 1).to_string()));
                body_rows(spec, &headers, 0, None)
                    .next()
                    .unwrap_or_else(VNode::empty)
            },
        }
    }
}
