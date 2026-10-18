use std::rc::Rc;

use blitz_dom::BaseDocument;
use dioxus::prelude::*;

use super::FreeDoc;
use super::{ancestors, anchor, focus, node_of};
use crate::platform::keyboard::{takes_arrows, takes_typing};

/// The focused element's upper-case tag, `type` attribute, and whether it
/// sits in a `contenteditable`. Blitz sends a key press to the focused node.
fn focused_element() -> Option<(String, Option<String>, bool)> {
    let anchor = anchor()?;
    let doc = anchor.free_doc()?;
    let node = doc.get_node(doc.get_focussed_node_id()?)?;
    let element = node.element_data()?;
    let attr = |element: &blitz_dom::node::ElementData, name: &str| {
        element
            .attrs
            .iter()
            .find(|attr| *attr.name.local == *name)
            .map(|attr| attr.value.clone())
    };
    let tag = element.name.local.to_string().to_ascii_uppercase();
    let kind = attr(element, "type");
    // The nearest `contenteditable` decides, as it inherits on the web.
    let mut editable = false;
    let mut next = Some(node);
    while let Some(node) = next {
        if let Some(value) = node
            .element_data()
            .and_then(|data| attr(data, "contenteditable"))
        {
            editable = !value.eq_ignore_ascii_case("false");
            break;
        }
        next = node.parent.and_then(|parent| doc.get_node(parent));
    }
    Some((tag, kind, editable))
}

pub(in crate::platform::backend) fn typing_target() -> bool {
    focused_element()
        .is_some_and(|(tag, kind, editable)| editable || takes_typing(&tag, kind.as_deref()))
}

pub(in crate::platform::backend) fn arrow_target() -> bool {
    focused_element().is_some_and(|(tag, kind, _)| takes_arrows(&tag, kind.as_deref()))
}

pub(in crate::platform::backend) fn caret_edges() -> Option<(bool, bool)> {
    let anchor = anchor()?;
    let doc = anchor.free_doc()?;
    let node = doc.get_node(doc.get_focussed_node_id()?)?;
    let editor = &node.element_data()?.text_input_data()?.editor;
    let range = editor.raw_selection().text_range();
    let collapsed = range.is_empty();
    Some((
        collapsed && range.start == 0,
        collapsed && range.end == editor.raw_text().len(),
    ))
}

/// Whether the focused node is `mounted` or inside it.
pub(in crate::platform::backend) fn focus_is_in(mounted: &Rc<MountedData>) -> bool {
    let Some((handle, target)) = node_of(mounted) else {
        return false;
    };
    handle.free_doc().is_some_and(|doc| {
        doc.get_focussed_node_id()
            .is_some_and(|focus| ancestors(&doc, focus).any(|id| id == target))
    })
}

/// Arms the silent-focus report for a move made by hand, as [`BlitzElement::focus`](super::element::BlitzElement) does.
pub(in crate::platform::backend) fn watch_focus(doc: &BaseDocument) {
    focus::watch(doc);
}
