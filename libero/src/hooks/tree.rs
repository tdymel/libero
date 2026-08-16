use std::collections::HashSet;

use dioxus::prelude::*;

/// Convenience state for [`crate::components::Tree`] - wires
/// `expanded`/`onexpandedchange`/`selected`/`onselectedchange` to a couple of
/// `use_signal`s so the common case doesn't need to manage a `HashSet`
/// by hand. `Tree` itself stays fully controlled (same shape as `Select`'s
/// `value`/`onchange`); this is a layer on top, not a new state paradigm.
#[derive(Clone, Copy)]
pub struct TreeState {
    expanded: Signal<HashSet<String>>,
    selected: Signal<Option<String>>,
}

impl TreeState {
    pub fn expanded(&self) -> HashSet<String> {
        self.expanded.read().clone()
    }

    pub fn is_expanded(&self, id: &str) -> bool {
        self.expanded.read().contains(id)
    }

    pub fn set_expanded(&self, expanded: HashSet<String>) {
        let mut signal = self.expanded;
        signal.set(expanded);
    }

    pub fn selected(&self) -> Option<String> {
        self.selected.read().clone()
    }

    pub fn is_selected(&self, id: &str) -> bool {
        self.selected.read().as_deref() == Some(id)
    }

    pub fn set_selected(&self, selected: Option<String>) {
        let mut signal = self.selected;
        signal.set(selected);
    }
}

pub fn use_tree_state() -> TreeState {
    let expanded = use_signal(HashSet::new);
    let selected = use_signal(|| None);

    use_hook(|| TreeState { expanded, selected })
}
