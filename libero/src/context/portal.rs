use std::rc::Rc;

use dioxus::prelude::*;

pub(crate) type PortalRender = Rc<dyn Fn() -> Option<Element>>;

pub(crate) struct PortalEntry {
    pub(crate) id: u64,
    pub(crate) render: PortalRender,
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

/// Renders everything registered via [`crate::hooks::use_portal`] whose
/// `render` currently returns `Some`. [`crate::LiberoProvider`] renders
/// exactly one of these, after its own children.
#[component]
pub fn PortalOutlet() -> Element {
    let host = use_context::<PortalHost>();

    rsx! {
        div {
            for (id , element) in host
                .entries
                .read()
                .iter()
                .filter_map(|entry| Some((entry.id, (entry.render)()?)))
            {
                Fragment {
                    key: "{id}",
                    {element}
                }
            }
        }
    }
}
