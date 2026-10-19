use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        common::{Glyph, HtmlTag, Part, attr},
        form::{ComboboxState, PreparedField, field_control_sx},
        layout::BoxStyle,
    },
    context::IconSlot,
    hooks::ElementHandle,
    platform::blur_counts,
    sx::{StaticSx, sx},
};

use super::{core::CascaderPart, keys::CascaderKeys};

/// The chevron sits inside the control, not the frame's trailing slot, so clicking it opens the list.
pub(super) static CASCADER_TRIGGER_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .display("flex")
        .align_items("center")
        // The frame's height, not its contents' (todo 532, as 520).
        .align_self("stretch")
        .gap("4px")
        .cursor("pointer")
        .user_select("none")
        .selector(
            format!("& > [data-slot='{}']", CascaderPart::Value.slot()),
            sx().flex("1 1 auto")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap"),
        )
        .selector("& > [data-placeholder]", sx().color("text-dimmed"))
        .selector(
            "& > svg",
            sx().flex("0 0 auto")
                .width("1em")
                .height("1em")
                .color("muted.6"),
        )
        .when("disabled", sx().cursor("not-allowed"))
});

/// Only one element is the combobox: while the search box is open, the trigger keeps just `aria-haspopup`.
fn trigger_aria(
    searchable: bool,
    opened: bool,
    empty: bool,
    controlled_id: &str,
    descendant: Option<String>,
) -> Vec<Attribute> {
    if searchable && opened {
        return vec![attr("aria-haspopup", "listbox")];
    }
    let expanded = opened && !empty;
    let mut trigger = vec![
        attr("role", "combobox"),
        attr("aria-haspopup", "listbox"),
        attr("aria-expanded", expanded.to_string()),
    ];
    // The list is mounted only while open; a dangling id is invalid.
    if expanded {
        trigger.push(attr("aria-controls", controlled_id.to_string()));
    }
    if let Some(target) = descendant.filter(|_| expanded) {
        trigger.push(attr("aria-activedescendant", target));
    }
    trigger
}

/// The trigger's text: the joined path, or the placeholder.
fn value_slot(display: &str, placeholder: Option<&str>) -> Element {
    match display.is_empty() {
        false => rsx! {
            span { "data-slot": CascaderPart::Value.slot(), "{display}" }
        },
        true => {
            let placeholder = placeholder.unwrap_or_default();
            rsx! {
                span { "data-slot": CascaderPart::Value.slot(), "data-placeholder": "true", "{placeholder}" }
            }
        }
    }
}

/// The open search box is the combobox; a role-less trigger may not carry `aria-required`.
pub(super) fn trigger_control(
    control: BoxStyle,
    field: &PreparedField,
    search_open: bool,
) -> BoxStyle {
    match search_open {
        true => control
            .attr("data-slot", CascaderPart::Control.slot())
            .attr("id", field.id().to_string()),
        false => field
            .aria(control)
            .attr("aria-labelledby", field.label_id()),
    }
}

pub(super) struct Trigger {
    pub(super) element: ElementHandle,
    pub(super) state: ComboboxState,
    pub(super) open: Rc<dyn Fn(bool)>,
    pub(super) keys: Rc<CascaderKeys>,
    pub(super) disabled: bool,
    pub(super) readonly: bool,
    pub(super) searchable: bool,
    /// No option to list: open, the trigger must not claim an expanded popup.
    pub(super) empty: bool,
    /// Nothing to clear, so the trigger draws its chevron.
    pub(super) chevron: bool,
    pub(super) display: String,
    pub(super) placeholder: Option<String>,
    pub(super) controlled_id: String,
    pub(super) descendant: Option<String>,
}

/// The frame's control, and the one tab stop.
pub(super) fn cascader_trigger(
    control: BoxStyle,
    parts: Trigger,
    opened: bool,
    extra: Vec<Attribute>,
) -> Element {
    let Trigger {
        element,
        state,
        open,
        keys,
        disabled,
        readonly,
        searchable,
        empty,
        chevron,
        display,
        placeholder,
        controlled_id,
        descendant,
    } = parts;

    let mut attributes = trigger_aria(searchable, opened, empty, &controlled_id, descendant);
    attributes.extend(extra);

    let value_slot = value_slot(&display, placeholder.as_deref());

    let toggle = open;
    control
        .element(&element)
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("tabindex", (!disabled).then_some("0"))
        // Keeps the open search box focused, so the click reads the list open and closes it (todo 2294).
        .event("onmousedown", move |event: MouseEvent| {
            if searchable && state.is_open() {
                event.prevent_default();
            }
        })
        .event("onclick", move |_: MouseEvent| {
            if !disabled && !readonly {
                toggle(!state.is_open());
            }
        })
        // While searchable, focus moves to the search box, whose blur closes instead.
        .event("onblur", move |event: FocusEvent| {
            if !searchable && blur_counts(&event) {
                state.close();
            }
        })
        // Not on a frame wrapper: it would take the x's Enter and Space.
        .event("onkeydown", move |event: KeyboardEvent| keys.handle(event))
        .render(
            HtmlTag::Div,
            attributes,
            rsx! {
                {value_slot}
                if chevron {
                    Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::dioxus_core::AttributeValue;

    fn names(attributes: &[Attribute]) -> Vec<&'static str> {
        attributes.iter().map(|attribute| attribute.name).collect()
    }

    #[test]
    fn an_open_trigger_over_no_options_claims_no_popup() {
        let empty = trigger_aria(false, true, true, "id-listbox", Some("row".into()));
        assert!(!names(&empty).contains(&"aria-controls"));
        assert!(!names(&empty).contains(&"aria-activedescendant"));
        let expanded = empty.iter().find(|a| a.name == "aria-expanded").unwrap();
        assert!(matches!(&expanded.value, AttributeValue::Text(text) if text == "false"));

        let full = trigger_aria(false, true, false, "id-listbox", Some("row".into()));
        assert!(names(&full).contains(&"aria-controls"));
        let expanded = full.iter().find(|a| a.name == "aria-expanded").unwrap();
        assert!(matches!(&expanded.value, AttributeValue::Text(text) if text == "true"));
    }
}
