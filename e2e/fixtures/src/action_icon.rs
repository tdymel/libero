//! `ActionIcon` in each of its states: plain, toggle, loading, disabled, and
//! the smallest sizes.

use dioxus::prelude::*;
use libero::components::{ActionIcon, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/action-icon", || rsx! { ActionIconPage {} })];

fn glyph() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", fill: "currentColor", "aria-hidden": "true",
            path { d: "M5 5h14v14H5z" }
        }
    }
}

#[component]
fn ActionIconPage() -> Element {
    let mut clicks = use_signal(|| 0);
    let mut pinned = use_signal(|| false);
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            ActionIcon { id: "plain", aria_label: "Copy", onclick: move |_| clicks += 1, {glyph()} }
            ActionIcon {
                id: "toggle",
                aria_label: "Pin",
                variant: "outlined",
                selected: pinned(),
                onclick: move |_| pinned.toggle(),
                {glyph()}
            }
            ActionIcon {
                id: "bare-toggle",
                aria_label: "Bookmark",
                selected: pinned(),
                onclick: move |_| pinned.toggle(),
                {glyph()}
            }
            ActionIcon {
                id: "loading",
                aria_label: "Save",
                variant: "filled",
                loading: true,
                onclick: move |_| clicks += 1,
                {glyph()}
            }
            ActionIcon { id: "disabled", aria_label: "Delete", variant: "filled", disabled: true, {glyph()} }
            ActionIcon { id: "sm", aria_label: "Small", size: "sm", {glyph()} }
            ActionIcon { id: "xs", aria_label: "Extra small", size: "xs", {glyph()} }
            Text { id: "clicks", "{clicks}" }
        }
    }
}
