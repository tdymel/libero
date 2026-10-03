//! `use_swipe` on a pad, and `use_edge_swipe` on a page that scrolls inside a
//! `ScrollArea`, as the docs shell wires it.

use dioxus::prelude::*;
use libero::{
    components::{Box, ScrollArea},
    hooks::{
        EdgeSwipeOptions, SwipeDirection, SwipeEvent, SwipeOptions, edge_swipe_sx, use_direction,
        use_edge_swipe, use_swipe,
    },
    sx::sx,
    theme::Direction,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-swipe/basic", || rsx! { SwipePad {} }),
    ("/use-swipe/edge", || rsx! { EdgePage {} }),
    ("/use-swipe/edge-rtl", || rsx! { EdgeRtlPage {} }),
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

/// The app turned right to left through `use_direction`, put back on leaving.
#[component]
fn EdgeRtlPage() -> Element {
    let direction = use_direction();
    use_hook(|| direction.set(Direction::Rtl));
    use_drop(move || direction.clear());
    rsx! { EdgePage {} }
}
