use dioxus::prelude::*;

pub(crate) struct PortalEntry {
    pub(crate) id: u64,
    /// The already-rendered content, computed by the registering component
    /// in its own scope (via `use_portal`) - not a closure `PortalOutlet`
    /// would call from its own, unrelated scope. Reading a signal owned by
    /// another, non-ancestor scope at call time is exactly what
    /// `dioxus_signals`' "used in a scope that is not a descendant of the
    /// owning scope" warning flags, so the value has to already be resolved
    /// by the time it lands here.
    pub(crate) render: Option<Element>,
}

pub(crate) type PortalEntries = Signal<Vec<PortalEntry>>;

/// Registry of portaled content, provided by [`crate::LiberoProvider`].
#[derive(Clone, Copy)]
pub struct PortalHost {
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
pub fn PortalOutlet() -> Element {
    let host = use_context::<PortalHost>();

    rsx! {
        div {
            for (id , element) in host
                .entries
                .read()
                .iter()
                .filter_map(|entry| Some((entry.id, entry.render.clone()?)))
            {
                Fragment {
                    key: "{id}",
                    {element}
                }
            }
        }
    }
}
