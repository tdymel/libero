//! Pieces more than one fixture module renders. Not a fixture: it has no
//! `ROUTES`.

use dioxus::prelude::*;
use libero::components::{Button, Options};

/// A control on each side of a keyboard group, so "Tab leaves the group" and
/// "Shift+Tab leaves the group" land somewhere real. At the document's edge
/// Chromium parks Shift+Tab on a stop of its own for one press, and the
/// assertion would be reading that instead of the component.
#[component]
pub fn Between(children: Element) -> Element {
    rsx! {
        Button { id: "before", variant: "outlined", "Before" }
        {children}
        Button { id: "after", variant: "outlined", "After" }
    }
}

/// The options of `Select` and `MultiSelect`.
#[derive(Clone, Copy, PartialEq, Options)]
pub enum Fruit {
    Apple,
    Banana,
    Cherry,
    Damson,
    Elderberry,
}
