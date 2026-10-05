use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::super::sortable::{
    FixedSlots, SORTABLE_HANDLE_SX, SORTABLE_MOVE_SX, SortableMove, SortableOptions,
    use_fixed_sortable, use_spanning_sortable_item,
};
use crate::{
    CssLayer,
    components::{
        accessibility::{Announcer, VisuallyHidden},
        common::{Glyph, Orientation},
        layout::use_kept_slot,
    },
    context::IconSlot,
    hooks::{current_localization, use_css, use_element, use_media_query},
    localization::{TableLabels, fill},
    platform::moves_table_rows,
};

/// A natively moved row's `transform` and settle `animation`, for its cells.
pub(super) const ROW_TRANSFORM: &str = "--lsx-row-transform";
pub(super) const ROW_ANIMATION: &str = "--lsx-row-animation";

/// The column of row reorder handles, with `onrowreorder`.
#[derive(Clone)]
pub(super) struct RowReorder {
    /// Called with a move between display slots.
    pub onreorder: Callback<SortableMove>,
    /// Sorted or filtered: the shown order is not `data`'s, so nothing moves.
    pub disabled: bool,
    pub announcer: Announcer,
    pub labels: TableLabels,
    /// The handles' description, a hidden node beside the table.
    pub instructions: String,
    /// Windowed: every shown row's slot, at the row height.
    pub fixed: Option<FixedSlots>,
}

/// One row's place among the shown rows, and its name in the controls.
pub(super) struct ReorderSlot {
    pub slot: usize,
    pub label: String,
}

impl RowReorder {
    /// Named for a screen reader only, as the detail toggles' header.
    pub fn header_cell(&self, rowspan: usize) -> Element {
        rsx! {
            th {
                scope: "col",
                rowspan: (rowspan > 1).then(|| rowspan.to_string()),
                "data-reorder": true,
                VisuallyHidden { "{self.labels.reorder}" }
            }
        }
    }

    /// The `tbody`, the list the rows drag in.
    pub fn body(&self, rows: Element) -> Element {
        rsx! {
            ReorderBody {
                onreorder: self.onreorder,
                announcer: self.announcer,
                fixed: self.fixed,
                {rows}
            }
        }
    }

    /// The handles' description, and while off why: hidden, read only through `aria-describedby`.
    pub fn instructions(&self) -> Element {
        rsx! {
            ReorderInstructions { id: self.instructions.clone() }
            if self.disabled {
                div { id: reason_id(&self.instructions), hidden: true, {self.labels.reorder_unavailable} }
            }
        }
    }
}

/// The id of the hidden reason the controls are off, beside the instructions.
fn reason_id(instructions: &str) -> String {
    format!("{instructions}-off")
}

#[component]
fn ReorderInstructions(id: String) -> Element {
    let words = current_localization().sortable;
    // A touch screen reader's tap on the handle lifts nothing: point it to the buttons.
    let touch = use_media_query("(pointer: coarse)");
    let described = match touch() {
        true => words.touch_instructions,
        false => words.instructions,
    };
    rsx! {
        div { id, hidden: true, {described} }
    }
}

#[component]
fn ReorderBody(
    onreorder: Callback<SortableMove>,
    announcer: Announcer,
    fixed: Option<FixedSlots>,
    children: Element,
) -> Element {
    let list = use_fixed_sortable(
        SortableOptions {
            orientation: Orientation::Vertical,
            onreorder,
        },
        fixed,
    );
    // Said by the table's live region: none is valid inside a table.
    use_effect(move || {
        let said = list.announcement.read().clone();
        if !said.is_empty() {
            announcer.say(said);
        }
    });
    rsx! {
        tbody {
            "data-sorting": (list.sorting)().then_some(true),
            onmounted: list.element.mount(),
            onpointermove: move |event| list.onpointermove.call(event),
            onpointerup: move |event| list.onpointerup.call(event),
            onpointercancel: move |event| list.onpointercancel.call(event),
            {children}
        }
    }
}

