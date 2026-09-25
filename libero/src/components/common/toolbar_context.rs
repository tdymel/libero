use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::components::common::{Orientation, attr};

static NEXT_ITEM: AtomicU64 = AtomicU64::new(0);

/// Marks a `Toolbar`'s items, valued with the item's id: what its arrow keys walk.
pub(crate) const TOOLBAR_ITEM: &str = "data-toolbar-item";

/// One `Toolbar`'s roving tab stop, shared with the controls below it.
#[derive(Clone, Copy)]
pub(crate) struct ToolbarScope {
    /// Items in mount order: DOM order for children rendered once.
    order: CopyValue<Vec<u64>>,
    /// The item focused last; `None` makes the first one the tab stop.
    current: Signal<Option<u64>>,
    orientation: Signal<Orientation>,
}

impl ToolbarScope {
    pub(crate) fn orientation(&self) -> Orientation {
        *self.orientation.read()
    }

    /// The registered items, in mount order.
    pub(crate) fn items(&self) -> Vec<u64> {
        self.order.peek().clone()
    }

    /// `id` took focus, so it becomes the tab stop.
    pub(crate) fn focused(&self, id: u64) {
        let mut current = self.current;
        if *current.peek() != Some(id) {
            current.set(Some(id));
        }
    }

    /// An unmounted tab stop hands the stop to the first item left.
    fn leave(&self, id: u64) {
        let mut order = self.order;
        let Ok(mut items) = order.try_write() else {
            return;
        };
        let was_first = items.first() == Some(&id);
        items.retain(|&item| item != id);
        let first = items.first().copied();
        drop(items);
        let mut current = self.current;
        let Ok(stop) = current.try_peek().map(|stop| *stop) else {
            return;
        };
        if stop == Some(id) || (stop.is_none() && was_first) {
            current.set(first);
        }
    }
}

/// Makes the controls below one `Toolbar`'s items.
pub(crate) fn use_provide_toolbar(orientation: Orientation) -> ToolbarScope {
    let scope = use_context_provider(|| ToolbarScope {
        order: CopyValue::new(Vec::new()),
        current: Signal::new(None),
        orientation: Signal::new(orientation),
    });
    let mut current = scope.orientation;
    if *current.peek() != orientation {
        current.set(orientation);
    }
    scope
}

/// The enclosing `Toolbar`, `None` outside one.
pub(crate) fn use_toolbar() -> Option<ToolbarScope> {
    use_hook(try_consume_context::<ToolbarScope>)
}

/// A control inside a `Toolbar`: one of its roving tab stops.
#[derive(Clone, Copy)]
pub(crate) struct ToolbarItem {
    id: u64,
    stop: bool,
    scope: ToolbarScope,
}

impl ToolbarItem {
    /// Makes this item the tab stop, from its own click: Blitz's `focusin` comes too early.
    pub(crate) fn take_stop(self) {
        self.scope.focused(self.id);
    }

    /// `0` on the toolbar's one tab stop, `-1` on the rest.
    pub(crate) fn tabindex(self) -> &'static str {
        if self.stop { "0" } else { "-1" }
    }

    /// The [`TOOLBAR_ITEM`] value the toolbar's keys find this item by.
    pub(crate) fn key(self) -> String {
        self.id.to_string()
    }

    /// Marker and `tabindex`, for a path that renders the caller's attributes only.
    pub(crate) fn attributes(self) -> [Attribute; 2] {
        [
            attr(TOOLBAR_ITEM, self.key()),
            attr("tabindex", self.tabindex()),
        ]
    }
}

/// This control's place in the enclosing `Toolbar`, `None` outside one.
pub(crate) fn use_toolbar_item() -> Option<ToolbarItem> {
    let scope = use_toolbar();
    let id = use_hook(|| {
        let id = NEXT_ITEM.fetch_add(1, Ordering::Relaxed);
        if let Some(scope) = scope {
            let mut order = scope.order;
            order.write().push(id);
        }
        id
    });
    use_drop(move || {
        if let Some(scope) = scope {
            scope.leave(id);
        }
    });

    let scope = scope?;
    let stop = match *scope.current.read() {
        Some(current) => current == id,
        None => scope.order.peek().first() == Some(&id),
    };
    Some(ToolbarItem { id, stop, scope })
}
