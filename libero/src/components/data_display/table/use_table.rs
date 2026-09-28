use dioxus::prelude::*;

use super::{
    column_filter::{ColumnFilter, FilterLogic},
    core::TableSort,
    pinning::PinnedColumns,
    resize::ColumnWidths,
};
use crate::{theme::Size, utils::warn};

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

    pub fn is_controlled(&self) -> bool {
        self.controlled
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
    pub hidden_columns: Option<Vec<String>>,
    pub default_hidden_columns: Vec<String>,
    pub onhiddencolumnschange: Option<EventHandler<Vec<String>>>,
    pub quick_filter: Option<String>,
    pub default_quick_filter: String,
    pub onquickfilterchange: Option<EventHandler<String>>,
    pub column_filters: Option<Vec<ColumnFilter>>,
    pub default_column_filters: Vec<ColumnFilter>,
    pub oncolumnfilterschange: Option<EventHandler<Vec<ColumnFilter>>>,
    pub filter_logic: Option<FilterLogic>,
    pub default_filter_logic: FilterLogic,
    pub onfilterlogicchange: Option<EventHandler<FilterLogic>>,
    pub pinned_columns: Option<PinnedColumns>,
    pub default_pinned_columns: PinnedColumns,
    pub onpinnedcolumnschange: Option<EventHandler<PinnedColumns>>,
    pub expanded: Option<Vec<String>>,
    pub default_expanded: Vec<String>,
    pub onexpandedchange: Option<EventHandler<Vec<String>>>,
    pub column_order: Option<Vec<String>>,
    pub default_column_order: Vec<String>,
    pub oncolumnorderchange: Option<EventHandler<Vec<String>>>,
    pub column_widths: Option<ColumnWidths>,
    pub default_column_widths: ColumnWidths,
    pub oncolumnwidthschange: Option<EventHandler<ColumnWidths>>,
    pub density: Option<Size>,
    pub default_density: Option<Size>,
    pub ondensitychange: Option<EventHandler<Size>>,
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
    /// The hidden columns' headers.
    pub hidden_columns: StateSlice<Vec<String>>,
    /// The quick filter's text.
    pub quick_filter: StateSlice<String>,
    pub column_filters: StateSlice<Vec<ColumnFilter>>,
    pub filter_logic: StateSlice<FilterLogic>,
    pub pinned_columns: StateSlice<PinnedColumns>,
    /// The keys of the rows whose detail shows.
    pub expanded: StateSlice<Vec<String>>,
    /// The headers in display order; unlisted ones follow in column order.
    pub column_order: StateSlice<Vec<String>>,
    /// Resized widths in px, by header.
    pub column_widths: StateSlice<ColumnWidths>,
    /// The picked size, over `size`; `None` until one is picked.
    pub density: StateSlice<Option<Size>>,
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
    let hidden_columns = use_state_slice(
        config.hidden_columns,
        || config.default_hidden_columns,
        config.onhiddencolumnschange,
        "Table: a controlled `hidden_columns` without `onhiddencolumnschange` never changes.",
    );
    let quick_filter = use_state_slice(
        config.quick_filter,
        || config.default_quick_filter,
        config.onquickfilterchange,
        "Table: a controlled `quick_filter` without `onquickfilterchange` never changes.",
    );
    let column_filters = use_state_slice(
        config.column_filters,
        || config.default_column_filters,
        config.oncolumnfilterschange,
        "Table: a controlled `column_filters` without `oncolumnfilterschange` never changes.",
    );
    let filter_logic = use_state_slice(
        config.filter_logic,
        || config.default_filter_logic,
        config.onfilterlogicchange,
        "Table: a controlled `filter_logic` without `onfilterlogicchange` never changes.",
    );
    let pinned_columns = use_state_slice(
        config.pinned_columns,
        || config.default_pinned_columns,
        config.onpinnedcolumnschange,
        "Table: a controlled `pinned_columns` without `onpinnedcolumnschange` never changes.",
    );
    let expanded = use_state_slice(
        config.expanded,
        || config.default_expanded,
        config.onexpandedchange,
        "Table: a controlled `expanded` without `onexpandedchange` never changes.",
    );
    let column_order = use_state_slice(
        config.column_order,
        || config.default_column_order,
        config.oncolumnorderchange,
        "Table: a controlled `column_order` without `oncolumnorderchange` never changes.",
    );
    let column_widths = use_state_slice(
        config.column_widths,
        || config.default_column_widths,
        config.oncolumnwidthschange,
        "Table: a controlled `column_widths` without `oncolumnwidthschange` never changes.",
    );
    let ondensitychange = config.ondensitychange;
    let density_change = use_callback(move |density: Option<Size>| {
        if let (Some(handler), Some(density)) = (ondensitychange, density) {
            handler.call(density);
        }
    });
    let density = use_state_slice(
        config.density.map(Some),
        || config.default_density,
        ondensitychange.map(|_| density_change),
        "Table: a controlled `density` without `ondensitychange` never changes.",
    );
    TableState {
        density,
        column_order,
        column_widths,
        sort,
        selection,
        page,
        page_size,
        hidden_columns,
        quick_filter,
        column_filters,
        filter_logic,
        pinned_columns,
        expanded,
    }
}
