use std::{
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

static NEXT_PORTAL_ID: AtomicU64 = AtomicU64::new(0);

type PortalRender = Rc<dyn Fn() -> Element>;

pub(crate) struct PortalEntry {
    id: u64,
    active: bool,
    render: PortalRender,
}

pub type PortalEntries = Signal<Vec<PortalEntry>>;

/// Registry of portaled content, provided by [`crate::LiberoProvider`].
#[derive(Clone, Copy)]
pub struct PortalHost {
    entries: PortalEntries,
}

impl PortalHost {
    pub(crate) fn new(entries: PortalEntries) -> Self {
        Self { entries }
    }
}

/// Renders everything registered via [`use_portal`] that's currently active.
/// [`crate::LiberoProvider`] renders exactly one of these, after its own
/// children.
#[component]
pub fn PortalOutlet() -> Element {
    let host = use_context::<PortalHost>();

    rsx! {
        div {
            for entry in host.entries.read().iter().filter(|entry| entry.active) {
                Fragment {
                    key: "{entry.id}",
                    {(entry.render)()}
                }
            }
        }
    }
}

/// Registers `render` to be rendered by the app's [`PortalOutlet`] while
/// `active` is `true` - escapes any ancestor's stacking context/clipping,
/// the way `Overlay`/dialog content generally wants to. Re-registers on
/// every call so `active`/`render` stay current; deregisters automatically
/// on drop.
pub fn use_portal(active: bool, render: impl Fn() -> Element + 'static) {
    let mut host = use_context::<PortalHost>();
    let id = use_signal(|| NEXT_PORTAL_ID.fetch_add(1, Ordering::Relaxed));
    let render: PortalRender = Rc::new(render);

    {
        let mut entries = host.entries.write();
        match entries.iter_mut().find(|entry| entry.id == id()) {
            Some(entry) => {
                entry.active = active;
                entry.render = render;
            }
            None => entries.push(PortalEntry {
                id: id(),
                active,
                render,
            }),
        }
    }

    use_drop(move || {
        host.entries.write().retain(|entry| entry.id != id());
    });
}
