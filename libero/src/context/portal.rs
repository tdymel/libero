use dioxus::prelude::*;

use crate::platform::backend;

pub(crate) struct PortalEntry {
    pub(crate) id: u64,
    /// Already rendered in the registering component's own scope, not a
    /// closure `PortalOutlet` would call from its unrelated one - that would
    /// read signals from a non-ancestor scope.
    pub(crate) render: Option<Element>,
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

    rsx! {
        div {
            style: backend::PORTAL_ROOT_STYLE,
            for (id , element) in host
                .entries
                .read()
                .iter()
                .filter_map(|entry| Some((entry.id, entry.render.clone()?)))
            {
                Fragment {
                    key: "{id}",
                    {backend::PortalEntry(element)}
                }
            }
        }
    }
}
