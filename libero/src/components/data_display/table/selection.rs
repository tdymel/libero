use std::{collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use super::use_table::StateSlice;
use crate::{
    components::{
        accessibility::Announcer,
        form::{CheckboxLook, LightState},
    },
    hooks::use_element,
    localization::TableLabels,
    platform::ElementApi,
};

/// The checkbox column: the selection, the rows select-all covers, and its live region.
#[derive(Clone)]
pub(super) struct Selection {
    pub slice: StateSlice<Vec<String>>,
    /// The rows select-all covers: those the quick filter keeps.
    pub scope: Rc<[String]>,
    pub announcer: Announcer,
    pub labels: TableLabels,
    /// The boxes, styled at the table's size so they scale with its text.
    pub look: Rc<CheckboxLook>,
}

impl Selection {
    fn set(&self, next: Vec<String>) {
        self.announcer.say((self.labels.selected)(next.len()));
        self.slice.set(next);
    }

    /// The select-all box: checked with every row selected, mixed with some.
    /// `rowspan` covers the header rows of column groups.
    pub fn header_cell(&self, selected: &HashSet<&str>, rowspan: usize) -> Element {
        let count = self
            .scope
            .iter()
            .filter(|key| selected.contains(key.as_str()))
            .count();
        let all = count > 0 && count == self.scope.len();
        let this = self.clone();
        rsx! {
            th {
                scope: "col",
                rowspan: (rowspan > 1).then(|| rowspan.to_string()),
                "data-select": true,
                SelectAll {
                    aria_label: self.labels.select_all,
                    look: self.look.clone(),
                    state: LightState {
                        checked: all,
                        indeterminate: count > 0 && !all,
                        disabled: self.scope.is_empty(),
                    },
                    onchange: move |on| this.set(toggle_all(&this.slice.read(), &this.scope, on)),
                }
            }
        }
    }

    /// Turns the row of `key` on or off, for [`Self::row_cell`]'s stable `toggle`.
    pub fn toggle(&self, key: &str, on: bool) {
        self.set(toggle_row(&self.slice.read(), key, on));
    }

    /// `name` names the row in its box's label; `toggle` is [`Self::toggle`] behind a `use_callback`.
    pub fn row_cell(
        &self,
        key: String,
        name: &str,
        selected: bool,
        toggle: Callback<(String, bool)>,
    ) -> Element {
        rsx! {
            RowSelect {
                row: key,
                aria_label: (self.labels.select_row)(name),
                look: self.look.clone(),
                checked: selected,
                toggle,
            }
        }
    }
}

/// A row's box, its own scope so an unchanged row skips its render (todo 1181).
/// The shared look keeps a first render to one hook (todo 1195).
#[component]
fn RowSelect(
    row: String,
    aria_label: String,
    look: Rc<CheckboxLook>,
    checked: bool,
    toggle: Callback<(String, bool)>,
) -> Element {
    let element = use_element();
    let state = LightState {
        checked,
        indeterminate: false,
        disabled: false,
    };
    rsx! {
        td {
            "data-select": true,
            // The box is not a row click. Stopped, it no longer reaches Blitz's focus
            // guard, whose default would clear the focus the box gave its input (todo 1406).
            onclick: |event| {
                event.stop_propagation();
                event.prevent_default();
            },
            {look.render(element, state, aria_label, move |on| toggle((row.clone(), on)))}
        }
    }
}

/// The header's box. A browser reads a native checkbox's mixed state from the property alone.
#[component]
fn SelectAll(
    aria_label: String,
    look: Rc<CheckboxLook>,
    state: LightState,
    onchange: EventHandler<bool>,
) -> Element {
    let element = use_element();
    let indeterminate = state.indeterminate;
    use_effect(use_reactive!(|indeterminate| {
        let _ = element.set_indeterminate(indeterminate);
    }));
    look.render(element, state, aria_label, move |on| onchange.call(on))
}

/// `selection` with `key` in or out, the others in their order.
pub(super) fn toggle_row(selection: &[String], key: &str, on: bool) -> Vec<String> {
    let mut next: Vec<String> = selection.iter().filter(|k| *k != key).cloned().collect();
    if on {
        next.push(key.to_string());
    }
    next
}

/// `selection` with every one of `keys` in or out. Keys of rows not in `keys`,
/// as from another page of a server, stay.
pub(super) fn toggle_all(selection: &[String], keys: &[String], on: bool) -> Vec<String> {
    let mut next: Vec<String> = match on {
        true => selection.to_vec(),
        false => {
            let rows: HashSet<&str> = keys.iter().map(String::as_str).collect();
            return selection
                .iter()
                .filter(|key| !rows.contains(key.as_str()))
                .cloned()
                .collect();
        }
    };
    let had: HashSet<&str> = selection.iter().map(String::as_str).collect();
    next.extend(
        keys.iter()
            .filter(|key| !had.contains(key.as_str()))
            .cloned(),
    );
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|key| key.to_string()).collect()
    }

    #[test]
    fn a_row_toggles_in_and_out_keeping_the_others() {
        let selection = keys(&["a", "c"]);

        assert_eq!(toggle_row(&selection, "b", true), keys(&["a", "c", "b"]));
        assert_eq!(toggle_row(&selection, "a", false), keys(&["c"]));
        assert_eq!(toggle_row(&selection, "a", true), keys(&["c", "a"]));
    }

    #[test]
    fn select_all_adds_the_missing_rows_and_keeps_foreign_keys() {
        let rows = keys(&["a", "b", "c"]);
        let selection = keys(&["x", "b"]);

        assert_eq!(
            toggle_all(&selection, &rows, true),
            keys(&["x", "b", "a", "c"])
        );
        assert_eq!(toggle_all(&selection, &rows, false), keys(&["x"]));
    }
}
