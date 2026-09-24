//! After a navigation: the new page starts at the top, focus on its heading,
//! or both on the section a [`SectionLink`](crate::components::SectionLink) named.
//! The e2e fixture `docs_shell` keeps its own copy (todo 951).

use dioxus::prelude::*;
use libero::{components::ScrollAreaHandle, hooks::ElementHandle, platform::ElementApi};

/// The `DocSection` id the next navigation lands on instead of the page top.
#[derive(Clone, Copy)]
pub struct PendingSection(pub Signal<Option<String>>);

/// Room above a section's heading once it is scrolled to.
const SECTION_MARGIN: f64 = 16.0;

/// Scrolls `area` back to the top whenever `location` changes, or to the pending
/// section. The web's heading focus only scrolls as far as the heading, Blitz's not at all.
/// Before [`use_heading_focus`], which clears the section.
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

/// The section's top below `main`'s is its offset in the area. A WebView has no
/// subtree queries: the page starts at the top there.
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

/// Focuses the `main h1` (needs `tabindex="-1"`) inside `content` whenever `location`
/// changes, or the pending section's heading. Never on the first render: a fresh load
/// keeps the browser's own start.
pub fn use_heading_focus<T: Clone + PartialEq + 'static>(
    location: T,
    content: ElementHandle,
    mut section: Signal<Option<String>>,
) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    use_effect(use_reactive!(|location| {
        if *last.peek() != location {
            last.set(location);
            // A `DocSection`'s title is its first child.
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
