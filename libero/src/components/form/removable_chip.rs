use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::ActionIcon,
        common::{COARSE_INSET, COARSE_POINTER, Glyph, Input, coarse_hit_area_sx},
        form::Chip,
    },
    context::IconSlot,
    hooks::{current_localization, use_localization},
    localization::{ChipsLabels, fill},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CHIP_HEIGHT, CssVar, Size},
};

/// The first line's height: the chip's height less its 1px borders.
const LINE_VAR: CssVar = CssVar::new("--lsx-removable-chip-line");

/// A long label wraps and the pill grows, so no text is hidden (todo 3570); the x
/// stays on the first line. Unclipped, so the x's 24x24 hit area reaches past the pill (todo 645).
static REMOVABLE_CHIP_SX: StaticSx = StaticSx::new(|| {
    sx().min_width("0")
        .overflow("visible")
        .align_items("flex-start")
        .white_space("normal")
        .per_size(|size| {
            sx().height("auto")
                .min_height(CHIP_HEIGHT.value(size))
                .var(LINE_VAR, format!("calc({} - 2px)", CHIP_HEIGHT.value(size)))
        })
        .selector(
            "& > [data-slot='label']",
            sx().min_width("0")
                .display("flex")
                .align_items("center")
                .min_height(LINE_VAR.value())
                .overflow_wrap("anywhere"),
        )
        .selector(
            "& > [data-slot='chip-trailing']",
            // No stacking context of its own, or the next row's chip covers this x's `::after`.
            sx().min_height(LINE_VAR.value()).z_index("auto"),
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
        // 44x44 under a finger (todo 3901). The `::after` on top owns the pill's own rows,
        // so the next row's x takes no press meant for this one (4188).
        .position("relative")
        .and(coarse_hit_area_sx("::before"))
        .media(
            COARSE_POINTER,
            sx().selector(
                "::after",
                sx().content("\"\"")
                    .position("absolute")
                    .z_index("1")
                    .left(COARSE_INSET)
                    .right(COARSE_INSET)
                    .top(coarse_block_inset())
                    .bottom(coarse_block_inset()),
            ),
        )
});

/// The coarse inset, floored at the pill's edge plus 2px: the button sits centred in
/// the first line, a 1px border from the edge.
fn coarse_block_inset() -> String {
    format!(
        "max({COARSE_INSET}, calc((100% - {}) / 2 - 3px))",
        LINE_VAR.value()
    )
}

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

impl Announced {
    /// Takes a render's list; its change is said only when it equals the list `edited` expects,
    /// the note whenever it is new. Any change uses `edited` up, so a refused edit vouches for no later list.
    fn observe(
        &mut self,
        labels: Vec<String>,
        edited: &mut Option<Vec<String>>,
        note: Option<(u64, String)>,
        words: &ChipsLabels,
    ) {
        let mut change = None;
        if self.labels != labels {
            if edited.take().is_some_and(|expected| expected == labels) {
                change = chip_change(&self.labels, &labels, words);
            }
            self.labels = labels;
        }
        let note = note.filter(|(number, _)| *number != self.noted);
        if let Some((number, _)) = &note {
            self.noted = *number;
        }
        let message = match (change, note) {
            (Some(change), Some((_, note))) => Some(format!("{change} {note}")),
            (change, note) => change.or(note.map(|(_, note)| note)),
        };
        if let Some(message) = message {
            self.message = Some(message);
            self.count += 1;
        }
    }
}

/// Set by a field right before it emits its own edit, with the labels it expects; that list
/// is then announced however late it lands. A list loaded, reset or replaced from outside stays silent.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Edited(CopyValue<Option<Vec<String>>>);

impl Edited {
    pub(crate) fn mark(self, expected: Vec<String>) {
        let mut edited = self.0;
        edited.set(Some(expected));
    }
}

pub(crate) fn use_edited() -> Edited {
    Edited(use_hook(|| CopyValue::new(None)))
}

/// A polite live region saying which chips a list gained or lost, by diffing
/// labels. Always mounted: a region inserted with its text is not announced.
pub(crate) fn use_chip_announcer(labels: Vec<String>, edited: Option<Edited>) -> Element {
    use_noting_chip_announcer(labels, None, edited)
}

/// The same, also saying `note` each time its number changes, after the chips' change.
pub(crate) fn use_noting_chip_announcer(
    labels: Vec<String>,
    note: Option<(u64, String)>,
    edited: Option<Edited>,
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
    match edited {
        Some(Edited(mut mark)) => announced.observe(labels, &mut mark.write(), note, words),
        None => announced.observe(labels, &mut None, note, words),
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
    use super::{Announced, chip_change};
    use crate::localization::ChipsLabels;

    fn labels(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn announced(values: &[&str]) -> Announced {
        Announced {
            labels: labels(values),
            message: None,
            count: 0,
            noted: 0,
        }
    }

    fn render(announced: &mut Announced, values: &[&str], mark: &mut Option<Vec<String>>) {
        announced.observe(labels(values), mark, None, &ChipsLabels::ENGLISH);
    }

    #[test]
    fn a_list_swapped_from_outside_is_silent() {
        let mut announced = announced(&["a"]);
        render(&mut announced, &["a", "b", "c"], &mut None);
        assert_eq!(announced.message, None);
        assert_eq!(announced.labels, labels(&["a", "b", "c"]));
    }

    #[test]
    fn the_fields_own_edit_is_said() {
        let mut announced = announced(&["a"]);
        render(&mut announced, &["a", "b"], &mut Some(labels(&["a", "b"])));
        assert_eq!(announced.message.as_deref(), Some("Added b"));
        render(&mut announced, &["a"], &mut Some(labels(&["a"])));
        assert_eq!(announced.message.as_deref(), Some("Removed b"));
    }

    #[test]
    fn an_edit_landing_renders_late_is_still_said() {
        let mut announced = announced(&["a"]);
        let mut mark = Some(labels(&["a", "b"]));
        // The field redraws (its draft cleared) before the caller's awaited list arrives.
        render(&mut announced, &["a"], &mut mark);
        render(&mut announced, &["a"], &mut mark);
        assert_eq!(mark, Some(labels(&["a", "b"])));
        assert_eq!(announced.count, 0);
        render(&mut announced, &["a", "b"], &mut mark);
        assert_eq!(mark, None);
        assert_eq!(announced.message.as_deref(), Some("Added b"));
    }

    #[test]
    fn a_refused_edit_vouches_for_no_other_list() {
        let mut announced = announced(&["a"]);
        let mut mark = Some(labels(&["a", "b"]));
        // The caller kept its list, then loaded another: silent, and the mark is spent.
        render(&mut announced, &["a"], &mut mark);
        render(&mut announced, &["c"], &mut mark);
        assert_eq!(mark, None);
        render(&mut announced, &["a", "b"], &mut mark);
        assert_eq!(announced.message, None);
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
