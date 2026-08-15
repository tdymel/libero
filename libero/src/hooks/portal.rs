use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::context::{PortalEntry, PortalHost, PortalRender};

static NEXT_PORTAL_ID: AtomicU64 = AtomicU64::new(0);

/// Registers `render` to be rendered by the app's [`crate::context::PortalOutlet`]
/// whenever it returns `Some` - escapes any ancestor's stacking
/// context/clipping, the way `Overlay`/dialog content generally wants to.
/// `render` is called fresh by `PortalOutlet` at its own render time, not
/// snapshotted here, so "should this render" and "what to render" can never
/// disagree. Re-registers on every call so `render` stays current;
/// deregisters automatically on drop.
pub fn use_portal(render: impl Fn() -> Option<Element> + 'static) {
    let mut host = use_context::<PortalHost>();
    let id = use_signal(|| NEXT_PORTAL_ID.fetch_add(1, Ordering::Relaxed));
    let render: PortalRender = std::rc::Rc::new(render);

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
