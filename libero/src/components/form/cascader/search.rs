use dioxus::prelude::*;

use crate::{
    components::{
        common::HtmlTag,
        form::{ComboboxState, PreparedField},
        layout::BoxStyle,
    },
    hooks::ElementHandle,
    platform::{ElementApi, blur_counts},
    sx::{StaticSx, sx},
};

/// The search box above the rows, as on `Select`. Not a field control, so it carries its own chrome.
pub(super) static CASCADER_SEARCH_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .border("none")
        .outline("none")
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .line_height("1.5")
        .padding("4px 8px")
        .border_bottom("1px solid")
        // The box's only boundary: 3:1, as a field frame (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        .selector("::placeholder", sx().color("text-dimmed"))
});

/// [`CASCADER_SEARCH_SX`]'s padding plus bottom border, for a drawn placeholder.
pub(super) const SEARCH_INSET: &str = "4px 8px 5px";

#[derive(Clone, Copy)]
pub(super) struct CascaderSearch {
    pub(super) element: ElementHandle,
    pub(super) query: Signal<String>,
    pub(super) cursor: Signal<Vec<usize>>,
    pub(super) state: ComboboxState,
}

/// The search box at the top of an open, `searchable` list.
pub(super) fn search_header(
    style: BoxStyle,
    search: CascaderSearch,
    controlled_id: String,
    descendant: Option<String>,
    placeholder: String,
    // The open box is the combobox, so it carries the field's label and captions.
    field: &PreparedField,
    required: bool,
) -> Element {
    let CascaderSearch {
        element,
        mut query,
        mut cursor,
        state,
    } = search;
    style
        .element(&element)
        .attr_default("type", "text")
        .attr("value", query())
        .attr("data-controlled", true)
        .attr("placeholder", placeholder)
        // Ours is the list underneath; the browser's would cover it.
        .attr("autocomplete", "off")
        .attr("aria-autocomplete", "list")
        .attr("role", "combobox")
        .attr("aria-haspopup", "listbox")
        .attr("aria-expanded", "true")
        .attr("aria-controls", controlled_id)
        .attr("aria-labelledby", field.label_id())
        .attr("aria-describedby", field.describedby())
        .attr("aria-invalid", field.invalid().then_some("true"))
        .attr("aria-required", required.then_some("true"))
        .attr("aria-activedescendant", descendant)
        .event("oninput", move |event: FormEvent| {
            query.set(event.value());
            // A new list: nothing is armed until an arrow says so.
            cursor.set(Vec::new());
        })
        // Closes while searchable; the rows cancel `mousedown`, so a click inside never blurs.
        .event("onblur", move |event: FocusEvent| {
            if blur_counts(&event) {
                state.close();
            }
        })
        // No `onkeydown`: the portaled dropdown has it, and a second pass reopened a committing Enter.
        .render(HtmlTag::Input, Vec::new(), ())
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
