use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::context::{PortalEntry, PortalHost};

static NEXT_PORTAL_ID: AtomicU64 = AtomicU64::new(0);

/// Renders `content` via [`crate::context::PortalOutlet`], escaping any
/// ancestor stacking/clipping context. Re-registers each call; deregisters on
/// drop.
///
/// Takes an already-rendered `Option<Element>`, not a closure, so signal reads
/// happen in your scope: `PortalOutlet` would invoke a closure from its own
/// scope, which isn't a descendant of the one owning your signals - a
/// cross-scope read, and a real hazard if your scope unmounts first.
pub fn use_portal(content: Option<Element>) {
    let mut host = use_context::<PortalHost>();
    let id = use_signal(|| NEXT_PORTAL_ID.fetch_add(1, Ordering::Relaxed));

    {
        let mut entries = host.entries.write();
        match entries.iter_mut().find(|entry| entry.id == id()) {
            Some(entry) => entry.render = content,
            None => entries.push(PortalEntry {
                id: id(),
                render: content,
            }),
        }
    }

    use_drop(move || {
        host.entries.write().retain(|entry| entry.id != id());
    });
}
