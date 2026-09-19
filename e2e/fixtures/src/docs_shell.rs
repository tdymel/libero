//! The docs shell's heading focus and scroll. The hooks are a copy of
//! `docs/src/heading_focus.rs`: e2e never includes docs source (todo 951).

use dioxus::prelude::*;
use libero::{
    components::{Button, ScrollArea, ScrollAreaHandle, use_scroll_area},
    hooks::{ElementHandle, use_element},
    platform::ElementApi,
};

use crate::Routes;

/// Scrolls `area` back to the top whenever `location` changes.
pub fn use_scroll_reset<T: Clone + PartialEq + 'static>(location: T, area: ScrollAreaHandle) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    use_effect(use_reactive!(|location| {
        if *last.peek() != location {
            last.set(location);
            area.scroll_to(0.0, 0.0);
        }
    }));
}

/// Focuses the `main h1` inside `content` whenever `location` changes, never on the first render.
pub fn use_heading_focus<T: Clone + PartialEq + 'static>(location: T, content: ElementHandle) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    use_effect(use_reactive!(|location| {
        if *last.peek() != location {
            last.set(location);
            let _ = content.query_selector("main h1").and_then(|h1| h1.focus());
        }
    }));
}

pub const ROUTES: Routes = &[
    ("/docs-shell/heading", || rsx! { HeadingPage {} }),
    ("/docs-shell/scroll", || rsx! { ScrollPage {} }),
];

/// Two "pages" behind one signal: the hook only needs a value that changes.
#[component]
fn HeadingPage() -> Element {
    let mut page = use_signal(|| "A");
    let content = use_element();
    use_heading_focus(page(), content);
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
    use_scroll_reset(page(), area);
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
