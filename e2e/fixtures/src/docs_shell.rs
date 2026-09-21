//! The docs shell's heading focus and scroll. The hooks are a copy of
//! `docs/src/heading_focus.rs`: e2e never includes docs source (todo 951).

use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Button, Icon, Menu, MenuEntry, MenuItem, ScrollArea, ScrollAreaHandle,
        use_menu, use_scroll_area,
    },
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
    ("/docs-shell/tldr", || rsx! { TldrPage { label: true } }),
    (
        "/docs-shell/tldr-icon",
        || rsx! { TldrPage { label: false } },
    ),
];

const MARK: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Ccircle cx='12' cy='12' r='10'/%3E%3C/svg%3E";

/// The docs' `Tldr` menu (`docs/src/components/tldr.rs`), with links that stay on the fixture.
#[component]
fn TldrPage(label: bool) -> Element {
    let menu = use_menu();
    let providers = ["ChatGPT", "Google AI", "Claude", "Perplexity"]
        .map(|name| {
            MenuItem::new(name)
                .href(format!("https://example.test/{name}?q=x"))
                .leading(
                    rsx! { Icon { src: MARK, variant: "standard", size: "sm", color: "inherit" } },
                )
                .into()
        })
        .to_vec();
    let items = vec![MenuEntry::Group {
        label: "Summarize with".into(),
        items: providers,
    }];
    rsx! {
        h1 { "A page" }
        Menu { state: menu, items,
            if label {
                Button {
                    size: "sm",
                    variant: "outlined",
                    color: "neutral",
                    icon: rsx! { Icon { variant: "standard", size: "sm", color: "inherit", svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "9" } } } },
                    attributes: menu.a11y_attributes(),
                    "TLDR"
                }
            } else {
                ActionIcon {
                    size: "sm",
                    variant: "outlined",
                    color: "neutral",
                    aria_label: "Summarize with AI",
                    attributes: menu.a11y_attributes(),
                    svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "9" } }
                }
            }
        }
    }
}

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
