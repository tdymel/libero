//! `use_swipe` on a pad, and `use_edge_swipe` on a page that scrolls inside a
//! `ScrollArea`, as the docs shell wires it.

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Box, ScrollArea},
    hooks::{
        EdgeSwipeOptions, SwipeDirection, SwipeEvent, SwipeOptions, edge_swipe_sx, use_edge_swipe,
        use_swipe,
    },
    sx::sx,
    theme::Direction,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-swipe/basic", || rsx! { SwipePad {} }),
    ("/use-swipe/edge", || rsx! { EdgePage {} }),
    ("/use-swipe/edge-rtl", || rsx! { EdgeRtlPage {} }),
    ("/use-swipe/drawer", || rsx! { DrawerPage {} }),
];

#[component]
fn SwipePad() -> Element {
    let mut last = use_signal(|| None::<SwipeDirection>);
    let swipe = use_swipe(
        Callback::new(move |event: SwipeEvent| last.set(Some(event.direction))),
        SwipeOptions::default(),
    );
    let said = last().map_or("none".to_string(), |direction| format!("{direction:?}"));
    rsx! {
        div {
            id: "pad",
            style: "width: 300px; height: 200px; touch-action: none; background: #eee;",
            onpointerdown: move |event| swipe.onpointerdown.call(event),
            onpointermove: move |event| swipe.onpointermove.call(event),
            onpointerup: move |event| swipe.onpointerup.call(event),
            onpointercancel: move |event| swipe.onpointercancel.call(event),
        }
        p { id: "last", "{said}" }
    }
}

#[component]
fn EdgePage() -> Element {
    let mut opens = use_signal(|| 0);
    let swipe = use_edge_swipe(
        Callback::new(move |()| opens += 1),
        EdgeSwipeOptions::default(),
    );
    rsx! {
        ScrollArea { aria_label: "Page", sx: sx().position("fixed").inset("0"),
            Box {
                id: "page",
                onpointerdown: move |event| swipe.onpointerdown.call(event),
                onpointermove: move |event| swipe.onpointermove.call(event),
                onpointerup: move |event| swipe.onpointerup.call(event),
                onpointercancel: move |event| swipe.onpointercancel.call(event),
                sx: edge_swipe_sx().height("3000px"),
                p { id: "opens", "{opens}" }
            }
        }
    }
}

/// The docs shell's drawer: the swipe opens it mid-press and the page turns inert,
/// so the press never ends on the page (todo 2170).
#[component]
fn DrawerPage() -> Element {
    let mut open = use_signal(|| false);
    let mut opens = use_signal(|| 0);
    let swipe = use_edge_swipe(
        Callback::new(move |()| {
            if !open() {
                open.set(true);
                opens += 1;
            }
        }),
        EdgeSwipeOptions::default(),
    );
    rsx! {
        ScrollArea { aria_label: "Page", sx: sx().position("fixed").inset("0"),
            Box {
                id: "page",
                inert: open().then_some(true),
                onpointerdown: move |event| swipe.onpointerdown.call(event),
                onpointermove: move |event| swipe.onpointermove.call(event),
                onpointerup: move |event| swipe.onpointerup.call(event),
                onpointercancel: move |event| swipe.onpointercancel.call(event),
                sx: edge_swipe_sx().height("3000px"),
                p { id: "opens", "{opens}" }
                // A code block's box: its own scroller, so the page's `touch-action` stops there.
                div {
                    id: "scroller",
                    style: "overflow-x: auto; height: 200px; background: #eef;",
                    "fn main() {{}}"
                }
            }
        }
        if open() {
            Box {
                id: "drawer",
                // Centred: API 36 draws edge to edge, the status bar over the top.
                sx: sx()
                    .position("fixed")
                    .inset("0")
                    .display("flex")
                    .align_items("center")
                    .justify_content("center")
                    .background("surface"),
                button { id: "close", onclick: move |_| open.set(false), "Close" }
            }
        }
    }
}

/// Right to left by a provider's start direction: `use_direction().set` kept it in
/// `localStorage`, turning every later page on the origin (todo 2164).
#[component]
fn EdgeRtlPage() -> Element {
    rsx! {
        LiberoProvider { direction: Direction::Rtl, EdgePage {} }
    }
}
