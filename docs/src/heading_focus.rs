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

/// A fresh load's `#id` fragment (todo 1175).
#[derive(Clone)]
struct LoadFragment(Option<String>);

/// Above the `Router`: it replaces the URL without the fragment as it starts.
pub fn use_load_fragment() {
    use_context_provider(|| LoadFragment(fragment(&history().current_route())));
}

fn fragment(route: &str) -> Option<String> {
    route
        .split_once('#')
        .map(|(_, id)| id.to_string())
        .filter(|id| !id.is_empty())
}

/// Lands the [`use_load_fragment`] fragment on its `DocSection`, scroll and heading
/// focus, as a [`SectionLink`](crate::components::SectionLink) does.
pub fn use_fragment_landing(area: ScrollAreaHandle, content: ElementHandle) {
    let id = try_use_context::<LoadFragment>().and_then(|fragment| fragment.0);
    let mut done = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        if *done.peek() || !content.is_mounted() {
            return;
        }
        done.set(true);
        let Some(id) = id.as_deref() else {
            return;
        };
        scroll_to_section(area, content, id);
        let _ = content
            .query_selector(&format!("#{id} > :first-child"))
            .and_then(|heading| heading.focus());
        #[cfg(target_arch = "wasm32")]
        restore_fragment(id);
    });
}

/// Puts the fragment the router dropped back into the URL, so a reload lands here too (todo 1184).
/// Sent, not formatted in: the id comes from the URL.
#[cfg(target_arch = "wasm32")]
fn restore_fragment(id: &str) {
    let eval = document::eval(
        "const id = await dioxus.recv();
        history.replaceState(history.state, '', location.pathname + location.search + '#' + id);",
    );
    let _ = eval.send(id);
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

#[cfg(test)]
mod tests {
    use super::fragment;

    #[test]
    fn fragment_is_the_part_after_the_hash() {
        assert_eq!(
            fragment("/about/styling#style-api").as_deref(),
            Some("style-api")
        );
        assert_eq!(fragment("/about/styling?x=1#a").as_deref(), Some("a"));
        assert_eq!(fragment("/about/styling#"), None);
        assert_eq!(fragment("/about/styling"), None);
    }
}
