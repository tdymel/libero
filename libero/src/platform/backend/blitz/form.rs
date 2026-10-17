use std::rc::Rc;

use blitz_dom::{BaseDocument, local_name};
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::activated;

/// A submit button, enabled or not.
fn submit_button(doc: &BaseDocument, node_id: NodeId) -> Option<&blitz_dom::node::ElementData> {
    let element = doc.get_node(node_id)?.element_data()?;
    let kind = element
        .attr(local_name!("type"))
        .map(str::to_ascii_lowercase);
    let submits = match &*element.name.local {
        "button" => kind.as_deref().is_none_or(|kind| kind == "submit"),
        "input" => matches!(kind.as_deref(), Some("submit" | "image")),
        _ => false,
    };
    submits.then_some(element)
}

fn is_submitter(doc: &BaseDocument, node_id: NodeId) -> bool {
    submit_button(doc, node_id)
        .is_some_and(|element| element.attr(local_name!("disabled")).is_none())
}

/// The nearest `form` around `node_id`, the one a control belongs to.
fn owning_form(doc: &BaseDocument, node_id: NodeId) -> Option<NodeId> {
    let mut next = doc.get_node(node_id)?.parent;
    while let Some(id) = next {
        let node = doc.get_node(id)?;
        if node
            .element_data()
            .is_some_and(|element| &*element.name.local == "form")
        {
            return Some(id);
        }
        next = node.parent;
    }
    None
}

pub(in crate::platform::backend) fn activated_submitter(form: &Rc<MountedData>) -> bool {
    let Some(form) = form.downcast::<NodeHandle>() else {
        return false;
    };
    let Some(doc) = form.try_doc() else {
        return false;
    };
    activated(&doc).is_some_and(|target| {
        is_submitter(&doc, target) && owning_form(&doc, target) == Some(form.node_id())
    })
}

/// Input types that block implicit submission when a form has no submit
/// button and more than one of them (HTML's "implicit submission").
const BLOCKING: &[&str] = &[
    "text",
    "search",
    "url",
    "tel",
    "email",
    "password",
    "date",
    "month",
    "week",
    "time",
    "datetime-local",
    "number",
];

fn blocks_implicit_submission(element: &blitz_dom::node::ElementData) -> bool {
    &*element.name.local == "input"
        && BLOCKING.contains(
            &element
                .attr(local_name!("type"))
                .unwrap_or("text")
                .to_ascii_lowercase()
                .as_str(),
        )
}

fn is_checkable(element: &blitz_dom::node::ElementData) -> bool {
    &*element.name.local == "input"
        && matches!(
            element
                .attr(local_name!("type"))
                .unwrap_or("text")
                .to_ascii_lowercase()
                .as_str(),
            "checkbox" | "radio"
        )
}

pub(in crate::platform::backend) fn implicit_submission(form: &Rc<MountedData>) -> bool {
    let Some(form) = form.downcast::<NodeHandle>() else {
        return false;
    };
    let Some(doc) = form.try_doc() else {
        return false;
    };
    let form_id = form.node_id();
    let Some(focused) = doc.get_focussed_node_id() else {
        return false;
    };
    let focused_element = doc.get_node(focused).and_then(|node| node.element_data());
    let in_field = focused_element.is_some_and(blocks_implicit_submission);
    // A checkbox or radio submits through the default button only, as in
    // Chromium: it never counts as the form's one field.
    let in_checkable = focused_element.is_some_and(is_checkable);
    if !(in_field || in_checkable) || owning_form(&doc, focused) != Some(form_id) {
        return false;
    }
    let owned = |selector: &str| -> Vec<NodeId> {
        doc.query_selector_all_in(form_id, selector)
            .map(|nodes| {
                nodes
                    .into_iter()
                    .filter(|&id| owning_form(&doc, id) == Some(form_id))
                    .collect()
            })
            .unwrap_or_default()
    };
    // The web submits through the first submit button, and not at all when
    // that one is disabled.
    let default_button = owned("button, input")
        .into_iter()
        .find(|&id| submit_button(&doc, id).is_some());
    match default_button {
        Some(button) => is_submitter(&doc, button),
        None if in_checkable => false,
        None => {
            owned("input")
                .into_iter()
                .filter(|&id| {
                    doc.get_node(id)
                        .and_then(|node| node.element_data())
                        .is_some_and(blocks_implicit_submission)
                })
                .count()
                == 1
        }
    }
}

