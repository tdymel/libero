use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::context::{PortalEntry, PortalHost};

static NEXT_PORTAL_ID: AtomicU64 = AtomicU64::new(0);

/// One component's claim on a slot in [`crate::context::PortalOutlet`].
///
/// Handed out by [`use_portal_slot`], and written through [`show`](Self::show)
/// as often as the owner likes - which is what lets a caller decide what to
/// portal *after* its hooks have run, rather than at the hook call itself.
#[derive(Clone, Copy)]
pub(crate) struct PortalSlot {
    // A plain `u64`, not a `Signal`: it never changes, and the lookup below
    // reads it once per entry - as a signal that was one read per entry per
    // portal per render, which is what made a page with many portals scale
    // quadratically.
    id: u64,
    host: PortalHost,
}

impl PortalSlot {
    /// Publishes `content`, escaping any ancestor stacking/clipping context.
    ///
    /// Takes an already-rendered `Option<Element>`, not a closure, so signal
    /// reads happen in your scope: `PortalOutlet` would invoke a closure from
    /// its own scope, which isn't a descendant of the one owning your signals -
    /// a cross-scope read, and a real hazard if your scope unmounts first.
    pub(crate) fn show(&self, content: Option<Element>) {
        let mut host = self.host;
        // A closed slot re-rendered closed: a write would re-render the
        // outlet for nothing, once per closed `Select` or `Menu` on the page.
        let unchanged = content.is_none()
            && host
                .entries
                .peek()
                .iter()
                .any(|entry| entry.id == self.id && entry.render.is_none());
        if unchanged {
            return;
        }
        let mut entries = host.entries.write();
        match entries.iter_mut().find(|entry| entry.id == self.id) {
            Some(entry) => entry.render = content,
            None => entries.push(PortalEntry {
                id: self.id,
                render: content,
            }),
        }
    }
}

/// Claims a portal slot for this component, and releases it on drop.
pub(crate) fn use_portal_slot() -> PortalSlot {
    let host = use_context::<PortalHost>();
    let id = use_hook(|| NEXT_PORTAL_ID.fetch_add(1, Ordering::Relaxed));

    use_drop(move || {
        let mut host = host;
        host.entries.write().retain(|entry| entry.id != id);
    });

    PortalSlot { id, host }
}

/// Renders `content` via [`crate::context::PortalOutlet`], escaping any
/// ancestor stacking/clipping context. Re-registers each call; deregisters on
/// drop.
///
/// Takes an already-rendered `Option<Element>`, not a closure - see
/// [`PortalSlot::show`] for why.
pub fn use_portal(content: Option<Element>) {
    use_portal_slot().show(content);
}
