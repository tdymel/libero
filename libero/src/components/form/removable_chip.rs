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
    /// Takes a render's list; its change is said only when it carries out the `edited` effect,
    /// the note whenever it is new. Any change uses `edited` up, so a refused edit vouches for no later list.
    fn observe(
        &mut self,
        labels: Vec<String>,
        edited: &mut Option<Effect>,
        note: Option<(u64, String)>,
        words: &ChipsLabels,
    ) {
        let mut change = None;
        if self.labels != labels {
            if edited
                .take()
                .is_some_and(|effect| effect == Effect::between(&self.labels, &labels))
            {
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

/// The labels an edit added and removed, case-folded and trimmed, so a host that sorts or
/// normalises its list still carries it out; a rename is one of each.
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Effect {
    added: Vec<String>,
    removed: Vec<String>,
}

impl Effect {
    fn between(before: &[String], after: &[String]) -> Self {
        let fold = |labels: &[String]| -> Vec<String> {
            labels
                .iter()
                .map(|label| label.trim().to_lowercase())
                .collect()
        };
        let (mut added, mut removed) = diff(&fold(before), &fold(after));
        added.sort();
        removed.sort();
        Self { added, removed }
    }
}

/// Set by a field right before it emits its own edit, with its list and the one it sends; the next
/// list change is announced if it does the same, however late it lands. Any other list stays silent.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Edited(CopyValue<Option<Effect>>);

impl Edited {
    pub(crate) fn mark(self, before: &[String], after: &[String]) {
        let mut edited = self.0;
        edited.set(Some(Effect::between(before, after)));
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
fn chip_change(before: &[String], after: &[String], words: &ChipsLabels) -> Option<String> {
    let (added, removed) = diff(before, after);
    let (added, removed) = (added.join(", "), removed.join(", "));
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

/// The labels `after` gained and `before` lost. Counted, not set-compared, so a duplicate's removal is heard.
fn diff(before: &[String], after: &[String]) -> (Vec<String>, Vec<String>) {
    let mut removed = Vec::new();
    let mut added = after.to_vec();
    for label in before {
        match added.iter().position(|held| held == label) {
            Some(at) => {
                added.remove(at);
            }
            None => removed.push(label.clone()),
        }
    }
    (added, removed)
}

#[cfg(test)]
mod tests {
    use super::{Announced, Effect, chip_change};
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

    fn edit(before: &[&str], after: &[&str]) -> Option<Effect> {
        Some(Effect::between(&labels(before), &labels(after)))
    }

    fn render(announced: &mut Announced, values: &[&str], mark: &mut Option<Effect>) {
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
        render(&mut announced, &["a", "b"], &mut edit(&["a"], &["a", "b"]));
        assert_eq!(announced.message.as_deref(), Some("Added b"));
        render(&mut announced, &["a"], &mut edit(&["a", "b"], &["a"]));
        assert_eq!(announced.message.as_deref(), Some("Removed b"));
    }

    #[test]
    fn an_edit_landing_renders_late_is_still_said() {
        let mut announced = announced(&["a"]);
        let mut mark = edit(&["a"], &["a", "b"]);
        // The field redraws (its draft cleared) before the caller's awaited list arrives.
        render(&mut announced, &["a"], &mut mark);
        render(&mut announced, &["a"], &mut mark);
        assert_eq!(mark, edit(&["a"], &["a", "b"]));
        assert_eq!(announced.count, 0);
        render(&mut announced, &["a", "b"], &mut mark);
        assert_eq!(mark, None);
        assert_eq!(announced.message.as_deref(), Some("Added b"));
    }

    #[test]
    fn a_refused_edit_vouches_for_no_other_list() {
        let mut announced = announced(&["a"]);
        let mut mark = edit(&["a"], &["a", "b"]);
        // The caller kept its list, then loaded another: silent, and the mark is spent.
        render(&mut announced, &["a"], &mut mark);
        render(&mut announced, &["c"], &mut mark);
        assert_eq!(mark, None);
        render(&mut announced, &["a", "b"], &mut mark);
        assert_eq!(announced.message, None);
    }

    #[test]
    fn a_sorting_or_normalising_host_still_carries_the_edit() {
        // MultiSelect holds [b], the user picks a, the host stores its option order.
        let mut announced = announced(&["b"]);
        render(&mut announced, &["a", "b"], &mut edit(&["b"], &["b", "a"]));
        assert_eq!(announced.message.as_deref(), Some("Added a"));
        // A host lowercasing and trimming tags still did what the edit asked.
        render(
            &mut announced,
            &["a", "b", "new"],
            &mut edit(&["a", "b"], &["a", "b", " New"]),
        );
        assert_eq!(announced.message.as_deref(), Some("Added new"));
        render(
            &mut announced,
            &["a", "renamed", "new"],
            &mut edit(&["a", "b", "new"], &["a", "Renamed", "new"]),
        );
        assert_eq!(
            announced.message.as_deref(),
            Some("Added renamed. Removed b")
        );
    }

    #[test]
    fn a_list_doing_something_else_is_silent_and_drops_the_mark() {
        let mut announced = announced(&["a"]);
        let mut mark = edit(&["a"], &["a", "b"]);
        render(&mut announced, &["a", "c"], &mut mark);
        assert_eq!((announced.count, &mark), (0, &None));
        // A later sync that happens to match the refused edit is not the field's.
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
