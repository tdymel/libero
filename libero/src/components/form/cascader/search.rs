use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Part, inset_focus_ring_sx},
        form::{ComboboxState, DropdownPart, TouchHold, with_drawn_placeholder},
        layout::BoxStyle,
    },
    hooks::ElementHandle,
    platform::{ElementApi, blur_counts},
    sx::{StaticSx, sx},
    theme::{FOCUS_RING_WIDTH, ios_text_floor_sx},
};

use super::core::Naming;

/// The search box above the rows, as on `Select`. Not a field control, so it carries its own chrome.
pub(super) static CASCADER_SEARCH_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .border("none")
        .outline("none")
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .letter_spacing("inherit")
        .line_height("1.5")
        .and(ios_text_floor_sx("1em"))
        .padding("4px 8px")
        .border_bottom("1px solid")
        // The box's only boundary: 3:1, as a field frame (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        .selector("::placeholder", sx().color("text-dimmed"))
        // Inset: the dropdown clips an outset ring (todo 1547).
        .focus_visible(inset_focus_ring_sx(&format!(
            "calc(-1 * {})",
            FOCUS_RING_WIDTH.value()
        )))
});

/// [`CASCADER_SEARCH_SX`]'s padding plus bottom border, for a drawn placeholder.
pub(super) const SEARCH_INSET: &str = "4px 8px 5px";

#[derive(Clone, Copy)]
pub(super) struct CascaderSearch {
    pub(super) element: ElementHandle,
    pub(super) query: Signal<String>,
    pub(super) cursor: Signal<Vec<usize>>,
    pub(super) state: ComboboxState,
    pub(super) blurred: Signal<bool>,
    pub(super) hold: TouchHold,
    /// Focused, so the combobox; a held trigger keeps the role.
    pub(super) combobox: bool,
}

/// The search box at the top of an open, `searchable` list.
pub(super) fn search_header(
    style: BoxStyle,
    search: CascaderSearch,
    controlled_id: Option<String>,
    descendant: Option<String>,
    placeholder: String,
    // The open box is the combobox, so it carries the field's label and captions.
    names: &Naming,
) -> Element {
    let CascaderSearch {
        element,
        mut query,
        mut cursor,
        state,
        mut blurred,
        hold,
        combobox,
    } = search;
    let drawn = placeholder.clone();
    let (controlled_id, descendant) = match combobox {
        true => (controlled_id, descendant),
        false => (None, None),
    };
    let input = style
        .element(&element)
        .attr_default("type", "text")
        .attr("data-slot", DropdownPart::Search.slot())
        .attr("value", query())
        .attr("data-controlled", true)
        .attr("placeholder", placeholder)
        // Ours is the list underneath; the browser's would cover it.
        .attr("autocomplete", "off")
        .attr("aria-autocomplete", "list")
        .attr("role", combobox.then_some("combobox"))
        .attr("aria-haspopup", combobox.then_some("listbox"))
        // Expanded only while a listbox is there to control.
        .attr(
            "aria-expanded",
            combobox.then(|| controlled_id.is_some().to_string()),
        )
        .attr("aria-controls", controlled_id)
        .attr("aria-labelledby", names.label_id.clone())
        .attr("aria-describedby", names.describedby.clone())
        .attr("aria-invalid", names.invalid.then_some("true"))
        .attr("aria-required", names.required.then_some("true"))
        .attr("aria-activedescendant", descendant)
        .event("oninput", move |event: FormEvent| {
            query.set(event.value());
            // A new list: nothing is armed until an arrow says so.
            cursor.set(Vec::new());
        })
        // Kept from the box's `mousedown` cancel, which would stop a click placing the caret.
        .event("onmousedown", move |event: MouseEvent| {
            event.stop_propagation()
        })
        // The tap that focuses it natively, raising the keyboard; the held trigger's blur lets it.
        .event("onpointerdown", move |_: PointerEvent| hold.search_press())
        .event("onfocus", move |_: FocusEvent| hold.search_focus())
        // Closes while searchable; the rows cancel `mousedown`, so a click inside never blurs.
        .event("onblur", move |event: FocusEvent| {
            if blur_counts(&event) {
                blurred.set(true);
                state.close();
            }
        })
        // No `onkeydown`: the portaled dropdown has it, and a second pass reopened a committing Enter.
        .render(HtmlTag::Input, Vec::new(), ());
    with_drawn_placeholder(Some(&drawn), SEARCH_INSET, input)
}

/// Focuses the search box once the list is placed, the first moment it is visible and focus can take.
pub(super) fn use_focus_search(
    opened: bool,
    searchable: bool,
    placed: bool,
    search: ElementHandle,
) {
    use_effect(use_reactive!(|(opened, searchable, placed)| {
        if opened && searchable && placed {
            // Deferred: the opening click ends by focusing the trigger.
            spawn(async move {
                let _ = search.focus();
            });
        }
    }));
}
