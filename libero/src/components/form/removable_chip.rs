use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::ActionIcon,
        common::{Glyph, Input},
        form::Chip,
    },
    context::IconSlot,
    hooks::{current_localization, use_localization},
    localization::{ChipsLabels, fill},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CHIP_HEIGHT, Size},
};

/// Shrinks below its content; the label span ellipsizes so the x stays whole.
/// Unclipped, so the x's 24x24 hit area reaches past the pill (todo 645).
static REMOVABLE_CHIP_SX: StaticSx = StaticSx::new(|| {
    sx().min_width("0").overflow("visible").selector(
        "& > [data-slot='label']",
        sx().min_width("0")
            .overflow("hidden")
            .text_overflow("ellipsis"),
    )
});

/// The x. A native `<button>` inherits neither `color` nor `font-size`; the
/// `currentColor` hover tint reads on any chip variant.
static REMOVE_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().color("inherit")
        .font_size("inherit")
        .border_radius("50%")
        .selector(
            "&:hover",
            sx().background("color-mix(in srgb, currentColor 20%, transparent)"),
        )
});

/// A list field's chip: the label and an x that drops it, one size step down.
/// Not a component: a memoized `Callback` would keep its first index ([[codebase/dioxus-memoization-traps]]).
pub(crate) fn removable_chip(
    label: String,
    remove: Callback<()>,
    size: Size,
    disabled: bool,
) -> Element {
    let size = size.step_down();
    // Of the chip's height, not its font: an `em` x shrinks as the field grows.
    let icon_size: Input<ThemeAwareValue> =
        ThemeAwareValue::String(format!("calc({} * 0.6)", CHIP_HEIGHT.value(size))).into();
    let remove_label = fill(current_localization().common.remove, &[("label", &label)]);
    rsx! {
        Chip { size, sx: &REMOVABLE_CHIP_SX,
            trailing: rsx! { span {
                "data-slot": "remove",
                // Keeps focus in the field: a focused button being removed would
                // drop it to the body ([[principles/focus-after-removal]]).
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                // Not a click on the field, which would open the list.
                onclick: move |event: MouseEvent| event.stop_propagation(),
                ActionIcon {
                    aria_label: remove_label,
                    size: icon_size,
                    disabled,
                    sx: &REMOVE_BUTTON_SX,
                    // The field is one tab stop; its keys remove a chip.
                    tabindex: "-1",
                    onclick: move |_| remove.call(()),
                    Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
                }
            } },
            span { "data-slot": "label", "{label}" }
        }
    }
}

/// The last list a chip announcer saw, and what it last said.
struct Announced {
    labels: Vec<String>,
    message: Option<String>,
    /// Bumped per message: the same text rewritten fires nothing.
    count: u64,
    /// The number of the last note said.
    noted: u64,
}

/// A polite live region saying which chips a list gained or lost, by diffing
/// labels. Always mounted: a region inserted with its text is not announced.
pub(crate) fn use_chip_announcer(labels: Vec<String>) -> Element {
    use_noting_chip_announcer(labels, None)
}

/// The same, also saying `note` each time its number changes, after the chips' change.
pub(crate) fn use_noting_chip_announcer(
    labels: Vec<String>,
    note: Option<(u64, String)>,
) -> Element {
    let words = &use_localization().chips;
    let announced = use_hook(|| {
        Rc::new(RefCell::new(Announced {
            labels: labels.clone(),
            message: None,
            count: 0,
            noted: note.as_ref().map_or(0, |(number, _)| *number),
        }))
    });
    let mut announced = announced.borrow_mut();
    let mut change = None;
    if announced.labels != labels {
        change = chip_change(&announced.labels, &labels, words);
        announced.labels = labels;
    }
    let note = note.filter(|(number, _)| *number != announced.noted);
    if let Some((number, _)) = &note {
        announced.noted = *number;
    }
    let message = match (change, note) {
        (Some(change), Some((_, note))) => Some(format!("{change} {note}")),
        (change, note) => change.or(note.map(|(_, note)| note)),
    };
    if let Some(message) = message {
        announced.message = Some(message);
        announced.count += 1;
    }
    let spoken = announced
        .message
        .clone()
        .map(|message| (announced.count, message));
    rsx! {
        VisuallyHidden { role: "status",
            for (count, message) in spoken {
                span { key: "{count}", "{message}" }
            }
        }
    }
}

/// What a list gained and lost, as one sentence; `None` when it only moved.
/// Counted, not set-compared, so a duplicate's removal is heard.
fn chip_change(before: &[String], after: &[String], words: &ChipsLabels) -> Option<String> {
    let mut removed: Vec<String> = Vec::new();
    let mut remaining = after.to_vec();
    for label in before {
        match remaining.iter().position(|held| held == label) {
            Some(at) => {
                remaining.remove(at);
            }
            None => removed.push(label.clone()),
        }
    }
    let (added, removed) = (remaining.join(", "), removed.join(", "));
    match (added.is_empty(), removed.is_empty()) {
        (true, true) => None,
        (false, true) => Some(fill(words.added, &[("labels", &added)])),
        (true, false) => Some(fill(words.removed, &[("labels", &removed)])),
        (false, false) => Some(fill(
            words.added_and_removed,
            &[("added", &added), ("removed", &removed)],
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::chip_change;
    use crate::localization::ChipsLabels;

    fn labels(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn a_change_names_what_came_and_what_went() {
        let change = |before: &[&str], after: &[&str]| {
            chip_change(&labels(before), &labels(after), &ChipsLabels::ENGLISH)
        };
        assert_eq!(change(&["a"], &["a", "b"]), Some("Added b".into()));
        assert_eq!(change(&["a", "b"], &["b"]), Some("Removed a".into()));
        assert_eq!(change(&["a", "b"], &[]), Some("Removed a, b".into()));
        assert_eq!(change(&["a"], &["b"]), Some("Added b. Removed a".into()));
        // A duplicate going is a removal, not "nothing changed".
        assert_eq!(change(&["a", "a"], &["a"]), Some("Removed a".into()));
        // Only the order moved: nothing to say.
        assert_eq!(change(&["a", "b"], &["b", "a"]), None);
    }
}
