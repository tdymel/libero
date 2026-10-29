use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{Part, group_id, listbox_id},
        feedback::Loader,
        form::DropdownPart,
        layout::{ScrollArea, ScrollAreaHandle, Virtualize, use_scroll_area},
    },
    platform::{ElementApi, PlatformError, next_task, when_laid_out, when_laid_out_retry},
    sx::sx,
    theme::Size,
    utils::warn,
};

use super::option::{ComboboxContext, ComboboxOption, ComboboxRowContext};

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
    aria_label: Option<String>,
    /// A windowed list's likely widest rows' content, drawn hidden to give the popover its width.
    widest: Vec<Element>,
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
    // Drawn in full, the highlight scrolls in only when out of view: a percent clipped a group heading (todo 4835).
    let reveal = match windowed.is_none() && !loading {
        true => active,
        false => None,
    };
    use_effect(use_reactive!(|reveal| {
        if reveal.is_some() {
            when_laid_out(move || reveal_active(area, scroll_y, UNLAID_TRIES));
        }
    }));
    // A newer list (a filter below the threshold, say) retires a pending follow (todo 2709).
    let mut follows = use_hook(|| CopyValue::new(0u32));
    // Unseen until the first follow lands, or the top rows flash before the highlight's (todo 2719).
    let landed = use_signal(|| false);
    use_effect(use_reactive!(|follow| {
        let run = *follows.peek() + 1;
        follows.set(run);
        let live = move || follows.try_peek().is_ok_and(|now| *now == run);
        if let Some((y, height)) = follow {
            // Natively a scroll in the render's own poll was lost (the PageDown e2e).
            when_laid_out(move || follow_highlight(area, y, height, FOLLOW_TRIES, live, landed));
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
            // Drawn whatever the window shows, so a row's `aria-describedby` never dangles (todo 3928).
            let names: Vec<(String, String)> = windowed
                .slots
                .iter()
                .filter_map(|slot| match slot {
                    Slot::Label { label, key } => Some((group_id(&id, *key), label.clone())),
                    Slot::Row { .. } => None,
                })
                .collect();
            let list_sx = match landed() {
                true => sx().max_height(max_height),
                false => sx().max_height(max_height).opacity("0"),
            };
            rsx! {
                ScrollArea {
                    sx: list_sx,
                    handle: area,
                    id: listbox_id(&id),
                    "data-slot": DropdownPart::Listbox.slot(),
                    "role": "listbox",
                    "aria-multiselectable": multiselectable.then_some("true"),
                    "aria-labelledby": labelled_by.clone(),
                    "aria-label": aria_label.clone().filter(|_| labelled_by.is_none()),
                    if !widest.is_empty() {
                        // Zero tall, so only the widest's width counts; not rows, so no id or pick.
                        div {
                            "aria-hidden": "true",
                            style: "height: 0; overflow: hidden; visibility: hidden; pointer-events: none",
                            for row in widest {
                                ComboboxOption { "role": "presentation", {row} }
                            }
                        }
                    }
                    if !names.is_empty() {
                        div {
                            "aria-hidden": "true",
                            style: "height: 0; overflow: hidden; visibility: hidden; pointer-events: none",
                            for (name_id , label) in names {
                                div { key: "{name_id}", id: name_id, "{label}" }
                            }
                        }
                    }
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
                    // The windowed list's area, so the root stays one across the switch.
                    handle: area,
                    id: listbox_id(&id),
                    "data-slot": DropdownPart::Listbox.slot(),
                    "role": "listbox",
                    "aria-multiselectable": multiselectable.then_some("true"),
                    "aria-labelledby": labelled_by.clone(),
                    "aria-label": aria_label.clone().filter(|_| labelled_by.is_none()),
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
        /// `(aria-posinset, aria-setsize)` over the whole listbox: no `role="group"` wraps a windowed run.
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
            set: (index + 1, len),
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
            // Seen only: the rows' `aria-describedby` reads the always-drawn copy (todo 3928).
            Slot::Label { label, .. } => rsx! {
                div {
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

/// Tries for an unlaid list before the reveal gives up.
const UNLAID_TRIES: u8 = 3;

/// Scrolls the list just enough to show its active row, `fallback` percent where the
/// renderer can't query below a handle (a WebView).
fn reveal_active(area: ScrollAreaHandle, fallback: Option<f64>, tries: u8) {
    let root = area.element;
    if !root.is_mounted() {
        return;
    }
    let (content, viewport) = (root.scroll_size(), root.dimensions());
    spawn(async move {
        let (Ok(content), Ok(viewport)) = (content.await, viewport.await) else {
            return;
        };
        if viewport.height <= 0.0 && tries > 0 {
            let failed = UNLAID_TRIES - tries;
            return when_laid_out_retry(failed, move || reveal_active(area, fallback, tries - 1));
        }
        // Rows that fit never scroll, and the reveal's walk would reach the page.
        if content.height <= viewport.height + 0.5 {
            return;
        }
        match root.query_selector("[role='option'][data-state~='active']") {
            Ok(row) => {
                let _ = row.scroll_into_view(false);
            }
            Err(PlatformError::Unsupported) => area.scroll_to_percent(None, fallback),
            Err(_) => {}
        }
    });
}

/// Scrolls a windowed list to `y` percent once its padding stands for every slot. A height that
/// never matches means rows off the theme's row height, which the window can't place.
fn follow_highlight(
    area: ScrollAreaHandle,
    y: Option<f64>,
    height: f64,
    tries: u8,
    live: impl Fn() -> bool + Copy + 'static,
    mut landed: Signal<bool>,
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
            return when_laid_out(move || {
                follow_highlight(area, y, height, tries - 1, live, landed)
            });
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
        if !*landed.peek() {
            // A task on, the window has redrawn for the new offset.
            next_task().await;
            if live() {
                landed.set(true);
            }
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

    /// A named group gets a label slot keyed by its first row; positions count over the whole list.
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
                row(0, (1, 4), Some(10)),
                row(1, (2, 4), Some(10)),
                row(2, (3, 4), None),
                Slot::Label {
                    label: "FR".into(),
                    key: 13
                },
                row(3, (4, 4), Some(13)),
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
