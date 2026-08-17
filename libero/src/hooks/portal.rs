use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::context::{PortalEntry, PortalHost};

static NEXT_PORTAL_ID: AtomicU64 = AtomicU64::new(0);

/// Renders `content` via [`crate::context::PortalOutlet`] whenever it's
/// `Some`, escaping any ancestor stacking/clipping context. Re-registers
/// each call so `content` stays current; deregisters on drop.
///
/// Takes an already-rendered `Option<Element>`, not a closure - compute it
/// from your own signals before calling this (e.g.
/// `use_portal(zoomed().then(|| rsx! { ... }))`), so any signal reads happen
/// in your own scope. `PortalOutlet` renders whatever's currently registered
/// from its own scope; a closure invoked there instead would read your
/// signals from a scope that isn't a descendant of the one that owns them -
/// exactly what `dioxus_signals`' cross-scope-read warning flags, and a real
/// hazard if your scope unmounts before the entry is deregistered.
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
