use dioxus::prelude::*;

use crate::{
    components::{
        common::{Part, group_id, listbox_id},
        feedback::Loader,
        form::DropdownPart,
        layout::ScrollArea,
    },
    sx::sx,
    theme::Size,
};

use super::option::{ComboboxContext, ComboboxRowContext};

/// Publishes the row's key to `ComboboxOption`. A `Signal` written in render: a provider runs once.
/// No visible index: a filter that shifts a kept row must not redraw it.
#[component]
fn ComboboxRow(row_key: usize, active: bool, disabled: bool, children: Element) -> Element {
    let mut context = use_context_provider(|| {
        Signal::new(ComboboxRowContext {
            key: row_key,
            active,
            disabled,
        })
    });
    let next = ComboboxRowContext {
        key: row_key,
        active,
        disabled,
    };
    if *context.peek() != next {
        context.set(next);
    }

    children
}

/// The scrolling option list. Rows arrive drawn, which erases `T` and prevents memoizing.
#[component]
pub(super) fn ComboboxDropdown(
    rows: Vec<Element>,
    /// One stable key per row, parallel to `rows`; empty keys by position. A kept row then keeps its scope.
    row_keys: Vec<usize>,
    active: Option<usize>,
    id: String,
    max_height: String,
    scroll_y: Option<f64>,
    empty: Option<Element>,
    loading: bool,
    /// One group label per row, parallel to `rows`: filtering only drops rows, so groups stay adjacent.
    /// Adjacent equal labels are one `role="group"`.
    groups: Vec<Option<String>>,
    /// Parallel to `rows`. A disabled row is drawn and read out, but never picked.
    row_disabled: Vec<bool>,
    /// Above the rows and outside the scroll.
    header: Option<Element>,
    multiselectable: bool,
    labelled_by: Option<String>,
    /// Re-provided: portaled under `PortalOutlet`, rows would otherwise lose it silently.
    context: ComboboxContext,
) -> Element {
    use_context_provider(|| context);

    // No rows and no `empty` draws nothing, but a `header` always draws. Loading wins over both;
    // the loader is silent, `ComboboxCore`'s status region says it.
    let list = match (loading, rows.is_empty()) {
        (true, _) => rsx! {
            Loader { size: Size::Sm, sx: sx().align_self("center") }
        },
        (false, true) => empty.unwrap_or_else(|| rsx! {}),
        (false, false) => {
            let key_of = |index: usize| row_keys.get(index).copied().unwrap_or(index);
            let drawn: Vec<Element> = rows
                .into_iter()
                .enumerate()
                .map(|(index, row)| {
                    let key = key_of(index);
                    rsx! {
                        ComboboxRow {
                            key: "{key}",
                            row_key: key,
                            active: active == Some(index),
                            disabled: row_disabled.get(index).copied().unwrap_or(false),
                            // The row itself, not `{row}` wrapped anew: an unchanged one compares equal and skips.
                            children: row,
                        }
                    }
                })
                .collect();
            let runs = group_runs(&groups, drawn.len());
            rsx! {
                ScrollArea {
                    sx: sx().max_height(max_height),
                    scroll_position_y: scroll_y,
                    id: listbox_id(&id),
                    "data-slot": DropdownPart::Listbox.slot(),
                    "role": "listbox",
                    "aria-multiselectable": multiselectable.then_some("true"),
                    "aria-labelledby": labelled_by,
                    for (label , start , end) in runs {
                        if let Some(label) = label {
                            div {
                                key: "group-{key_of(start)}",
                                "data-slot": DropdownPart::Group.slot(),
                                role: "group",
                                "aria-labelledby": group_id(&id, key_of(start)),
                                div {
                                    id: group_id(&id, key_of(start)),
                                    // Already the group's name; would be read twice.
                                    role: "presentation",
                                    "data-slot": DropdownPart::GroupLabel.slot(),
                                    "{label}"
                                }
                                {drawn[start..end].iter().cloned()}
                            }
                        } else {
                            {drawn[start..end].iter().cloned()}
                        }
                    }
                }
            }
        }
    };

    rsx! {
        {header}
        {list}
    }
}

/// Runs of adjacent equal group labels, `(label, start, end)` with `end` exclusive.
fn group_runs(groups: &[Option<String>], len: usize) -> Vec<(Option<String>, usize, usize)> {
    let mut runs: Vec<(Option<String>, usize, usize)> = Vec::new();
    for index in 0..len {
        let label = groups.get(index).cloned().flatten();
        match runs.last_mut() {
            Some((last, _, end)) if *last == label => *end = index + 1,
            _ => runs.push((label, index, index + 1)),
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(name: &str) -> Option<String> {
        Some(name.to_string())
    }

    #[test]
    fn an_ungrouped_list_is_one_unnamed_run() {
        assert_eq!(group_runs(&[None, None, None], 3), [(None, 0, 3)]);
        assert_eq!(group_runs(&[], 2), [(None, 0, 2)]);
    }

    #[test]
    fn adjacent_equal_labels_are_one_run() {
        let groups = [label("DE"), label("DE"), label("FR"), None];
        assert_eq!(
            group_runs(&groups, 4),
            [(label("DE"), 0, 2), (label("FR"), 2, 3), (None, 3, 4),]
        );
    }

    /// A returning label is a second run, not a merge: the caller's order is kept.
    #[test]
    fn a_label_that_comes_back_starts_a_second_run() {
        let groups = [label("A"), label("B"), label("A")];
        assert_eq!(
            group_runs(&groups, 3),
            [(label("A"), 0, 1), (label("B"), 1, 2), (label("A"), 2, 3)]
        );
    }

    /// After a filter, runs shorten and emptied groups vanish.
    #[test]
    fn a_filtered_list_regroups_from_what_survived() {
        // "DE" lost its second row and "FR" lost all of its.
        let groups = [label("DE"), label("NL")];
        assert_eq!(
            group_runs(&groups, 2),
            [(label("DE"), 0, 1), (label("NL"), 1, 2)]
        );
    }
}
