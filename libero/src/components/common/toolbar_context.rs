use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::components::common::{Orientation, attr};

static NEXT_ITEM: AtomicU64 = AtomicU64::new(0);

/// Marks a `Toolbar`'s items, valued with the item's id: what its arrow keys walk.
pub(crate) const TOOLBAR_ITEM: &str = "data-toolbar-item";

/// One `Toolbar`'s roving tab stop, shared with the controls below it.
#[derive(Clone, Copy)]
pub(crate) struct ToolbarScope {
    /// Items in mount order. Where it can query, the bar moves the stop to the first in DOM
    /// order; a WebView walks this order, so a child shown later lands at the end (todo 5167).
    order: CopyValue<Vec<u64>>,
    /// The item focused last; `None` makes the first one the tab stop.
    current: Signal<Option<u64>>,
    /// Registered items that take no focus now (hard-disabled): never the stop.
    out: Signal<Vec<u64>>,
    orientation: Signal<Orientation>,
    /// An item left the arrow at its end to the bar, for the press bubbling now.
    passed: CopyValue<bool>,
}

impl ToolbarScope {
    /// Whether an item passed the press on, cleared by the asking.
    pub(crate) fn take_passed(&self) -> bool {
        let mut passed = self.passed;
        std::mem::take(&mut *passed.write())
    }

    pub(crate) fn orientation(&self) -> Orientation {
        *self.orientation.read()
    }

    /// The registered items, in mount order.
    pub(crate) fn items(&self) -> Vec<u64> {
        self.order.peek().clone()
    }

    /// The tab stop: the item focused last, else the first that joins.
    pub(crate) fn stop(&self) -> Option<u64> {
        resolve_stop(*self.current.peek(), &self.order.peek(), &self.out.peek())
    }

    /// Subscribes the caller to the stop and to items leaving the order.
    pub(crate) fn watch(&self) {
        let _ = self.current.read();
        let _ = self.out.read();
    }

    /// Whether an item was focused or picked as the stop, not just first by default.
    pub(crate) fn chosen(&self) -> bool {
        self.current.peek().is_some()
    }

    /// `id` took focus, so it becomes the tab stop.
    pub(crate) fn focused(&self, id: u64) {
        let mut current = self.current;
        if *current.peek() != Some(id) {
            current.set(Some(id));
        }
    }

    /// The bar found `id` the item to hold the stop: it moves there unless it is there.
    pub(crate) fn settle(&self, id: u64) {
        if self.stop() != Some(id) {
            self.focused(id);
        }
    }

    /// An unmounted tab stop hands the stop to the first item left. A value borrowed now
    /// retries after this turn; a dropped bar has nothing left to update (todo 5168).
    fn leave(&self, id: u64) {
        if self.try_leave(id).is_err() {
            let scope = *self;
            dioxus::core::spawn_forever(async move {
                let _ = scope.try_leave(id);
            });
        }
    }

    fn try_leave(&self, id: u64) -> Result<(), ()> {
        let mut order = self.order;
        let mut out = self.out;
        let mut current = self.current;
        let stop = current.try_peek().map(|stop| *stop).map_err(drop)?;
        order.try_write().map_err(drop)?.retain(|&item| item != id);
        if out.try_peek().map_err(drop)?.contains(&id) {
            out.try_write().map_err(drop)?.retain(|&item| item != id);
        }
        // `None` falls to the first item left.
        if stop == Some(id) {
            current.set(None);
        }
        Ok(())
    }
}

/// The stop among `order`: `current` unless it is `out`, else the first item not `out`.
fn resolve_stop(current: Option<u64>, order: &[u64], out: &[u64]) -> Option<u64> {
    current
        .filter(|current| !out.contains(current))
        .or_else(|| order.iter().copied().find(|item| !out.contains(item)))
}

/// The nearest `Toolbar`, or `None` below an item that keeps its parts out.
#[derive(Clone, Copy)]
struct ToolbarContext(Option<ToolbarScope>);

/// Makes the controls below one `Toolbar`'s items.
pub(crate) fn use_provide_toolbar(orientation: Orientation) -> ToolbarScope {
    let scope = use_hook(|| ToolbarScope {
        order: CopyValue::new(Vec::new()),
        current: Signal::new(None),
        out: Signal::new(Vec::new()),
        orientation: Signal::new(orientation),
        passed: CopyValue::new(false),
    });
    use_context_provider(|| ToolbarContext(Some(scope)));
    // From an effect: a write in the body re-runs every reader mid-render (todo 5169).
    use_effect(use_reactive!(|orientation| {
        let mut current = scope.orientation;
        if *current.peek() != orientation {
            current.set(orientation);
        }
    }));
    scope
}

/// The enclosing `Toolbar`, `None` outside one.
pub(crate) fn use_toolbar() -> Option<ToolbarScope> {
    use_hook(|| try_consume_context::<ToolbarContext>().and_then(|context| context.0))
}

/// Keeps the controls below this one out of an enclosing `Toolbar`'s arrow
/// order: a `NumberField`'s steppers belong to it, not the bar.
pub(crate) fn use_no_toolbar() {
    use_context_provider(|| ToolbarContext(None));
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

    /// An item that steps on arrows itself leaves this one, at its end, to the bar.
    pub(crate) fn pass_on(self) {
        let mut passed = self.scope.passed;
        passed.set(true);
    }

    /// The toolbar's arrow axis.
    pub(crate) fn orientation(self) -> Orientation {
        self.scope.orientation()
    }

    /// Keeps this item in the arrow order while `joins`; out of it (hard-disabled), it
    /// takes no focus and leaves the tab stop to the next item (todo 5166).
    pub(crate) fn join(self, joins: bool) -> Option<Self> {
        let out = self.scope.out;
        let id = self.id;
        if out.peek().contains(&id) == joins {
            // After this render, as a write in render re-runs every reader.
            spawn(async move {
                let mut out = out;
                let listed = out.peek().contains(&id);
                if listed && joins {
                    out.write().retain(|&item| item != id);
                } else if !listed && !joins {
                    out.write().push(id);
                }
            });
        }
        joins.then_some(self)
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
    let current = *scope.current.read();
    let stop = resolve_stop(current, &scope.order.peek(), &scope.out.read()) == Some(id);
    Some(ToolbarItem { id, stop, scope })
}

#[cfg(test)]
mod tests {
    use super::resolve_stop;

    #[test]
    fn the_stop_is_the_focused_item_else_the_first() {
        assert_eq!(resolve_stop(Some(3), &[1, 2, 3], &[]), Some(3));
        assert_eq!(resolve_stop(None, &[1, 2, 3], &[]), Some(1));
        assert_eq!(resolve_stop(None, &[], &[]), None);
    }

    #[test]
    fn a_hard_disabled_item_is_never_the_stop() {
        assert_eq!(resolve_stop(None, &[1, 2, 3], &[1]), Some(2));
        assert_eq!(resolve_stop(Some(2), &[1, 2, 3], &[2]), Some(1));
        assert_eq!(resolve_stop(None, &[1], &[1]), None);
    }
}
