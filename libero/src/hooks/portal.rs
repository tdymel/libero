use dioxus::prelude::*;

use crate::context::{PortalEntry, PortalHost};
use crate::utils::unique_id;

/// One component's claim on a slot in `PortalOutlet`, written through
/// [`show`](Self::show) after its hooks have run.
#[derive(Clone, Copy)]
pub(crate) struct PortalSlot {
    // Not a `Signal`: read once per entry, a signal made many portals quadratic.
    id: u64,
    host: PortalHost,
}

impl PortalSlot {
    /// Publishes `content`, escaping any ancestor stacking/clipping context.
    /// Rendered, not a closure: `PortalOutlet` would read your signals cross-scope.
    pub(crate) fn show(&self, content: Option<Element>) {
        self.publish(content, false);
    }

    /// [`show`](Self::show) for content that stays mounted while it draws
    /// nothing: `idle` tells the outlet so.
    pub(crate) fn show_idle(&self, content: Element, idle: bool) {
        self.publish(Some(content), idle);
    }

    fn publish(&self, content: Option<Element>, idle: bool) {
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
            Some(entry) => {
                entry.render = content;
                entry.idle = idle;
            }
            None => entries.push(PortalEntry {
                id: self.id,
                render: content,
                idle,
            }),
        }
    }
}

/// Claims a portal slot for this component, and releases it on drop.
pub(crate) fn use_portal_slot() -> PortalSlot {
    let host = use_context::<PortalHost>();
    let id = use_hook(unique_id);

    use_drop(move || {
        let mut host = host;
        host.entries.write().retain(|entry| entry.id != id);
    });

    PortalSlot { id, host }
}

/// Renders `content` via `PortalOutlet`, like [`PortalSlot::show`].
/// Re-registers each call; deregisters on drop.
pub(crate) fn use_portal(content: Option<Element>) {
    use_portal_slot().show(content);
}
