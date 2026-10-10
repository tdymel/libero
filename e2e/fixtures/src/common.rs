//! Pieces more than one fixture module renders. Not a fixture: it has no
//! `ROUTES`.

use dioxus::prelude::*;
use libero::components::{Button, Options};

/// A control on each side of a keyboard group, so Tab and Shift+Tab leave it somewhere real.
/// At the document's edge Chromium parks Shift+Tab on a stop of its own for one press.
#[component]
pub fn Between(children: Element) -> Element {
    rsx! {
        Button { id: "before", variant: "outlined", "Before" }
        {children}
        Button { id: "after", variant: "outlined", "After" }
    }
}

/// A counter the page's event queue bumps after the effects pending before the click ran:
/// `e2e::driver::tick` waits on it where a negative check needs "everything queued has run".
/// A press on it leaves the focus where it was.
#[component]
pub fn Tick() -> Element {
    let mut ticks = use_signal(|| 0u32);
    rsx! {
        span { id: "ticks", "{ticks}" }
        Button {
            id: "tick",
            variant: "outlined",
            onmousedown: move |event: Event<MouseData>| event.prevent_default(),
            onclick: move |_| ticks += 1,
            "Tick"
        }
    }
}

/// The options of `Select` and `MultiSelect`.
#[derive(Clone, Copy, PartialEq, Debug, Options)]
pub enum Fruit {
    Apple,
    Banana,
    Cherry,
    Damson,
    Elderberry,
}
