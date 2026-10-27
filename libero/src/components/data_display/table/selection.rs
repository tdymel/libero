use std::{collections::HashSet, rc::Rc};

use dioxus::prelude::*;

use super::use_table::StateSlice;
use crate::{
    components::{
        accessibility::Announcer,
        form::{CheckboxLook, LightState},
    },
    hooks::{use_element, use_id},
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
    /// With `parent_key`: a row's box selects its descendants too.
    pub tree: Option<Rc<TreeSelect>>,
}

impl Selection {
    fn set(&self, next: Vec<String>) {
        let (announcer, said) = (self.announcer, (self.labels.selected)(next.len()));
        self.slice.set_then(next, move || announcer.say(said));
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
                MixedSelect {
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
        let selection = self.slice.read();
        let row = self
            .tree
            .as_ref()
            .and_then(|tree| Some(tree).zip(tree.keys.iter().position(|at| at == key)));
        self.set(match row {
            Some((tree, row)) => tree.toggle(&selection, row, on),
            None => toggle_row(&selection, key, on),
        });
    }

    /// Turns every row of `keys` on or off, as a group row's box does.
    pub fn toggle_rows(&self, keys: &[String], on: bool) {
        self.set(toggle_all(&self.slice.read(), keys, on));
    }

    /// `name` names the row in its box's label; `toggle` is [`Self::toggle`] behind a `use_callback`.
    /// A tree row's box may be `mixed`.
    pub fn row_cell(
        &self,
        key: String,
        name: &str,
        selected: bool,
        mixed: bool,
        toggle: Callback<(String, bool)>,
    ) -> Element {
        if self.tree.is_some() {
            // Always a mixed box: a swap of components would drop the clicked box's focus.
            return rsx! {
                td {
                    "data-select": true,
                    onclick: |event| {
                        event.stop_propagation();
                        event.prevent_default();
                    },
                    MixedSelect {
                        aria_label: (self.labels.select_row)(name),
                        look: self.look.clone(),
                        state: LightState {
                            checked: selected && !mixed,
                            indeterminate: mixed,
                            disabled: false,
                        },
                        onchange: move |on| toggle((key.clone(), on)),
                    }
                }
            };
        }
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
    // Found on focus, no `onmounted`: a windowed fling mounts hundreds (todo 3252).
    let id = use_id();
    let state = LightState {
        checked,
        indeterminate: false,
        disabled: false,
    };
    let input = format!("td[data-row-select=\"{}\"] input", id.peek());
    rsx! {
        td {
            "data-select": true,
            "data-row-select": id,
            // The box is not a row click. Stopped, it no longer reaches Blitz's focus
            // guard, whose default would clear the focus the box gave its input (todo 1406).
            onclick: |event| {
                event.stop_propagation();
                event.prevent_default();
            },
            {look.render_found(input, state, aria_label, move |on| toggle((row.clone(), on)))}
        }
    }
}

/// A box over many rows: the header's or a group row's. A browser reads a native
/// checkbox's mixed state from the property alone.
#[component]
pub(super) fn MixedSelect(
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

/// A group row's box: checked with each of its `rows` selected, mixed with some.
pub(super) fn group_state(rows: &[usize], selected: impl Fn(usize) -> bool) -> LightState {
    let count = rows.iter().filter(|&&index| selected(index)).count();
    let all = count > 0 && count == rows.len();
    LightState {
        checked: all,
        indeterminate: count > 0 && !all,
        disabled: rows.is_empty(),
    }
}

/// A tree's rows as its boxes select them: each row's children in the select-all scope.
pub(super) struct TreeSelect {
    keys: Rc<[String]>,
    parents: Rc<[usize]>,
    children: Vec<Vec<usize>>,
    /// Parents before their children.
    preorder: Vec<usize>,
    within: Vec<bool>,
}

impl TreeSelect {
    /// `scope` by `data` index, `None` for every row.
    pub fn new(keys: Rc<[String]>, parents: Rc<[usize]>, scope: Option<&[bool]>) -> Self {
        let within: Vec<bool> = (0..parents.len())
            .map(|row| scope.is_none_or(|scope| scope[row]))
            .collect();
        let mut children = vec![Vec::new(); parents.len()];
        let mut roots = Vec::new();
        for row in (0..parents.len()).filter(|&row| within[row]) {
            match parents[row] {
                up if up != row && within[up] => children[up].push(row),
                _ => roots.push(row),
            }
        }
        let mut preorder = Vec::with_capacity(parents.len());
        let mut stack = roots;
        stack.reverse();
        while let Some(row) = stack.pop() {
            preorder.push(row);
            stack.extend(children[row].iter().rev());
        }
        Self {
            keys,
            parents,
            children,
            preorder,
            within,
        }
    }

    /// `row`'s parent in scope.
    fn parent(&self, row: usize) -> Option<usize> {
        Some(self.parents[row]).filter(|&up| up != row && self.within[up])
    }

    /// Per row by `data` index, whether its box is mixed: some of the row and its
    /// descendants selected, not all. Out of scope, `false`.
    pub fn mixed(&self, selected: impl Fn(usize) -> bool) -> Vec<bool> {
        let (on, total) = self.counts(&selected);
        (0..self.parents.len())
            .map(|row| on[row] > 0 && on[row] < total[row])
            .collect()
    }

    /// Each row's selected and total rows, itself and its descendants in scope.
    fn counts(&self, selected: &dyn Fn(usize) -> bool) -> (Vec<usize>, Vec<usize>) {
        let mut on = vec![0; self.parents.len()];
        let mut total = vec![0; self.parents.len()];
        for &row in self.preorder.iter().rev() {
            on[row] += usize::from(selected(row));
            total[row] += 1;
            if let Some(up) = self.parent(row) {
                on[up] += on[row];
                total[up] += total[row];
            }
        }
        (on, total)
    }

    /// `selection` once row `row`'s box turned `on`: the row and its descendants in
    /// scope follow, an ancestor turns on with all its descendants, off with one off.
    pub fn toggle(&self, selection: &[String], row: usize, on: bool) -> Vec<String> {
        let mut rows = vec![row];
        let mut at = 0;
        while let Some(&next) = rows.get(at) {
            rows.extend(&self.children[next]);
            at += 1;
        }
        let mut ancestors = Vec::new();
        let mut up = row;
        while let Some(next) = self.parent(up) {
            up = next;
            ancestors.push(up);
        }
        let keys = |rows: &[usize]| -> Vec<String> {
            rows.iter().map(|&row| self.keys[row].clone()).collect()
        };
        let next = toggle_all(selection, &keys(&rows), on);
        if !on {
            return toggle_all(&next, &keys(&ancestors), false);
        }
        let had: HashSet<&str> = next.iter().map(String::as_str).collect();
        let (selected, _) = self.counts(&|row| had.contains(self.keys[row].as_str()));
        let (_, total) = self.counts(&|_| true);
        // An ancestor turned on fills the one above by one.
        let mut filled = 0;
        let mut ancestors_on = Vec::new();
        for &up in &ancestors {
            let own = usize::from(had.contains(self.keys[up].as_str()));
            if selected[up] - own + filled + 1 < total[up] {
                break;
            }
            if own == 0 {
                ancestors_on.push(up);
                filled += 1;
            }
        }
        toggle_all(&next, &keys(&ancestors_on), true)
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

    /// a > (b > c, d), e.
    fn tree(scope: Option<&[bool]>) -> TreeSelect {
        TreeSelect::new(
            keys(&["a", "b", "c", "d", "e"]).into(),
            Rc::from([0, 0, 1, 0, 4]),
            scope,
        )
    }

    #[test]
    fn a_tree_row_selects_its_descendants_and_fills_its_ancestors() {
        let tree = tree(None);
        let sorted = |mut keys: Vec<String>| {
            keys.sort();
            keys
        };
        assert_eq!(
            sorted(tree.toggle(&keys(&["e"]), 1, true)),
            keys(&["b", "c", "e"])
        );
        // With d on, b's subtree fills a.
        assert_eq!(
            sorted(tree.toggle(&keys(&["d"]), 1, true)),
            keys(&["a", "b", "c", "d"])
        );
        // Off takes the descendants and every ancestor.
        assert_eq!(
            tree.toggle(&keys(&["a", "b", "c", "d"]), 2, false),
            keys(&["d"])
        );
    }

    #[test]
    fn a_tree_box_is_mixed_with_some_of_its_rows() {
        let tree = tree(None);
        let mixed = |on: &[usize]| tree.mixed(|row| on.contains(&row));
        assert_eq!(mixed(&[2]), [true, true, false, false, false]);
        assert_eq!(mixed(&[1, 2]), [true, false, false, false, false]);
        assert_eq!(mixed(&[0]), [true, false, false, false, false]);
        assert_eq!(mixed(&[0, 1, 2, 3]), [false; 5]);
    }

    #[test]
    fn a_tree_box_covers_the_filtered_rows_only() {
        // d filtered out: b's subtree alone fills a.
        let tree = tree(Some(&[true, true, true, false, true]));
        let mut next = tree.toggle(&[], 1, true);
        next.sort();
        assert_eq!(next, keys(&["a", "b", "c"]));
        assert_eq!(tree.toggle(&[], 0, true).len(), 3);
    }

    #[test]
    fn a_group_box_is_checked_with_all_its_rows_and_mixed_with_some() {
        let state = |selected: &[usize]| {
            let state = group_state(&[1, 3, 4], |index| selected.contains(&index));
            (state.checked, state.indeterminate)
        };
        assert_eq!(state(&[]), (false, false));
        assert_eq!(state(&[0, 3]), (false, true));
        assert_eq!(state(&[1, 3, 4]), (true, false));
        assert!(group_state(&[], |_| true).disabled);
    }
}
