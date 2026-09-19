//! After a navigation: the new page starts at the top, focus on its heading.
//! The e2e fixture `docs_shell` keeps its own copy (todo 951).

use dioxus::prelude::*;
use libero::{components::ScrollAreaHandle, hooks::ElementHandle, platform::ElementApi};

/// Scrolls `area` back to the top whenever `location` changes. The web's
/// heading focus only scrolls as far as the heading, Blitz's not at all.
pub fn use_scroll_reset<T: Clone + PartialEq + 'static>(location: T, area: ScrollAreaHandle) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    use_effect(use_reactive!(|location| {
        if *last.peek() != location {
            last.set(location);
            area.scroll_to(0.0, 0.0);
        }
    }));
}

/// Focuses the `main h1` inside `content` whenever `location` changes, so focus
/// lands on the new page rather than where the link was. Never on the first
/// render: a fresh load keeps the browser's own start. The h1 needs `tabindex="-1"`.
pub fn use_heading_focus<T: Clone + PartialEq + 'static>(location: T, content: ElementHandle) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    use_effect(use_reactive!(|location| {
        if *last.peek() != location {
            last.set(location);
            let _ = content.query_selector("main h1").and_then(|h1| h1.focus());
        }
    }));
}
