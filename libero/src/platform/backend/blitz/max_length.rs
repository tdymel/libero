//! Blitz's text editor ignores `maxlength` (todo 942): a press that would type
//! past it is refused here, before the editor's default action.

use blitz_dom::{Node, local_name};
use dioxus::prelude::*;

use super::anchor;

/// Refuses a character, a textarea's Enter or a paste that would run the
/// focused text control past its `maxlength`. A paste that starts under the
/// limit still overshoots: its length is not known here.
pub(super) fn key_down(event: &Event<KeyboardData>) {
    if !event.default_action_enabled() || event.is_composing() {
        return;
    }
    let Some(anchor) = anchor() else {
        return;
    };
    let over = anchor.try_doc().is_some_and(|doc| {
        let Some(node) = doc.get_focussed_node_id().and_then(|id| doc.get_node(id)) else {
            return false;
        };
        typed_units(event, node).is_some_and(|typed| over_limit(node, typed))
    });
    if over {
        event.prevent_default();
    }
}

/// How many UTF-16 units the press would type; `0` for a paste, which only
/// counts once the control is full.
fn typed_units(event: &Event<KeyboardData>, node: &Node) -> Option<usize> {
    let modifiers = event.modifiers();
    let chord = modifiers.intersects(Modifiers::CONTROL | Modifiers::META | Modifiers::SUPER);
    match event.key() {
        Key::Character(text) if chord => text.eq_ignore_ascii_case("v").then_some(0),
        Key::Character(text) => Some(text.encode_utf16().count()),
        Key::Enter if !chord && node.data.is_element_with_tag_name(&local_name!("textarea")) => {
            Some(1)
        }
        _ => None,
    }
}

/// The DOM counts `maxlength` in UTF-16 units; the editor keeps UTF-8. A
/// selection is replaced by the press, so only a collapsed one is refused.
fn over_limit(node: &Node, typed: usize) -> bool {
    let Some(element) = node.element_data() else {
        return false;
    };
    let Some(limit) = element
        .attrs()
        .iter()
        .find(|attr| &*attr.name.local == "maxlength")
        .and_then(|attr| attr.value.trim().parse::<usize>().ok())
    else {
        return false;
    };
    let Some(input) = element.text_input_data() else {
        return false;
    };
    if !input.editor.raw_selection().text_range().is_empty() {
        return false;
    }
    let length = input.editor.raw_text().encode_utf16().count();
    length + typed.max(1) > limit
}
