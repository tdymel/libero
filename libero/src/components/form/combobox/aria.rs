use dioxus::prelude::*;

use crate::components::common::attr;

pub(crate) fn listbox_id(id: &str) -> String {
    format!("{id}-listbox")
}

pub(crate) fn option_id(id: &str, index: usize) -> String {
    format!("{id}-option-{index}")
}

/// A group heading's id, which the `role="group"` around it points at with
/// `aria-labelledby`. Keyed on the group's first row, not on its position in
/// the list: the search filter can empty a whole group, and an id that shifts
/// under a surviving group would re-label it.
pub(crate) fn group_id(id: &str, first_row: usize) -> String {
    format!("{id}-group-{first_row}")
}

/// The trigger's half of the listbox wiring. Reached through
/// [`ComboboxState::trigger`](crate::hooks::ComboboxState::trigger), which is
/// the only place that knows the `id` and the active row at once.
///
/// `aria-controls` only while `listbox` is mounted: a closed list is not in the
/// DOM, and an id that names nothing is an invalid reference.
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