/// A box's `checked` attribute: what dioxus last wrote, its markup default.
pub(super) fn checked_attr(element: &blitz_dom::node::ElementData) -> bool {
    element
        .attr(local_name!("checked"))
        .is_some_and(|checked| checked != "false")
}

/// The value a `<select>` posts: its selected options, else a single one's
/// first option, as the web picks. Blitz changes none on input.
fn select_values(doc: &BaseDocument, select: NodeId) -> Vec<String> {
    let Some(element) = doc.get_node(select).and_then(|node| node.element_data()) else {
        return Vec::new();
    };
    let multiple = element.attr(local_name!("multiple")).is_some();
    let Ok(options) = doc.query_selector_all_in(select, "option") else {
        return Vec::new();
    };
    let options: Vec<_> = options
        .into_iter()
        .filter_map(|id| {
            let node = doc.get_node(id)?;
            let option = node.element_data()?;
            let value = option
                .attr(local_name!("value"))
                .map(str::to_string)
                .unwrap_or_else(|| node.text_content().trim().to_string());
            let selected = option
                .attr(local_name!("selected"))
                .is_some_and(|selected| selected != "false");
            let disabled = option.attr(local_name!("disabled")).is_some();
            Some((value, selected, disabled))
        })
        .collect();
    let mut chosen = options
        .iter()
        .filter(|(_, selected, disabled)| *selected && !disabled);
    match multiple {
        true => chosen.map(|(value, ..)| value.clone()).collect(),
        false => chosen
            .next()
            .or_else(|| options.iter().find(|(_, _, disabled)| !disabled))
            .map(|(value, ..)| vec![value.clone()])
            .unwrap_or_default(),
    }
}

/// The named, enabled controls of `form`, as a submit would post them: text,
/// ticked boxes and selects. Files are left out.
pub(in crate::platform::backend) fn form_values(
    form: &Rc<MountedData>,
) -> Vec<(String, dioxus::html::FormValue)> {
    let Some(form) = form.downcast::<NodeHandle>() else {
        return Vec::new();
    };
    let Some(doc) = form.try_doc() else {
        return Vec::new();
    };
    let form_id = form.node_id();
    let Ok(controls) =
        doc.query_selector_all_in(form_id, "input[name], textarea[name], select[name]")
    else {
        return Vec::new();
    };
    controls
        .into_iter()
        .filter(|&id| owning_form(&doc, id) == Some(form_id))
        .flat_map(|id| {
            let posted = || -> Option<(String, Vec<String>)> {
                let element = doc.get_node(id)?.element_data()?;
                let name = element.attr(local_name!("name"))?.to_string();
                if element.attr(local_name!("disabled")).is_some() {
                    return None;
                }
                if &*element.name.local == "select" {
                    return Some((name, select_values(&doc, id)));
                }
                let kind = element
                    .attr(local_name!("type"))
                    .unwrap_or("text")
                    .to_ascii_lowercase();
                let value = match kind.as_str() {
                    // Blitz keeps a click's tick apart from the attribute.
                    "checkbox" | "radio" => element
                        .checkbox_input_checked()
                        .unwrap_or_else(|| checked_attr(element))
                        .then(|| {
                            element
                                .attr(local_name!("value"))
                                .unwrap_or("on")
                                .to_string()
                        })?,
                    "submit" | "image" | "button" | "reset" | "file" => return None,
                    _ => match element.text_input_data() {
                        Some(input) => input.editor.raw_text().to_string(),
                        None => element
                            .attr(local_name!("value"))
                            .unwrap_or_default()
                            .to_string(),
                    },
                };
                Some((name, vec![value]))
            };
            let (name, values) = posted().unwrap_or_default();
            values
                .into_iter()
                .map(move |value| (name.clone(), dioxus::html::FormValue::Text(value)))
        })
        .collect()
}
