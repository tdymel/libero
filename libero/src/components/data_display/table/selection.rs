use std::{collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use super::use_table::StateSlice;
use crate::{
    components::{accessibility::Announcer, form::Checkbox},
    localization::TableLabels,
    theme::Size,
};

/// The checkbox column: the selection, every row's key, and its live region.
#[derive(Clone)]
pub(super) struct Selection {
    pub slice: StateSlice<Vec<String>>,
    /// Every row of `data`, in source order.
    pub keys: Rc<[String]>,
    pub announcer: Announcer,
    pub labels: TableLabels,
    /// The table's, so the box scales with its text.
    pub size: Size,
}

impl Selection {
    fn set(&self, next: Vec<String>) {
        self.announcer.say((self.labels.selected)(next.len()));
        self.slice.set(next);
    }

    /// The select-all box: checked with every row selected, mixed with some.
    pub fn header_cell(&self, selected: &HashSet<&str>) -> Element {
        let count = self
            .keys
            .iter()
            .filter(|key| selected.contains(key.as_str()))
            .count();
        let all = count > 0 && count == self.keys.len();
        let this = self.clone();
        rsx! {
            th { scope: "col", "data-select": true,
                Checkbox {
                    aria_label: self.labels.select_all,
                    size: self.size,
                    checked: all,
                    indeterminate: count > 0 && !all,
                    disabled: self.keys.is_empty(),
                    onchange: move |on| this.set(toggle_all(&this.slice.read(), &this.keys, on)),
                }
            }
        }
    }

    /// `name` names the row in its box's label.
    pub fn row_cell(&self, key: String, name: &str, selected: bool) -> Element {
        let this = self.clone();
        rsx! {
            td {
                "data-select": true,
                // The box is not a row click.
                onclick: |event| event.stop_propagation(),
                Checkbox {
                    aria_label: (self.labels.select_row)(name),
                    size: self.size,
                    checked: selected,
                    onchange: move |on| this.set(toggle_row(&this.slice.read(), &key, on)),
                }
            }
        }
    }
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
