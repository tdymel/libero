use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::super::sortable::{
    SORTABLE_HANDLE_SX, SORTABLE_MOVE_SX, SortableMove, SortableOptions,
    use_labelled_sortable_item, use_sortable,
};
use crate::{
    CssLayer,
    components::{
        accessibility::{Announcer, VisuallyHidden},
        common::{Glyph, Orientation},
    },
    context::IconSlot,
    hooks::{current_localization, use_css, use_media_query},
    localization::{TableLabels, fill},
    platform::moves_table_rows,
};

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
            ReorderBody { onreorder: self.onreorder, announcer: self.announcer, {rows} }
        }
    }

    /// The handles' description: hidden, read only through `aria-describedby`.
    pub fn instructions(&self) -> Element {
        rsx! {
            ReorderInstructions { id: self.instructions.clone() }
        }
    }
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
    children: Element,
) -> Element {
    let list = use_sortable(SortableOptions {
        orientation: Orientation::Vertical,
        onreorder,
    });
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
    children: Element,
) -> Element {
    let item = use_labelled_sortable_item(slot, Some(label.clone()));
    let words = current_localization().sortable;
    let named = |template: &str| fill(template, &[("label", &label)]);
    let handle_class = use_css(Some(&SORTABLE_HANDLE_SX), CssLayer::Framework);
    let move_class = use_css(Some(&SORTABLE_MOVE_SX), CssLayer::Framework);
    // Blitz paints no moved `tr`: the keys and buttons still reorder there.
    let drags = moves_table_rows() && !disabled;
    let (onpointerdown, onkeydown, onblur) = (item.onpointerdown, item.onkeydown, item.onblur);
    let (onearlier, onlater) = (item.onearlier, item.onlater);
    // At rest no transform: it would make each row a stacking context under the sticky cells.
    let style = item.style();
    let moving = item.offset() != 0.0 || style.contains("animation");
    rsx! {
        tr {
            onmounted: item.element.mount(),
            style: (moves_table_rows() && moving).then_some(style),
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
                        aria_describedby: instructions,
                        disabled,
                        onmounted: item.handle.mount(),
                        onpointerdown: move |event| {
                            if drags {
                                onpointerdown.call(event);
                            }
                        },
                        onkeydown: move |event| onkeydown.call(event),
                        onblur: move |event| onblur.call(event),
                        Glyph { slot: IconSlot::Grip, icon: lucide::grip_vertical::outlined }
                    }
                    button {
                        class: move_class.clone(),
                        r#type: "button",
                        "data-reorder-move": "up",
                        aria_label: named(words.move_up),
                        disabled: disabled || (item.first)(),
                        onmounted: item.earlier.mount(),
                        onclick: move |event| onearlier.call(event),
                        Glyph { slot: IconSlot::ChevronUp, icon: lucide::chevron_up::outlined }
                    }
                    button {
                        class: move_class,
                        r#type: "button",
                        "data-reorder-move": "down",
                        aria_label: named(words.move_down),
                        disabled: disabled || (item.last)(),
                        onmounted: item.later.mount(),
                        onclick: move |event| onlater.call(event),
                        Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                    }
                }
            }
            {children}
        }
    }
}
