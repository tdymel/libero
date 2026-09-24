use dioxus::html::FileData;
use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, HtmlTag, Input, navigation_chord},
        feedback::Loader,
        form::{SelectionArgs, removable_chip},
        layout::BoxStyle,
    },
    context::IconSlot,
    hooks::{ElementHandle, current_formats, current_localization, id_selector},
    localization::fill,
    platform::{ElementApi, logical_key},
    sx::{ThemeAwareValue, sx},
    theme::{FileFieldVariant, Size},
};

use super::files::{Files, format_size};

/// What the next render owes the keyboard, once the control focus was on has
/// gone away. See [[principles/focus-after-removal]] in the project brain.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum FocusDebt {
    /// This row was removed; focus the one that took its place.
    Removed(usize),
    /// Files arrived; focus the first row when the surface stood down.
    Took,
}

/// After a removal, moves focus to the row that took its place (clamped), or
/// the Browse button, not the body. `focus_prefix` + index is a row's id.
pub(super) fn use_focus_debt(
    mut owed: Signal<Option<FocusDebt>>,
    remaining: usize,
    surface_survives: bool,
    list_element: ElementHandle,
    browse_element: ElementHandle,
    focus_prefix: String,
) {
    use_effect(use_reactive!(|remaining| {
        let Some(debt) = *owed.peek() else {
            return;
        };
        owed.set(None);
        let target = match debt {
            // Nothing left to remove: the Browse button is what is left.
            FocusDebt::Removed(_) if remaining == 0 => None,
            // The row that took this one's place, clamped to the new last.
            FocusDebt::Removed(index) => Some(index.min(remaining - 1)),
            // A single-file dropzone puts its surface away once it holds a
            // file, so the x that replaced it is what the keyboard needs.
            FocusDebt::Took if !surface_survives && remaining > 0 => Some(0),
            // The Browse button is still there, and still focused.
            FocusDebt::Took => return,
        };
        focus_row(list_element, browse_element, &focus_prefix, target);
    }));
}

/// Focuses row `target` by its tab stop's id under `list`, or the Browse
/// button for `None`.
fn focus_row(list: ElementHandle, browse: ElementHandle, prefix: &str, target: Option<usize>) {
    match target {
        Some(index) => {
            if let Ok(row) = list.query_selector(&id_selector(&format!("{prefix}-{index}"))) {
                let _ = row.focus();
            }
        }
        None => {
            let _ = browse.focus();
        }
    }
}

/// Draws the picked files, for one render: chips in the `Input` variant,
/// cards under a dropzone, or the caller's `selection`.
pub(super) struct FileRows {
    pub(super) draw: Option<Callback<SelectionArgs<FileData>, Element>>,
    pub(super) remove_at: Callback<usize>,
    pub(super) cards: bool,
    pub(super) multiple: bool,
    /// Draws the x at all. Off while disabled or read-only.
    pub(super) editable: bool,
    pub(super) icon_size: Input<ThemeAwareValue>,
    /// The field's; the chips step down from it themselves.
    pub(super) size: Size,
    /// The loader a card carries, once no surface is left to carry it.
    pub(super) card_loader: Option<Size>,
    pub(super) field_id: String,
    /// The chip that holds the list's one tab stop.
    pub(super) chip_cursor: Option<usize>,
    pub(super) keys: ChipKeys,
    pub(super) chip_class: Option<String>,
    pub(super) card_class: Option<String>,
}

impl FileRows {
    pub(super) fn all(&self, files: &Files) -> Vec<Element> {
        files
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, file)| self.row(index, file))
            .collect()
    }

    fn row(&self, index: usize, file: FileData) -> Element {
        let remove_at = self.remove_at;
        let remove = Callback::new(move |_: ()| remove_at.call(index));
        let content = match &self.draw {
            Some(draw) => draw.call(SelectionArgs {
                value: file.clone(),
                remove,
            }),
            None => match self.cards {
                true => default_card(
                    &file,
                    remove,
                    self.icon_size.clone(),
                    self.editable,
                    self.card_loader,
                    format!("{}-remove-{index}", self.field_id),
                ),
                false => default_chip(&file, remove, self.size, self.multiple, self.editable),
            },
        };
        match self.cards {
            true => rsx! {
                li { key: "{index}", class: self.card_class.clone(), {content} }
            },
            false => {
                let keys = self.keys.clone();
                let mut cursor = keys.cursor;
                let id = format!("{}-{index}", keys.id_prefix);
                // One tab stop for the whole list; the arrows walk the rest.
                let stop =
                    keys.interactive
                        .then_some(match self.chip_cursor.unwrap_or(0) == index {
                            true => "0",
                            false => "-1",
                        });
                rsx! {
                    li {
                        key: "{index}",
                        class: self.chip_class.clone(),
                        "data-slot": "chip",
                        id,
                        tabindex: stop,
                        // Keeps a press on an x (a caller's too) from taking
                        // the focus the removal hands on.
                        onmousedown: move |event: MouseEvent| event.prevent_default(),
                        onfocus: move |_| cursor.set(Some(index)),
                        onkeydown: move |event: KeyboardEvent| keys.handle(event, Some(index)),
                        {content}
                    }
                }
            }
        }
    }
}

/// The `Input` variant's chips, a list inside the group. `None` while empty.
pub(super) fn chip_list(drawn: Vec<Element>, list_element: ElementHandle) -> Option<Element> {
    (!drawn.is_empty()).then(|| {
        rsx! {
            // Safari with VoiceOver drops list semantics from a
            // `list-style: none` list.
            ul {
                "data-slot": "value",
                role: "list",
                onmounted: list_element.mount(),
                {drawn.into_iter()}
            }
        }
    })
}

