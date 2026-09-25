use std::{
    collections::HashMap,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::{core::Runtime, prelude::*};

use super::data::{Entry, NotificationId};
use crate::{
    components::common::FOCUSABLE_SELECTOR,
    platform::{self, ElementApi},
};

pub(super) static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// The queue. The app's lives in the root scope, so a notification survives its
/// caller navigating away. A contained host owns one of its own.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct NotificationStore {
    pub(super) entries: Signal<Vec<Entry>>,
    /// Hover or focus on any notification pauses every timer.
    pub(super) hovered: Signal<Option<NotificationId>>,
    pub(super) focused: Signal<Option<NotificationId>>,
    /// Each stack's ids in document order, as the host last drew them.
    pub(super) drawn: CopyValue<Vec<Vec<NotificationId>>>,
    /// Each drawn notification's element, to hand focus on to.
    pub(super) elements: CopyValue<HashMap<NotificationId, Rc<MountedData>>>,
    /// What held focus before it entered the notifications (todo 423).
    pub(super) return_to: CopyValue<Option<Rc<dyn ElementApi>>>,
    /// `return_to` sits inside this contained host, and goes when it does.
    pub(super) return_in_host: CopyValue<bool>,
    /// Set while the store moves focus itself, so that move is no entry.
    pub(super) handing_off: CopyValue<bool>,
    /// `show` may run from a timer callback outside every runtime, and creates
    /// a signal. Weak: the runtime owns the store.
    pub(super) runtime: CopyValue<Weak<Runtime>>,
    /// The scope every signal of the store, and of each entry, belongs to.
    pub(super) owner: ScopeId,
    /// Names a contained host's box in a selector.
    pub(super) id: u64,
}

impl NotificationStore {
    pub(super) fn new(owner: ScopeId) -> Self {
        Self {
            entries: Signal::new_in_scope(Vec::new(), owner),
            hovered: Signal::new_in_scope(None, owner),
            focused: Signal::new_in_scope(None, owner),
            drawn: CopyValue::new_in_scope(Vec::new(), owner),
            elements: CopyValue::new_in_scope(HashMap::new(), owner),
            return_to: CopyValue::new_in_scope(None, owner),
            return_in_host: CopyValue::new_in_scope(false, owner),
            handing_off: CopyValue::new_in_scope(false, owner),
            runtime: CopyValue::new_in_scope(Rc::downgrade(&Runtime::current()), owner),
            owner,
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub(super) fn owned_signal<V: 'static>(&self, value: V) -> Signal<V> {
        let runtime = self
            .runtime
            .peek()
            .upgrade()
            .expect("the notification store outlived its runtime");
        runtime.in_scope(self.owner, || Signal::new_in_scope(value, self.owner))
    }

    /// A handle may outlive a contained host's store.
    pub(super) fn alive(&self) -> bool {
        self.entries.try_peek().is_ok()
    }

    /// A contained host's box, `None` for the app's host.
    pub(super) fn host_selector(&self) -> Option<String> {
        (self.owner != ScopeId::ROOT).then(|| format!("[data-notifications-host=\"{}\"]", self.id))
    }

    pub(super) fn paused(&self) -> bool {
        self.hovered.read().is_some() || self.focused.read().is_some()
    }

    /// Starts the exit of one that is showing, and drops one that is queued.
    pub(super) fn hide(&self, id: NotificationId) {
        if !self.alive() {
            return;
        }
        let mut entries = self.entries;
        let mut entries = entries.write();
        let Some(index) = entries.iter().position(|entry| entry.id == id) else {
            return;
        };
        if entries[index].shown.get() {
            entries[index].leaving = true;
        } else {
            entries.remove(index);
        }
        drop(entries);
        if *self.focused.peek() == Some(id) {
            self.hand_focus_on(id);
        }
    }

    /// Focus leaves a closing notification for the next one in its stack, the
    /// previous one if it was the last, else for where it came from.
    fn hand_focus_on(&self, id: NotificationId) {
        let entries = self.entries.peek();
        let open = |other: &&NotificationId| {
            entries
                .iter()
                .any(|entry| entry.id == **other && !entry.leaving)
        };
        let drawn = self.drawn.peek();
        let elements = self.elements.peek();
        let stack = drawn
            .iter()
            .find(|stack| stack.contains(&id))
            .map(Vec::as_slice)
            .unwrap_or_default();
        let at = stack.iter().position(|other| *other == id).unwrap_or(0);
        let next = stack.get(at + 1..).unwrap_or_default().iter();
        let previous = stack[..at].iter().rev();
        let target = next
            .chain(previous)
            .filter(open)
            .filter_map(|other| elements.get(other))
            .find_map(|item| {
                let item = platform::element(item);
                item.query_selector(r#"[data-slot="close"]"#)
                    .or_else(|_| item.query_selector(FOCUSABLE_SELECTOR))
                    .ok()
            })
            .map(Rc::from)
            .or_else(|| self.return_target());
        self.focus(target);
    }

    /// Where focus came from, if that element is still in the document.
    pub(super) fn return_target(&self) -> Option<Rc<dyn ElementApi>> {
        let return_to = self.return_to.peek().clone();
        return_to.filter(|element| element.is_connected())
    }

    pub(super) fn focus(&self, target: Option<Rc<dyn ElementApi>>) {
        let Some(target) = target else {
            return;
        };
        let Some(runtime) = self.runtime.peek().upgrade() else {
            return;
        };
        // Spawned: this may run inside the close button's click dispatch.
        let mut handing_off = self.handing_off;
        handing_off.set(true);
        runtime.in_scope(self.owner, || {
            spawn(async move {
                let _ = target.focus();
                handing_off.set(false);
            })
        });
    }

    pub(super) fn remove(&self, id: NotificationId) {
        if !self.alive() {
            return;
        }
        let mut entries = self.entries;
        entries.write().retain(|entry| entry.id != id);
    }

    /// The newest notification on screen and not closing. Ids only grow.
    pub(super) fn newest(&self) -> Option<NotificationId> {
        let entries = self.entries.try_peek().ok()?;
        let drawn = self.drawn.peek();
        drawn
            .iter()
            .flatten()
            .filter(|id| {
                entries
                    .iter()
                    .any(|entry| entry.id == **id && !entry.leaving)
            })
            .max_by_key(|id| id.0)
            .copied()
    }

    /// The hotkey's move: into the newest notification's first focusable,
    /// else onto the notification itself.
    pub(super) fn focus_newest(&self) {
        let Some(id) = self.newest() else {
            return;
        };
        let Some(item) = self.elements.peek().get(&id).cloned() else {
            return;
        };
        let item = platform::element(&item);
        let _ = match item.query_selector(FOCUSABLE_SELECTOR) {
            Ok(inner) => inner.focus(),
            Err(_) => item.focus(),
        };
    }
}

/// The nearest contained host's store, else the app's. The app's lives in the
/// root scope, so it outlives the component that first asked.
pub(super) fn use_notification_store() -> NotificationStore {
    use_hook(|| {
        try_consume_context::<NotificationStore>().unwrap_or_else(|| {
            dioxus::core::provide_root_context(NotificationStore::new(ScopeId::ROOT))
        })
    })
}
