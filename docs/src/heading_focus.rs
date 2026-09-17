//! Focus on the new page's heading after a navigation. Self-contained: the e2e
//! fixture `docs_shell` includes this file.

use dioxus::prelude::*;
use libero::{hooks::ElementHandle, platform::ElementApi};

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
