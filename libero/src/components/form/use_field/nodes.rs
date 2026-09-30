use dioxus::{core::AttributeValue, prelude::*};

use super::activation::{Activation, CLICK_BOUNDARY};
use crate::{
    components::{
        accessibility::VisuallyHidden,
        common::Part,
        form::{Caption, FieldPart, FieldStatus},
    },
    hooks::ElementHandle,
    platform::{ElementApi, nested_interactive},
};

pub(super) fn attribute_text(attributes: &[Attribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .rev()
        .find_map(|attribute| match (attribute.name, &attribute.value) {
            (found, AttributeValue::Text(value)) if found == name => Some(value.clone()),
            _ => None,
        })
}

pub(in crate::components::form) fn join_ids<const N: usize>(
    id: &str,
    slots: [(&'static str, bool); N],
) -> Option<String> {
    let present = || slots.iter().filter(|(_, present)| *present);
    let capacity: usize = present()
        .map(|(slot, _)| id.len() + slot.len() + 2)
        .sum::<usize>();
    if capacity == 0 {
        return None;
    }

    let mut names = String::with_capacity(capacity - 1);
    for (slot, _) in present() {
        if !names.is_empty() {
            names.push(' ');
        }
        names.push_str(id);
        names.push('-');
        names.push_str(slot);
    }
    Some(names)
}

pub(in crate::components::form) fn caption_content(caption: &Caption) -> Element {
    match caption {
        Caption::None => rsx! {},
        Caption::Text(text) => rsx! { "{text}" },
        Caption::Node(node) => node.clone(),
    }
}

/// `required` is `Some` on a required field, with the word to say if any.
pub(super) fn label_node(
    id: &str,
    label: &Caption,
    required: Option<Option<&'static str>>,
    labelled_by: bool,
    with_id: bool,
    activation: Option<Activation>,
    focuses: Option<ElementHandle>,
) -> Option<Element> {
    if label.is_none() {
        return None;
    }

    // `for` names only a labelable element; any other control points at the label's id.
    let named = (labelled_by || with_id).then(|| format!("{id}-label"));
    let points_at = (!labelled_by).then(|| id.to_string());
    let content = caption_content(label);
    // `aria-required` (or the spoken word) tells AT; the asterisk is decoration.
    let asterisk = required.map(|required_word| {
        rsx! {
            span { "aria-hidden": "true", "data-slot": FieldPart::Required.slot(), "*" }
            if let Some(word) = required_word {
                VisuallyHidden { " {word}" }
            }
        }
    });
    let slot = FieldPart::Label.slot();
    // One arm per listener, because a listener cannot be optional and most
    // labels need none.
    Some(match (activation, focuses) {
        (Some(activation), _) => rsx! {
            label {
                "data-slot": slot,
                id: named,
                r#for: points_at,
                onclick: activation.label_click(),
                {content}
                {asterisk}
            }
        },
        (None, Some(root)) => rsx! {
            label {
                "data-slot": slot,
                id: named.clone(),
                onclick: focus_labelled(root, named.unwrap_or_default()),
                {content}
                {asterisk}
            }
        },
        (None, None) => rsx! {
            label { "data-slot": slot, id: named, r#for: points_at,
                {content}
                {asterisk}
            }
        },
    })
}

/// The tab stop of a control that `label_id` names through `aria-labelledby`.
pub(in crate::components::form) fn labelled_focus_selector(label_id: &str) -> String {
    let named = format!("[aria-labelledby~=\"{label_id}\"]");
    format!(
        "{named}[tabindex=\"0\"], {named} input:not([type=\"hidden\"]):not(:disabled):not([tabindex=\"-1\"])"
    )
}

/// The label's click for a control it names by id: focuses the tab stop that
/// id names - the control, or its first input (`PinField`'s group).
fn focus_labelled(root: ElementHandle, label_id: String) -> impl FnMut(Event<MouseData>) + 'static {
    let selector = labelled_focus_selector(&label_id);
    move |event| {
        if nested_interactive(&event, CLICK_BOUNDARY) {
            return;
        }
        if let Ok(control) = root.query_selector(&selector) {
            let _ = control.focus();
        }
    }
}

pub(in crate::components::form) fn slot_node(
    slot: &'static str,
    id: &str,
    caption: &Caption,
) -> Option<Element> {
    if caption.is_none() {
        return None;
    }

    // Markup is not named, so it is not in `aria-describedby` either.
    let named = caption.text().is_some().then(|| format!("{id}-{slot}"));
    let content = caption_content(caption);
    Some(rsx! {
        span { "data-slot": slot, id: named, {content} }
    })
}

pub(in crate::components::form) fn status_node(
    id: &str,
    status: Option<&FieldStatus>,
    warning_word: &'static str,
) -> Option<Element> {
    let message = status.and_then(FieldStatus::message)?;
    // An error is `aria-invalid`; a warning says what it is, as GOV.UK's "Error:" does (1527).
    let warning = status.is_some_and(FieldStatus::is_warning);

    Some(rsx! {
        span { "data-slot": FieldPart::Status.slot(), id: "{id}-status",
            if warning {
                VisuallyHidden { "{warning_word} " }
            }
            "{message}"
        }
    })
}
