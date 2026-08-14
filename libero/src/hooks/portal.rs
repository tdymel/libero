use std::{
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

static NEXT_PORTAL_ID: AtomicU64 = AtomicU64::new(0);

type PortalRender = Rc<dyn Fn() -> Element>;
pub type PortalEntries = Signal<Vec<(u64, PortalRender)>>;

/// Registry of portaled content, provided by [`crate::LiberoProvider`].
#[derive(Clone, Copy)]
pub struct PortalHost {
    entries: PortalEntries,
}

impl PortalHost {
    pub fn new(entries: PortalEntries) -> Self {
        Self { entries }
    }
}

/// Renders everything registered via [`use_portal`]. [`crate::LiberoProvider`]
/// renders exactly one of these, after its own children.
#[component]
pub fn PortalOutlet() -> Element {
    let host = use_context::<PortalHost>();

    rsx! {
        div {
            for (id , render) in host.entries.read().iter() {
                Fragment {
                    key: "{id}",
                    {render()}
                }
            }
        }
    }
}

/// Registers `render` to be rendered by the app's [`PortalOutlet`] instead of
/// in place - escapes any ancestor's stacking context/clipping, the way
/// `Backdrop`/overlay content generally wants to. Re-registers on every call
/// so the rendered content stays live; deregisters automatically on drop.
pub fn use_portal(render: impl Fn() -> Element + 'static) {
    let mut host = use_context::<PortalHost>();
    let id = use_signal(|| NEXT_PORTAL_ID.fetch_add(1, Ordering::Relaxed));
    let render: Rc<dyn Fn() -> Element> = Rc::new(render);

    {
        let mut entries = host.entries.write();
        match entries.iter_mut().find(|(entry_id, _)| *entry_id == id()) {
            Some(entry) => entry.1 = render,
            None => entries.push((id(), render)),
        }
    }

    use_drop(move || {
        host.entries
            .write()
            .retain(|(entry_id, _)| *entry_id != id());
    });
}
