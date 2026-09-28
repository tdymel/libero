use std::time::Duration;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{selection::toggle_row, use_table::StateSlice};
use crate::{
    components::{accessibility::VisuallyHidden, common::Glyph, layout::Collapse},
    context::IconSlot,
    hooks::{use_presence, use_theme},
    localization::TableLabels,
};

/// The column of detail toggles, with `row_detail`.
#[derive(Clone)]
pub(super) struct Details {
    pub slice: StateSlice<Vec<String>>,
    pub labels: TableLabels,
    /// Prefixes each detail row's id.
    pub id: String,
}

impl Details {
    /// Named for a screen reader only: the toggles say what they do. `rowspan`
    /// covers the header rows of column groups.
    pub fn header_cell(&self, rowspan: usize) -> Element {
        rsx! {
            th {
                scope: "col",
                rowspan: (rowspan > 1).then(|| rowspan.to_string()),
                "data-detail-toggle": true,
                VisuallyHidden { "{self.labels.details}" }
            }
        }
    }

    /// Opens or closes the detail of the row of `key`.
    pub fn toggle(&self, key: &str, open: bool) {
        self.slice.set(toggle_row(&self.slice.read(), key, open));
    }

    /// The id of the detail row of the row at `index` in `data`.
    pub fn row_id(&self, index: usize) -> String {
        format!("{}-detail-{index}", self.id)
    }

    /// The row's toggle, or an empty cell when the row has no detail.
    pub fn row_cell(
        &self,
        key: String,
        name: &str,
        index: usize,
        open: Option<bool>,
        toggle: Callback<(String, bool)>,
    ) -> Element {
        match open {
            Some(open) => rsx! {
                DetailToggle {
                    row: key,
                    aria_label: (self.labels.row_details)(name),
                    open,
                    controls: self.row_id(index),
                    toggle,
                }
            },
            None => rsx! {
                td { "data-detail-toggle": true }
            },
        }
    }
}

/// A row's toggle, its own scope so an unchanged row skips its render.
#[component]
fn DetailToggle(
    row: String,
    aria_label: String,
    open: bool,
    controls: String,
    toggle: Callback<(String, bool)>,
) -> Element {
    rsx! {
        td {
            "data-detail-toggle": true,
            // The toggle is not a row click.
            onclick: |event| event.stop_propagation(),
            button {
                r#type: "button",
                "data-detail-button": true,
                aria_label,
                aria_expanded: "{open}",
                // The detail row exists only while open.
                aria_controls: open.then_some(controls),
                onclick: move |_| toggle((row.clone(), !open)),
                Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
            }
        }
    }
}

/// A detail row that slides open and shut, with `animate_details`: it stays
/// through its close, showing the body it last had.
#[component]
pub(super) fn SlidingDetail(
    id: String,
    open: bool,
    body: Option<Element>,
    columns: usize,
) -> Element {
    let duration = use_theme().collapse.duration;
    // The `Collapse` inside ends on this property; its `transitionend` bubbles here.
    let presence = use_presence(
        open,
        "grid-template-rows",
        Some(Duration::from_millis(duration.into())),
    );
    let mut last = use_hook(|| CopyValue::new(None::<Element>));
    if body.is_some() {
        last.set(body);
    }
    if !presence.mounted() {
        return rsx! {};
    }
    rsx! {
        tr {
            id,
            "data-detail": true,
            "data-sliding": true,
            onmounted: move |_| presence.on_mounted(),
            ontransitionend: move |event| presence.on_transition_end(&event),
            td { colspan: "{columns}",
                Collapse { open,
                    div { "data-detail-body": true, {last.peek().clone()} }
                }
            }
        }
    }
}
