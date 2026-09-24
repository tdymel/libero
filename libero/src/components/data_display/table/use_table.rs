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

    /// `read` without subscribing.
    pub fn peek(&self) -> V {
        self.value.peek().clone()
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
    pub selection: Option<Vec<String>>,
    pub default_selection: Vec<String>,
    pub onselectionchange: Option<EventHandler<Vec<String>>>,
    pub page: Option<u32>,
    pub default_page: u32,
    pub onpagechange: Option<EventHandler<u32>>,
    pub page_size: Option<usize>,
    /// Seeds the page size; read only while paginated.
    pub default_page_size: usize,
    pub onpagesizechange: Option<EventHandler<usize>>,
}

/// A table's state, one slice per feature.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TableState {
    pub sort: StateSlice<Vec<TableSort>>,
    /// The selected rows' keys.
    pub selection: StateSlice<Vec<String>>,
    /// 1-based.
    pub page: StateSlice<u32>,
    pub page_size: StateSlice<usize>,
}

pub(crate) fn use_table(config: TableConfig) -> TableState {
    let sort = use_state_slice(
        config.sort,
        || config.default_sort,
        config.onsortchange,
        "Table: a controlled `sort` without `onsortchange` never changes.",
    );
    let selection = use_state_slice(
        config.selection,
        || config.default_selection,
        config.onselectionchange,
        "Table: a controlled `selection` without `onselectionchange` never changes.",
    );
    let page = use_state_slice(
        config.page,
        || config.default_page,
        config.onpagechange,
        "Table: a controlled `page` without `onpagechange` never changes.",
    );
    let page_size = use_state_slice(
        config.page_size,
        || config.default_page_size,
        config.onpagesizechange,
        "Table: a controlled `page_size` without `onpagesizechange` never changes.",
    );
    TableState {
        sort,
        selection,
        page,
        page_size,
    }
}