/// A body row that drags by its handle, or steps by its move buttons.
#[component]
pub(super) fn ReorderRow(
    slot: usize,
    label: String,
    disabled: bool,
    instructions: String,
    states: Option<String>,
    stripe: bool,
    selected: Option<bool>,
    attributes: Vec<Attribute>,
    /// While open: the detail row's id and body, and the table's column count.
    detail: Option<(String, Element)>,
    columns: usize,
    /// Windowed: the row height, every slot's.
    pitch: Option<f64>,
    children: Element,
) -> Element {
    let kept = use_kept_slot().filter(|_| pitch.is_some());
    // The open detail row moves with its row, one span with it.
    let extent = use_element();
    let item = use_spanning_sortable_item(
        slot,
        Some(label.clone()),
        detail.is_some().then_some(extent),
    );
    let words = current_localization().sortable;
    let named = |template: &str| fill(template, &[("label", &label)]);
    let handle_class = use_css(Some(&SORTABLE_HANDLE_SX), CssLayer::Framework);
    let move_class = use_css(Some(&SORTABLE_MOVE_SX), CssLayer::Framework);
    let drags = !disabled;
    let (onpointerdown, onkeydown, onblur) = (item.onpointerdown, item.onkeydown, item.onblur);
    let (onearlier, onlater) = (item.onearlier, item.onlater);
    // At rest no transform: it would make each row a stacking context under the sticky cells.
    // A lifted row the window keeps beside it sits off its slot: measured from there (todo 1408).
    let laid_off = match (kept.and_then(|kept| kept()), pitch, (item.dragging)()) {
        (Some((at, laid)), Some(pitch), true) if at == slot => (slot as f64 - laid as f64) * pitch,
        _ => 0.0,
    };
    let style = item.style_by(laid_off);
    let moving = item.offset() + laid_off != 0.0 || style.contains("animation");
    // Blitz paints no moved `tr`: its cells take the row's declarations (todo 1520).
    let cells = moving && !moves_table_rows();
    let style = moving.then(|| match cells {
        true => style
            .replace("transform:", &format!("{ROW_TRANSFORM}:"))
            .replace("animation:", &format!("{ROW_ANIMATION}:")),
        false => style,
    });
    let cells = cells.then_some(true);
    // Off while sorted or filtered: still Tab stops, so the reason is heard (todo 1430).
    let off = disabled.then_some("true");
    let described = match disabled {
        true => reason_id(&instructions),
        false => instructions,
    };
    let reason = disabled.then(|| described.clone());
    let detail = detail.map(|(id, body)| {
        rsx! {
            tr {
                id,
                onmounted: extent.mount(),
                style: style.clone(),
                "data-cell-moved": cells,
                "data-detail": true,
                "data-dragging": (item.dragging)().then_some(true),
                td { colspan: "{columns}", {body} }
            }
        }
    });
    rsx! {
        tr {
            onmounted: item.element.mount(),
            style,
            "data-cell-moved": cells,
            "data-state": states,
            "data-stripe": stripe.then_some(true),
            "data-dragging": (item.dragging)().then_some(true),
            aria_selected: selected.map(|selected| selected.to_string()),
            ..attributes,
            td {
                "data-reorder": true,
                // The controls are not a row click.
                onclick: |event| event.stop_propagation(),
                div { "data-reorder-controls": true,
                    button {
                        class: handle_class,
                        r#type: "button",
                        "data-reorder-handle": true,
                        aria_label: named(words.handle),
                        aria_describedby: described,
                        "aria-disabled": off,
                        onmounted: item.handle.mount(),
                        onpointerdown: move |event| {
                            if drags {
                                onpointerdown.call(event);
                            }
                        },
                        onkeydown: move |event| {
                            if !disabled {
                                onkeydown.call(event);
                            }
                        },
                        onblur: move |event| onblur.call(event),
                        Glyph { slot: IconSlot::Grip, icon: lucide::grip_vertical::outlined }
                    }
                    button {
                        class: move_class.clone(),
                        r#type: "button",
                        "data-reorder-move": "up",
                        aria_label: named(words.move_up),
                        aria_describedby: reason.clone(),
                        "aria-disabled": off,
                        disabled: !disabled && (item.first)(),
                        onmounted: item.earlier.mount(),
                        onclick: move |event| {
                            if !disabled {
                                onearlier.call(event);
                            }
                        },
                        Glyph { slot: IconSlot::ChevronUp, icon: lucide::chevron_up::outlined }
                    }
                    button {
                        class: move_class,
                        r#type: "button",
                        "data-reorder-move": "down",
                        aria_label: named(words.move_down),
                        aria_describedby: reason,
                        "aria-disabled": off,
                        disabled: !disabled && (item.last)(),
                        onmounted: item.later.mount(),
                        onclick: move |event| {
                            if !disabled {
                                onlater.call(event);
                            }
                        },
                        Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                    }
                }
            }
            {children}
        }
        {detail}
    }
}
