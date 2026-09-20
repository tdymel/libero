use dioxus::prelude::*;

use crate::components::common::attr;

pub(crate) fn listbox_id(id: &str) -> String {
    format!("{id}-listbox")
}

pub(crate) fn option_id(id: &str, index: usize) -> String {
    format!("{id}-option-{index}")
}

/// A group heading's id, keyed on its first row: a position would shift when the filter
/// empties a group.
pub(crate) fn group_id(id: &str, first_row: usize) -> String {
    format!("{id}-group-{first_row}")
}

/// The trigger's half of the listbox wiring. `aria-controls` only while `listbox` is
/// mounted: an id that names nothing is an invalid reference.
pub(crate) fn trigger_aria(
    id: &str,
    opened: bool,
    listbox: bool,
    active: Option<usize>,
) -> Vec<Attribute> {
    let mut attributes = vec![
        attr("role", "combobox"),
        attr("aria-haspopup", "listbox"),
        attr("aria-expanded", opened.to_string()),
    ];
    if listbox {
        attributes.push(attr("aria-controls", listbox_id(id)));
    }
    if let Some(active) = active {
        attributes.push(attr("aria-activedescendant", option_id(id, active)));
    }
    attributes
}
