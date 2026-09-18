use dioxus::prelude::*;

use crate::platform;

pub(crate) struct PortalEntry {
    pub(crate) id: u64,
    /// Already rendered in the registering component's own scope, not a
    /// closure `PortalOutlet` would call from its unrelated one - that would
    /// read signals from a non-ancestor scope.
    pub(crate) render: Option<Element>,
    /// Mounted but drawing nothing (a notification stack's empty live
    /// regions): a scroll need not realign the outlet for it.
    pub(crate) idle: bool,
}

pub(crate) type PortalEntries = Signal<Vec<PortalEntry>>;

/// Registry of portaled content, provided by [`crate::LiberoProvider`].
#[derive(Clone, Copy)]
pub(crate) struct PortalHost {
    pub(crate) entries: PortalEntries,
}

impl PortalHost {
    pub(crate) fn new(entries: PortalEntries) -> Self {
        Self { entries }
    }
}

/// Renders everything registered via [`crate::hooks::use_portal`] that
/// currently returns `Some`. Rendered once by [`crate::LiberoProvider`].
#[component]
pub(crate) fn PortalOutlet() -> Element {
    let host = use_context::<PortalHost>();

    platform::PortalRoot(rsx! {
        for (id , element , idle) in host
            .entries
            .read()
            .iter()
            .filter_map(|entry| Some((entry.id, entry.render.clone()?, entry.idle)))
        {
            Fragment {
                key: "{id}",
                {platform::PortalEntry(element, idle)}
            }
        }
    })
}
