//! The docs shell's heading focus and scroll. The hooks are a copy of
//! `docs/src/heading_focus.rs`: e2e never includes docs source (todo 951).

use dioxus::prelude::*;
use libero::{
    components::{Button, ScrollArea, ScrollAreaHandle, use_scroll_area},
    hooks::{ElementHandle, use_element},
    platform::ElementApi,
};

use crate::Routes;

/// Room above a section's heading once it is scrolled to.
const SECTION_MARGIN: f64 = 16.0;

/// Scrolls `area` back to the top whenever `location` changes, or to the pending section.
pub fn use_scroll_reset<T: Clone + PartialEq + 'static>(
    location: T,
    area: ScrollAreaHandle,
    content: ElementHandle,
    section: Signal<Option<String>>,
) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    use_effect(use_reactive!(|location| {
        if *last.peek() != location {
            last.set(location);
            match section.peek().as_deref() {
                Some(id) => scroll_to_section(area, content, id),
                None => area.scroll_to(0.0, 0.0),
            }
        }
    }));
}

fn scroll_to_section(area: ScrollAreaHandle, content: ElementHandle, id: &str) {
    let (Ok(target), Ok(main)) = (
        content.query_selector(&format!("#{id}")),
        content.query_selector("main"),
    ) else {
        return area.scroll_to(0.0, 0.0);
    };
    let (target, main) = (target.client_offset(), main.client_offset());
    spawn(async move {
        match (target.await, main.await) {
            (Ok((_, y)), Ok((_, top))) => area.scroll_to(0.0, y - top - SECTION_MARGIN),
            _ => area.scroll_to(0.0, 0.0),
        }
    });
}

/// Focuses the `main h1` inside `content` whenever `location` changes, or the pending
/// section's heading, never on the first render.
pub fn use_heading_focus<T: Clone + PartialEq + 'static>(
    location: T,
    content: ElementHandle,
    mut section: Signal<Option<String>>,
) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    use_effect(use_reactive!(|location| {
        if *last.peek() != location {
            last.set(location);
            let heading = section.write().take().and_then(|id| {
                content
                    .query_selector(&format!("#{id} > :first-child"))
                    .ok()
            });
            let _ = heading
                .map_or_else(|| content.query_selector("main h1"), Ok)
                .and_then(|h| h.focus());
        }
    }));
}

/// Lands a fresh load's fragment `id` on that section, scroll and heading focus. The
/// docs read `id` above their `Router`, which drops it; here the page passes it.
pub fn use_fragment_landing(
    area: ScrollAreaHandle,
    content: ElementHandle,
    id: Option<&'static str>,
) {
    let mut done = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        if *done.peek() || !content.is_mounted() {
            return;
        }
        done.set(true);
        let Some(id) = id else {
            return;
        };
        scroll_to_section(area, content, id);
        let _ = content
            .query_selector(&format!("#{id} > :first-child"))
            .and_then(|heading| heading.focus());
    });
}

pub const ROUTES: Routes = &[
    ("/docs-shell/heading", || rsx! { HeadingPage {} }),
    ("/docs-shell/scroll", || rsx! { ScrollPage {} }),
    ("/docs-shell/section", || rsx! { SectionPage {} }),
    ("/docs-shell/fragment", || rsx! { FragmentPage {} }),
];

/// Two "pages" behind one signal: the hook only needs a value that changes.
#[component]
fn HeadingPage() -> Element {
    let mut page = use_signal(|| "A");
    let content = use_element();
    let section = use_signal(|| None);
    use_heading_focus(page(), content, section);
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
    let content = use_element();
    use_scroll_reset(page(), area, content, use_signal(|| None));
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

/// Page B's `#far` section below the fold, landed on as the docs' `SectionLink` does.
#[component]
fn SectionPage() -> Element {
    let mut page = use_signal(|| "A");
    let mut section = use_signal(|| None);
    let area = use_scroll_area();
    let content = use_element();
    use_scroll_reset(page(), area, content, section);
    use_heading_focus(page(), content, section);
    rsx! {
        Button {
            id: "to-far",
            onclick: move |_| {
                section.set(Some("far".to_string()));
                page.set("B");
            },
            "Page B, far section"
        }
        div { height: "300px",
            ScrollArea { id: "page-area", handle: area, "aria-label": "Page",
                div { display: "contents", onmounted: content.mount(),
                    main {
                        h1 { tabindex: "-1", "Page {page}" }
                        div { height: "1500px" }
                        if page() == "B" {
                            section { id: "far",
                                h2 { tabindex: "-1", "Far" }
                                div { height: "1500px" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A `#far` section below the fold, landed on at the first render (todo 1175).
#[component]
fn FragmentPage() -> Element {
    let area = use_scroll_area();
    let content = use_element();
    use_fragment_landing(area, content, Some("far"));
    rsx! {
        div { height: "300px",
            ScrollArea { id: "page-area", handle: area, "aria-label": "Page",
                div { display: "contents", onmounted: content.mount(),
                    main {
                        h1 { tabindex: "-1", "Fragment" }
                        div { height: "1500px" }
                        section { id: "far",
                            h2 { tabindex: "-1", "Far" }
                            div { height: "1500px" }
                        }
                    }
                }
            }
        }
    }
}
