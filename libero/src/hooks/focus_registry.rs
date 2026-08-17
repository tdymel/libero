use std::{collections::HashMap, hash::Hash};

use dioxus::prelude::*;

use super::ElementRef;

/// A keyed collection of [`ElementRef`]s - lets any descendant register the
/// element it's bound to under some key (typically its own id/index), and
/// lets an ancestor focus one of them by that key without knowing which
/// descendant currently owns it. The shape a roving-tabindex/keyboard-nav
/// pattern needs (`Tree`, and any future `Tabs`/`Menu`/`Listbox`) - many
/// candidate elements, pick one dynamically - as opposed to [`ElementRef`]
/// itself, which is for one specific, statically-known element.
///
/// Provide one per owning component instance via
/// `use_context_provider(FocusRegistry::new)` (so a nested instance of the
/// same component automatically gets its own registry instead of sharing an
/// ancestor's), and consume it in descendants via
/// `use_context::<FocusRegistry<K>>()`.
pub struct FocusRegistry<K: 'static> {
    nodes: Signal<HashMap<K, ElementRef>>,
}

impl<K> Clone for FocusRegistry<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for FocusRegistry<K> {}

impl<K: Eq + Hash + 'static> Default for FocusRegistry<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + 'static> FocusRegistry<K> {
    pub fn new() -> Self {
        Self {
            nodes: Signal::new(HashMap::new()),
        }
    }

    /// Registers `element_ref` under `key`, overwriting any previous
    /// registration for that key.
    pub fn register(&mut self, key: K, element_ref: ElementRef) {
        self.nodes.write().insert(key, element_ref);
    }

    /// Focuses the element registered under `key`, if any.
    pub fn focus(&self, key: &K) {
        if let Some(element_ref) = self.nodes.read().get(key) {
            element_ref.focus();
        }
    }
}
