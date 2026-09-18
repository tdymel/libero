//! The docs shell's heading focus and scroll, from the docs' own files.

use dioxus::prelude::*;
use libero::{
    components::{Button, ScrollArea, use_scroll_area},
    hooks::use_element,
};

#[path = "../../../docs/src/heading_focus.rs"]
mod heading_focus;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/docs-shell/heading", || rsx! { HeadingPage {} }),
    ("/docs-shell/scroll", || rsx! { ScrollPage {} }),
];

/// Two "pages" behind one signal: the hook only needs a value that changes.
#[component]
fn HeadingPage() -> Element {
    let mut page = use_signal(|| "A");
    let content = use_element();
    heading_focus::use_heading_focus(page(), content);
    rsx! {
        nav { aria_label: "Pages",
            Button { id: "to-a", onclick: move |_| page.set("A"), "Page A" }
            Button { id: "to-b", onclick: move |_| page.set("B"), "Page B" }
        }
        div { onmounted: content.mount(),
            main {
                h1 { tabindex: "-1", "Page {page}" }
            }
        }
    }
}

/// Only the reset: the web's heading focus alone would scroll it back up.
#[component]
fn ScrollPage() -> Element {
    let mut page = use_signal(|| "A");
    let area = use_scroll_area();
    heading_focus::use_scroll_reset(page(), area);
    rsx! {
        Button { id: "to-b", onclick: move |_| page.set("B"), "Page B" }
        div { height: "200px",
            ScrollArea { id: "page-area", handle: area, "aria-label": "Page",
                h1 { "Page {page}" }
                div { height: "2000px" }
            }
        }
    }
}
