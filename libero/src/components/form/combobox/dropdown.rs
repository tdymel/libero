use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{Part, group_id, listbox_id},
        feedback::Loader,
        form::DropdownPart,
        layout::{ScrollArea, ScrollAreaHandle, Virtualize, use_scroll_area},
    },
    platform::{ElementApi, next_task, when_laid_out},
    sx::sx,
    theme::Size,
    utils::warn,
};

use super::option::{ComboboxContext, ComboboxRowContext};

/// From this many rows the list draws only those in view (todo 2234).
pub(crate) const VIRTUAL_ROWS: usize = 200;

/// Frames to wait for the windowed list's full height before scrolling to the highlight.
const FOLLOW_TRIES: u8 = 8;

/// Publishes the row's key to `ComboboxOption`. A `Signal` written in render: a provider runs once.
/// No visible index: a filter that shifts a kept row must not redraw it.
#[component]
fn ComboboxRow(
    row_key: usize,
    active: bool,
    disabled: bool,
    /// Windowed only: `(posinset, setsize)` in its group, and the group label's key.
    #[props(default)]
    set: Option<(usize, usize)>,
    #[props(default)] group: Option<usize>,
    children: Element,
) -> Element {
    let next = ComboboxRowContext {
        key: row_key,
        active,
        disabled,
        set,
        group,
    };
    let mut context = use_context_provider(|| Signal::new(next));
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
    /// The theme's row height in px: a windowed list's pitch, so it never waits for a probe.
    row_height: f64,
    /// Re-provided: portaled under `PortalOutlet`, rows would otherwise lose it silently.
    context: ComboboxContext,
) -> Element {
    use_context_provider(|| context);
    let area = use_scroll_area();

    // Windowed, a group label is a slot of its own and the highlight scrolls by slot.
    let (rows, windowed) = match !loading && rows.len() >= VIRTUAL_ROWS {
        true => {
            let key_of = |index: usize| row_keys.get(index).copied().unwrap_or(index);
            let slots = windowed_slots(&groups, rows.len(), key_of);
            let active_slot = active.and_then(|row| {
                slots
                    .iter()
                    .position(|slot| matches!(slot, Slot::Row { index, .. } if *index == row))
            });
            let windowed = Windowed {
                keys: (0..rows.len()).map(key_of).collect(),
                rows,
                disabled: row_disabled.clone(),
                slots,
                active,
                active_slot,
                id: id.clone(),
                row_height,
            };
            (Vec::new(), Some(Rc::new(windowed)))
        }
        false => (rows, None),
    };
    let follow = windowed.as_ref().map(|windowed| {
        let last = windowed.slots.len().saturating_sub(1).max(1) as f64;
        let y = windowed.active_slot.map(|slot| slot as f64 / last * 100.0);
        (y, windowed.slots.len() as f64 * row_height)
    });
    // A newer list (a filter below the threshold, say) retires a pending follow (todo 2709).
    let mut follows = use_hook(|| CopyValue::new(0u32));
    use_effect(use_reactive!(|follow| {
        let run = *follows.peek() + 1;
        follows.set(run);
        let live = move || follows.try_peek().is_ok_and(|now| *now == run);
        if let Some((y, height)) = follow {
            // Natively a scroll in the render's own poll was lost (the PageDown e2e).
            when_laid_out(move || follow_highlight(area, y, height, FOLLOW_TRIES, live));
        }
    }));

    // No rows and no `empty` draws nothing, but a `header` always draws. Loading wins over both;
    // the loader is silent, `ComboboxCore`'s status region says it.
    let list = match (loading, windowed) {
        (true, _) => rsx! {
            Loader { size: Size::Sm, sx: sx().align_self("center") }
        },
        (false, Some(windowed)) => {
            let (count, keep) = (windowed.slots.len(), windowed.active_slot);
            let keyed = windowed.clone();
            rsx! {
                ScrollArea {
                    sx: sx().max_height(max_height),
                    handle: area,
                    id: listbox_id(&id),
                    "data-slot": DropdownPart::Listbox.slot(),
                    "role": "listbox",
                    "aria-multiselectable": multiselectable.then_some("true"),
                    "aria-labelledby": labelled_by,
                    Virtualize {
                        count,
                        item_size: Some(row_height),
                        keep_rendered: keep,
                        item_key: move |slot: usize| keyed.key(slot),
                        item: move |slot: usize| windowed.draw(slot),
                    }
                }
            }
        }
        (false, None) if rows.is_empty() => empty.unwrap_or_else(|| rsx! {}),
        (false, None) => {
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

/// One slot of a windowed list, each one row tall.
#[derive(Clone, Debug, PartialEq)]
enum Slot {
    /// A group's heading, keyed by its first row.
    Label { label: String, key: usize },
    Row {
        index: usize,
        /// `(aria-posinset, aria-setsize)` within its group.
        set: (usize, usize),
        group: Option<usize>,
    },
}

/// The rows flattened with a label slot before each named group.
fn windowed_slots(
    groups: &[Option<String>],
    len: usize,
    key_of: impl Fn(usize) -> usize,
) -> Vec<Slot> {
    let mut slots = Vec::with_capacity(len);
    for (label, start, end) in group_runs(groups, len) {
        let group = label.is_some().then(|| key_of(start));
        if let Some(label) = label {
            slots.push(Slot::Label {
                label,
                key: key_of(start),
            });
        }
        slots.extend((start..end).map(|index| Slot::Row {
            index,
            set: (index - start + 1, end - start),
            group,
        }));
    }
    slots
}

/// What a windowed list's `item` draws from: a slot is drawn only once in view.
struct Windowed {
    rows: Vec<Element>,
    keys: Vec<usize>,
    disabled: Vec<bool>,
    slots: Vec<Slot>,
    active: Option<usize>,
    active_slot: Option<usize>,
    id: String,
    row_height: f64,
}

impl Windowed {
    fn key(&self, slot: usize) -> String {
        match &self.slots[slot] {
            Slot::Label { key, .. } => format!("group-{key}"),
            Slot::Row { index, .. } => self.keys[*index].to_string(),
        }
    }

    fn draw(&self, slot: usize) -> Element {
        match &self.slots[slot] {
            // Hidden: the rows name their group by `aria-describedby`, which still reads it.
            Slot::Label { label, key } => rsx! {
                div {
                    id: group_id(&self.id, *key),
                    "aria-hidden": "true",
                    "data-slot": DropdownPart::GroupLabel.slot(),
                    style: "height: {self.row_height}px; box-sizing: border-box; display: flex; align-items: flex-end",
                    "{label}"
                }
            },
            Slot::Row { index, set, group } => rsx! {
                ComboboxRow {
                    row_key: self.keys[*index],
                    active: self.active == Some(*index),
                    disabled: self.disabled.get(*index).copied().unwrap_or(false),
                    set: Some(*set),
                    group: *group,
                    children: self.rows[*index].clone(),
                }
            },
        }
    }
}

/// Scrolls a windowed list to `y` percent once its padding stands for every slot. A height that
/// never matches means rows off the theme's row height, which the window can't place.
fn follow_highlight(
    area: ScrollAreaHandle,
    y: Option<f64>,
    height: f64,
    tries: u8,
    live: impl Fn() -> bool + Copy + 'static,
) {
    let element = area.element;
    if !element.is_mounted() || !live() {
        return;
    }
    let size = element.scroll_size();
    spawn(async move {
        let Ok(size) = size.await else {
            return;
        };
        if !live() {
            return;
        }
        let off = (size.height - height).abs() > 1.0;
        if off && tries > 0 {
            next_task().await;
            return when_laid_out(move || follow_highlight(area, y, height, tries - 1, live));
        }
        if off && cfg!(debug_assertions) {
            warn(&format!(
                "Combobox: {height}px of rows expected, {}px drawn. Above {VIRTUAL_ROWS} rows each must be one row tall.",
                size.height
            ));
        }
        if y.is_some() {
            area.scroll_to_percent(None, y);
        }
    });
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

    /// A named group gets a label slot keyed by its first row; positions count within the group.
    #[test]
    fn a_windowed_list_puts_a_label_slot_before_each_named_group() {
        let groups = [label("DE"), label("DE"), None, label("FR")];
        let slots = windowed_slots(&groups, 4, |index| index + 10);
        let row = |index, set, group| Slot::Row { index, set, group };
        assert_eq!(
            slots,
            [
                Slot::Label {
                    label: "DE".into(),
                    key: 10
                },
                row(0, (1, 2), Some(10)),
                row(1, (2, 2), Some(10)),
                row(2, (1, 1), None),
                Slot::Label {
                    label: "FR".into(),
                    key: 13
                },
                row(3, (1, 1), Some(13)),
            ]
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
