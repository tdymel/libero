//! The sequential focus starting point: prose between buttons to click on, and
//! a group removed with the focus in it.

use dioxus::prelude::*;
use libero::components::Button;

use crate::Routes;

pub const ROUTES: Routes = &[("/focus-start", || rsx! { FocusStartPage {} })];

#[component]
fn FocusStartPage() -> Element {
    rsx! {
        Button { id: "first", "First" }
        p { id: "early", "Before the middle" }
        Button { id: "middle", "Middle" }
        p { id: "late", "After the middle" }
        Button { id: "last", "Last" }
        Removable {}
        Button { id: "after", "After" }
    }
}

/// A group that removes itself, focus and all, from its own button.
#[component]
fn Removable() -> Element {
    let mut shown = use_signal(|| true);
    rsx! {
        if shown() {
            div {
                Button { id: "inside", "Inside" }
                Button { id: "remove", onclick: move |_| shown.set(false), "Remove" }
            }
        }
    }
}
