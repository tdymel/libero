use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{ActionIcon, Chip, Input, VisuallyHidden, common::CloseIcon},
    hooks::{current_localization, use_localization},
    localization::{ChipsLabels, fill},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CHIP_HEIGHT, Size},
};

/// Capped at its slot by `Chip`'s own `max-width`, and allowed below its
/// content: a flex item's floor is otherwise its min-content size. `Chip`
/// ellipsizes the label and keeps its `trailing` x whole.
static REMOVABLE_CHIP_SX: StaticSx = StaticSx::new(|| sx().min_width("0"));

/// The x. A native `<button>` inherits neither `color` nor `font-size` - it
/// takes the UA's `buttontext` and 13.3px.
///
/// The hover tint is `currentColor` at 20%, so it reads on a filled chip and a
/// tonal one alike without either knowing the other's colour.
static REMOVE_BUTTON_SX: StaticSx = StaticSx::new(|| {
    sx().color("inherit")
        .font_size("inherit")
        .border_radius("50%")
        .selector(
            "&:hover",
            sx().background("color-mix(in srgb, currentColor 20%, transparent)"),
        )
});

/// The default chip of a field that holds a list - `MultiSelect`'s selection,
/// `TagsField`'s tags, a `multiple` `FileField`'s files: the label, and an x
/// that drops it.
///
/// `size` is the field's; chips ride inside the control, so they sit one step
/// down the same scale.
///
/// A plain function, not a component: `remove` is a `Callback`, which always
/// compares equal, so a memoized chip would keep dropping the index it was
/// first drawn with ([[codebase/dioxus-memoization-traps]]).
pub(crate) fn removable_chip(
    label: String,
    remove: Callback<()>,
    size: Size,
    disabled: bool,
) -> Element {
    let size = size.step_down();
    // A fraction of the chip's own height, not of its font: the two do not
    // scale at the same rate (20 -> 36px against 11 -> 15px), so an `em` x
    // shrinks against its chip as the field grows.
    let icon_size: Input<ThemeAwareValue> =
        ThemeAwareValue::String(format!("calc({} * 0.6)", CHIP_HEIGHT.value(size))).into();
    let remove_label = fill(current_localization().common.remove, &[("label", &label)]);
    rsx! {
        Chip { size, sx: &REMOVABLE_CHIP_SX,
            trailing: rsx! { span {
                "data-slot": "remove",
                // Load-bearing twice, and only the first reason is obvious. The
                // field holds the focus that keeps its list open, so the press
                // must not move it onto the button. And the button is about to
                // be removed: a focused one would take the focus down with it,
                // to the body ([[principles/focus-after-removal]]). With this,
                // no mouse removal moves focus at all, which is why no
                // field repairs it. Tested in `tests/all/events.rs`.
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                // A remove is not a click on the field, which would open the
                // list under the chip that just went away.
                onclick: move |event: MouseEvent| event.stop_propagation(),
                ActionIcon {
                    aria_label: remove_label,
                    size: icon_size,
                    disabled,
                    sx: &REMOVE_BUTTON_SX,
                    // The field is one tab stop: its keys, not a button per
                    // chip, are how a keyboard removes one.
                    tabindex: "-1",
                    onclick: move |_| remove.call(()),
                    CloseIcon {}
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
    /// Bumped per message, so a sentence equal to the last one still arrives
    /// as a new node - a text node rewritten to the same text fires nothing.
    count: u64,
}

/// A polite live region saying which chips a list just gained or lost.
///
/// It diffs the labels it is handed against the ones it saw last, so every
/// path that edits the list - a pick, a typed tag, a paste, Backspace, an x,
/// the clear button, the caller's own code - is announced by one mechanism,
/// once per change. Always mounted: a region inserted together with its text
/// is not announced. Call it unconditionally; it is a hook.
pub(crate) fn use_chip_announcer(labels: Vec<String>) -> Element {
    let words = &use_localization().chips;
    let announced = use_hook(|| {
        Rc::new(RefCell::new(Announced {
            labels: labels.clone(),
            message: None,
            count: 0,
        }))
    });
    let mut announced = announced.borrow_mut();
    if announced.labels != labels {
        if let Some(message) = chip_change(&announced.labels, &labels, words) {
            announced.message = Some(message);
            announced.count += 1;
        }
        announced.labels = labels;
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

/// What a list gained and lost, as one sentence - `None` when it only moved.
/// Counted, not compared as sets: a list allowed to hold "a" twice lost one
/// when one of them goes.
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
