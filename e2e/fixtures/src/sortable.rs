//! `Sortable` and `use_sortable`.

use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex, Orientation, Sortable, SortableItem, Text},
    hooks::{SortableMove, SortableOptions, use_sortable, use_sortable_item},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/sortable", || rsx! { SortablePage {} }),
    ("/sortable/horizontal", || rsx! { HorizontalPage {} }),
    ("/sortable/rtl", || rsx! { RtlPage {} }),
    ("/sortable/hook", || rsx! { HookPage {} }),
    ("/sortable/long", || rsx! { LongPage {} }),
];

const NAMES: [&str; 4] = ["Alpha", "Beta", "Gamma", "Delta"];

/// Four items between two buttons; `#order` lists them, `#moves` each move.
#[component]
fn SortablePage() -> Element {
    let mut items = use_signal(|| NAMES.to_vec());
    let mut moves = use_signal(String::new);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            Sortable {
                id: "list",
                onreorder: move |step: SortableMove| {
                    moves.write().push_str(&format!("{}>{} ", step.from, step.to));
                    step.apply(&mut items.write());
                },
                for (index, name) in items().into_iter().enumerate() {
                    SortableItem { key: "{name}", index, id: "{name}", label: name, Text { "{name}" } }
                }
            }
            Text { id: "order", {items().join(" ")} }
            Text { id: "moves", "{moves}" }
            Button { id: "after", "After" }
        }
    }
}

/// 40 unlabelled items in a page taller than the screen: touch scroll and a long drag.
#[component]
fn LongPage() -> Element {
    let mut items = use_signal(|| (1..=40).map(|n| format!("Item{n}")).collect::<Vec<_>>());
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Text { id: "order", {items().join(" ")} }
            Sortable {
                id: "list",
                onreorder: move |step: SortableMove| step.apply(&mut items.write()),
                for (index, name) in items().into_iter().enumerate() {
                    SortableItem { key: "{name}", index, id: "{name}",
                        Text { id: "{name}-text", "{name}" }
                    }
                }
            }
        }
    }
}

/// A row of four.
#[component]
fn HorizontalPage() -> Element {
    let mut items = use_signal(|| NAMES.to_vec());
    rsx! {
        Flex { direction: "column", gap: "md",
            Sortable {
                id: "list",
                orientation: Orientation::Horizontal,
                onreorder: move |step: SortableMove| step.apply(&mut items.write()),
                for (index, name) in items().into_iter().enumerate() {
                    SortableItem { key: "{name}", index, id: "{name}", Text { "{name}" } }
                }
            }
            Text { id: "order", {items().join(" ")} }
        }
    }
}

/// A row under `dir="rtl"`: Alpha sits on the right.
#[component]
fn RtlPage() -> Element {
    let mut items = use_signal(|| NAMES.to_vec());
    rsx! {
        div { dir: "rtl",
            Flex { direction: "column", gap: "md",
                Sortable {
                    id: "list",
                    orientation: Orientation::Horizontal,
                    onreorder: move |step: SortableMove| step.apply(&mut items.write()),
                    for (index, name) in items().into_iter().enumerate() {
                        SortableItem { key: "{name}", index, id: "{name}", label: name, Text { "{name}" } }
                    }
                }
                Text { id: "order", {items().join(" ")} }
            }
        }
    }
}

/// The hook with handles that count their clicks in `#clicks`.
#[component]
fn HookPage() -> Element {
    let mut items = use_signal(|| NAMES.to_vec());
    let clicks = use_signal(|| 0);
    let list = use_sortable(SortableOptions {
        orientation: Orientation::Vertical,
        onreorder: Callback::new(move |step: SortableMove| step.apply(&mut items.write())),
    });
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Box {
                id: "list",
                onmounted: list.element.mount(),
                onpointermove: move |event| list.onpointermove.call(event),
                onpointerup: move |event| list.onpointerup.call(event),
                onpointercancel: move |event| list.onpointercancel.call(event),
                for (index, name) in items().into_iter().enumerate() {
                    HookItem { key: "{name}", index, name, clicks }
                }
            }
            Text { id: "order", {items().join(" ")} }
            Text { id: "clicks", "{clicks}" }
        }
    }
}

#[component]
fn HookItem(index: usize, name: &'static str, clicks: Signal<i32>) -> Element {
    let item = use_sortable_item(index);
    rsx! {
        Box { id: "{name}", onmounted: item.element.mount(), style: item.style(),
            button {
                onmounted: item.handle.mount(),
                onpointerdown: move |event| item.onpointerdown.call(event),
                onkeydown: move |event| item.onkeydown.call(event),
                onblur: move |event| item.onblur.call(event),
                onclick: move |_| clicks += 1,
                class: "handle",
                style: "touch-action: none",
                "Move {name}"
            }
        }
    }
}
