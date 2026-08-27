use dioxus::prelude::*;

use crate::components::common::attr;

pub(crate) fn listbox_id(id: &str) -> String {
    format!("{id}-listbox")
}

pub(crate) fn option_id(id: &str, index: usize) -> String {
    format!("{id}-option-{index}")
}

/// The trigger's half of the listbox wiring. Reached through
/// [`ComboboxState::trigger`](crate::hooks::ComboboxState::trigger), which is
/// the only place that knows the `id` and the active row at once.
pub(crate) fn trigger_aria(id: &str, opened: bool, active: Option<usize>) -> Vec<Attribute> {
    let mut attributes = vec![
        attr("role", "combobox"),
        attr("aria-haspopup", "listbox"),
        attr("aria-expanded", opened.to_string()),
        attr("aria-controls", listbox_id(id)),
    ];
    if let Some(active) = active {
        attributes.push(attr("aria-activedescendant", option_id(id, active)));
    }
    attributes
}
