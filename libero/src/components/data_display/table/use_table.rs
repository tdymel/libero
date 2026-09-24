use dioxus::prelude::*;

use super::core::TableSort;
use crate::utils::warn;

/// One piece of table state: the caller's when controlled, else the table's own.
/// Controlled, a change only asks via `onchange`.
pub(crate) struct StateSlice<V: 'static> {
    value: Signal<V>,
    controlled: bool,
    onchange: Option<EventHandler<V>>,
}

// Hand-written: a derive would demand `V: Copy`.
impl<V> Clone for StateSlice<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V> Copy for StateSlice<V> {}

impl<V> PartialEq for StateSlice<V> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.controlled == other.controlled
            && self.onchange == other.onchange
    }
}

impl<V: Clone + PartialEq + 'static> StateSlice<V> {
    pub fn read(&self) -> V {
        self.value.read().clone()
    }

    pub fn set(mut self, next: V) {
        if !self.controlled {
            self.value.set(next.clone());
        }
        if let Some(onchange) = self.onchange {
            onchange.call(next);
        }
    }
}

/// `default` seeds the slice once; `controlled` overrides it on every render.
fn use_state_slice<V: Clone + PartialEq + 'static>(
    controlled: Option<V>,
    default: impl FnOnce() -> V,
    onchange: Option<EventHandler<V>>,
    unanswered: &'static str,
) -> StateSlice<V> {
    let is_controlled = controlled.is_some();
    let mut value = use_signal(|| controlled.clone().unwrap_or_else(default));
    // Guarded render-time write, as `Tree` does, so an unchanged value wakes nothing.
    if let Some(controlled) = controlled
        && *value.peek() != controlled
    {
        value.set(controlled);
    }
    if is_controlled && onchange.is_none() {
        warn(unanswered);
    }
    StateSlice {
        value,
        controlled: is_controlled,
        onchange,
    }
}

/// The caller's side of each slice: controlled value, seed, and change handler.
pub(crate) struct TableConfig {
    pub sort: Option<Vec<TableSort>>,
    pub default_sort: Vec<TableSort>,
    pub onsortchange: Option<EventHandler<Vec<TableSort>>>,
}

/// A table's state, one slice per feature.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TableState {
    pub sort: StateSlice<Vec<TableSort>>,
}

pub(crate) fn use_table(config: TableConfig) -> TableState {
    let sort = use_state_slice(
        config.sort,
        || config.default_sort,
        config.onsortchange,
        "Table: a controlled `sort` without `onsortchange` never changes.",
    );
    TableState { sort }
}