/// The chip the keyboard is on, clamped to `count`. A dropzone has no chips,
/// so no cursor.
pub(super) fn chip_cursor(cursor: Option<usize>, count: usize, cards: bool) -> Option<usize> {
    match count {
        _ if cards => None,
        0 => None,
        count => cursor.map(|index| index.min(count - 1)),
    }
}

/// The `Input` variant's keys, on each chip and on the Browse button: the
/// arrows move the focus along the chips, Backspace and Delete remove.
#[derive(Clone)]
pub(super) struct ChipKeys {
    pub(super) interactive: bool,
    /// The keys that remove. The arrows only read, so a read-only field still
    /// walks the chips.
    pub(super) editable: bool,
    pub(super) count: usize,
    /// Removes a chip the focus is on, and owes the focus a new place.
    pub(super) remove_at: Callback<usize>,
    /// Removes a chip from the Browse button, where the focus stays.
    pub(super) drop_at: Callback<usize>,
    pub(super) list: ElementHandle,
    pub(super) browse: ElementHandle,
    pub(super) id_prefix: String,
    /// The chip that last had the focus, and so holds the tab stop.
    pub(super) cursor: Signal<Option<usize>>,
}

impl ChipKeys {
    /// `at` is the chip the focus is on, `None` for the Browse button.
    pub(super) fn handle(&self, event: KeyboardEvent, at: Option<usize>) {
        // A chord is the browser's (Alt+ArrowLeft is Back).
        if !self.interactive || self.count == 0 || navigation_chord(&event).is_some() {
            return;
        }
        let last = self.count - 1;
        let target = match (logical_key(&event), at) {
            (Key::ArrowLeft, Some(index)) => Some(index.saturating_sub(1)),
            // From the Browse button, the last chip - the one Backspace takes.
            (Key::ArrowLeft, None) => Some(last),
            // Past the last chip is the Browse button, not a wrap.
            (Key::ArrowRight, Some(index)) => (index < last).then_some(index + 1),
            (Key::Home, Some(_)) => Some(0),
            (Key::End, Some(_)) => Some(last),
            (Key::Backspace | Key::Delete, Some(index)) if self.editable => {
                event.prevent_default();
                self.remove_at.call(index);
                return;
            }
            (Key::Backspace | Key::Delete, None) if self.editable => {
                event.prevent_default();
                self.drop_at.call(last);
                return;
            }
            _ => return,
        };
        event.prevent_default();
        focus_row(self.list, self.browse, &self.id_prefix, target);
    }
}

/// The dropzone's cards, a list under the surface.
pub(super) fn card_list(
    style: BoxStyle,
    list_element: ElementHandle,
    labelledby: Option<String>,
    surface: bool,
    loading: bool,
    drawn: Vec<Element>,
) -> Element {
    style
        // With no surface left, the list is what the field's label names.
        .attr(
            "aria-labelledby",
            (!surface).then_some(labelledby).flatten(),
        )
        .attr("aria-busy", (loading && !surface).then_some("true"))
        .element(&list_element)
        // Safari with VoiceOver drops list semantics from a `list-style: none`
        // list.
        .attr("role", "list")
        .render(HtmlTag::Ul, Vec::new(), drawn)
}

/// A chip with an x for several files, the bare filename for one: the clear
/// button already removes it.
fn default_chip(
    file: &FileData,
    remove: Callback<()>,
    size: Size,
    multiple: bool,
    editable: bool,
) -> Element {
    let name = file.name();
    if !multiple {
        // `data-slot` so the control can clip it: one long filename would
        // otherwise push the frame past its parent's width.
        return rsx! {
            span { "data-slot": "name", "{name}" }
        };
    }
    // The chip `MultiSelect` and `TagsField` draw. Its press guard keeps the
    // focus on the control, so a mouse removal needs no repair.
    removable_chip(name, remove, size, !editable)
}

/// One picked file as a row under the dropzone: the name, its size, and an x
/// that is an ordinary tab stop.
fn default_card(
    file: &FileData,
    remove: Callback<()>,
    icon_size: Input<ThemeAwareValue>,
    editable: bool,
    loading: Option<Size>,
    id: String,
) -> Element {
    let name = file.name();
    let words = current_localization();
    let size = format_size(
        file.size(),
        &words.file_field,
        current_formats().decimal_separator,
    );
    let remove_label = fill(words.common.remove, &[("label", &name)]);
    rsx! {
        span { "data-slot": "name", "{name}" }
        span { "data-slot": "size", "{size}" }
        if let Some(size) = loading {
            Loader { size }
        }
        if editable {
            span { "data-slot": "remove",
                ActionIcon {
                    // The id is how the focus finds the row that takes this
                    // one's place: a list cannot hold a hook per row.
                    id,
                    aria_label: remove_label,
                    size: icon_size,
                    sx: sx()
                        .color("inherit")
                        .border_radius("50%")
                        .selector(
                            "&:hover",
                            sx().background("color-mix(in srgb, currentColor 12%, transparent)"),
                        ),
                    onclick: move |_: MouseEvent| remove.call(()),
                    Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
                }
            }
        }
    }
}

/// Which chip the keyboard is on. Only the `Input` variant has one.
pub(super) fn use_chip_cursor(
    variant: FileFieldVariant,
    count: usize,
    cards: bool,
) -> (Signal<Option<usize>>, Option<usize>) {
    let mut cursor = use_signal(|| None::<usize>);
    // Cleared on a variant switch, so coming back shows no stale chip; one
    // render late is unseen.
    use_effect(use_reactive!(|(variant,)| {
        let _ = variant;
        if cursor.peek().is_some() {
            cursor.set(None);
        }
    }));

    (cursor, chip_cursor(cursor(), count, cards))
}
