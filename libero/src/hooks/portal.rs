use std::{
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

static NEXT_PORTAL_ID: AtomicU64 = AtomicU64::new(0);

type PortalRender = Rc<dyn Fn() -> Option<Element>>;

pub(crate) struct PortalEntry {
    id: u64,
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

/// Renders everything registered via [`use_portal`] whose `render` currently
/// returns `Some`. [`crate::LiberoProvider`] renders exactly one of these,
/// after its own children.
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

/// Registers `render` to be rendered by the app's [`PortalOutlet`] whenever
/// it returns `Some` - escapes any ancestor's stacking context/clipping, the
/// way `Overlay`/dialog content generally wants to. `render` is called fresh
/// by `PortalOutlet` at its own render time, not snapshotted here, so
/// "should this render" and "what to render" can never disagree. Re-registers
/// on every call so `render` stays current; deregisters automatically on
/// drop.
pub fn use_portal(render: impl Fn() -> Option<Element> + 'static) {
    let mut host = use_context::<PortalHost>();
    let id = use_signal(|| NEXT_PORTAL_ID.fetch_add(1, Ordering::Relaxed));
    let render: PortalRender = Rc::new(render);

    {
        let mut entries = host.entries.write();
        match entries.iter_mut().find(|entry| entry.id == id()) {
            Some(entry) => entry.render = render,
            None => entries.push(PortalEntry { id: id(), render }),
        }
    }

    use_drop(move || {
        host.entries.write().retain(|entry| entry.id != id());
    });
}
