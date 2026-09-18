use dioxus::prelude::*;

use crate::{
    components::{
        common::{group_id, listbox_id},
        feedback::Loader,
        layout::ScrollArea,
    },
    sx::sx,
    theme::Size,
};

use super::option::{ComboboxContext, ComboboxRowContext};

/// Publishes where this row sits, so `ComboboxOption` needs no props for its
/// `id` or its highlight. A `Signal`, written during render: a provider runs
/// once, but `active` moves with the arrow keys.
#[component]
fn ComboboxRow(index: usize, active: bool, disabled: bool, children: Element) -> Element {
    let mut context = use_context_provider(|| {
        Signal::new(ComboboxRowContext {
            index,
            active,
            disabled,
        })
    });
    let next = ComboboxRowContext {
        index,
        active,
        disabled,
    };
    if *context.peek() != next {
        context.set(next);
    }

    children
}

/// The scrolling option list. The rows arrive drawn, which is what erases the
/// `Combobox`'s `T` - and what keeps them from being memoized.
#[component]
pub(super) fn ComboboxDropdown(
    rows: Vec<Element>,
    active: Option<usize>,
    id: String,
    max_height: String,
    scroll_y: Option<f64>,
    empty: Option<Element>,
    /// The options are being fetched.
    loading: bool,
    /// One group label per row, parallel to `rows`, `None` for a row in no
    /// group. Adjacent rows sharing a label are drawn as one `role="group"`.
    ///
    /// Parallel rather than nested, because a search filter drops rows out of
    /// the middle: the list arrives grouped and the filter only ever removes,
    /// so a group's survivors are still next to each other and the runs are
    /// simply shorter. Nothing is remapped, and a row's index stays its index
    /// in the full list.
    groups: Vec<Option<String>>,
    /// One flag per row, parallel to `rows`. A disabled row is drawn and read
    /// out; the arrows, typeahead and the click all pass over it.
    row_disabled: Vec<bool>,
    /// Above the rows and outside the scroll, so it stays put while the list
    /// under it changes - or empties.
    header: Option<Element>,
    multiselectable: bool,
    labelled_by: Option<String>,
    /// Re-provided here, not inherited: the dropdown is portaled, so it mounts
    /// under `PortalOutlet` rather than under `ComboboxCore`, and a context
    /// resolves along the mounted chain. Without this every row loses its `id`,
    /// its highlight and its Enter target - silently, because `ComboboxOption`
    /// looks it up with `try_consume_context`.
    context: ComboboxContext,
) -> Element {
    use_context_provider(|| context);

    // No rows and no `empty` draws nothing at all - an empty bordered box is
    // not a state worth showing. A `header` is the exception: it is drawn
    // either way, because a search that matched nothing still needs its box.
    //
    // Loading wins over both. The loader is silent: `ComboboxCore`'s status
    // region, outside this `aria-busy` dropdown, is what says it.
    let list = match (loading, rows.is_empty()) {
        (true, _) => rsx! {
            Loader { size: Size::Sm, sx: sx().align_self("center") }
        },
        (false, true) => empty.unwrap_or_else(|| rsx! {}),
        (false, false) => {
            // Drawn once and indexed by the runs below, so a row is built
            // whether or not it ends up inside a group wrapper.
            let drawn: Vec<Element> = rows
                .into_iter()
                .enumerate()
                .map(|(index, row)| {
                    rsx! {
                        ComboboxRow {
                            key: "{index}",
                            index,
                            active: active == Some(index),
                            disabled: row_disabled.get(index).copied().unwrap_or(false),
                            {row}
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
                    "role": "listbox",
                    "aria-multiselectable": multiselectable.then_some("true"),
                    "aria-labelledby": labelled_by,
                    for (label , start , end) in runs {
                        // A named run is wrapped and labelled; an unnamed one
                        // is bare, which is the markup an ungrouped list has
                        // always had.
                        if let Some(label) = label {
                            div {
                                key: "group-{start}",
                                role: "group",
                                "aria-labelledby": group_id(&id, start),
                                div {
                                    id: group_id(&id, start),
                                    // Named by the group's `aria-labelledby`,
                                    // so as an element of its own it would be
                                    // read a second time.
                                    role: "presentation",
                                    "data-slot": "group-label",
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

/// The rows cut into runs of adjacent equal group labels, each
/// `(label, start, end)` with `end` exclusive.
///
/// Run-length rather than a nested list, because this is rebuilt from what the
/// search filter left behind: the labels arrive already in group order, so
/// equal neighbours are one group and a group that lost every row simply has
/// no run.
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
        // No labels at all is the shape every existing caller hands down.
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

    /// A label used again after another group is a second run, not a merge:
    /// the caller listed the rows in that order and keeps it. The two runs get
    /// separate headings, each with its own id, which is legal - nothing
    /// requires a `role="group"` to be uniquely named.
    #[test]
    fn a_label_that_comes_back_starts_a_second_run() {
        let groups = [label("A"), label("B"), label("A")];
        assert_eq!(
            group_runs(&groups, 3),
            [(label("A"), 0, 1), (label("B"), 1, 2), (label("A"), 2, 3)]
        );
    }

    /// What the search filter leaves: rows dropped out of the middle, so a run
    /// is shorter and a group that lost everything is gone. The surviving rows
    /// keep their own indices, which is what the ids are keyed on.
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
