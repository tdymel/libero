use dioxus::prelude::*;
use libero::{
    components::{Anchor, Flex, Title},
    sx::sx,
};

use crate::{Route, heading_focus::PendingSection};

/// One titled section of a [`DocPage`](super::DocPage). With an `id`, a
/// [`SectionLink`] can land on it.
#[component]
pub fn DocSection(
    #[props(default)] id: Option<String>,
    title: String,
    children: Element,
) -> Element {
    let landing = id.is_some();
    rsx! {
        Flex {
            id,
            direction: "column",
            gap: "sm",
            // Focused by the landing (`AppShell`), without a ring, as the page title.
            Title {
                size: "xl",
                tabindex: landing.then_some("-1"),
                sx: sx().selector("&:focus", sx().outline("none").box_shadow("none")),
                "{title}"
            }
            {children}
        }
    }
}

/// A link to `to` that scrolls to its `DocSection` with id `section` and focuses that
/// heading, not the page's top.
#[component]
pub fn SectionLink(to: Route, section: String, children: Element) -> Element {
    let pending = try_use_context::<PendingSection>();
    rsx! {
        // `Anchor` takes no `onclick`; the click bubbles here before the new route renders.
        span {
            display: "contents",
            onclick: move |event: MouseEvent| {
                // A modified click opens a tab: this page stays.
                if let Some(PendingSection(mut pending)) = pending
                    && event.modifiers().is_empty()
                {
                    pending.set(Some(section.clone()));
                }
            },
            Anchor { to, {children} }
        }
    }
}
