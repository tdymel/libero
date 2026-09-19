//! Blitz's text editor ignores `maxlength` (todos 942, 950): a press that would
//! type past it is refused before the editor's default action, and a paste,
//! whose length is not known then, is cut once it has landed.

use std::cell::RefCell;

use blitz_dom::{Node, QualName, local_name, ns};
use dioxus::prelude::*;
use dioxus_native_dom::NodeId;

use super::{anchor, run_or_defer};
use crate::platform::max_length::fit_insertion;

/// A paste on its way into a control with a `maxlength`: the control, its
/// text before the paste and the limit. Until the next key press.
struct Paste {
    node: NodeId,
    before: String,
    limit: usize,
}

thread_local! {
    static PASTE: RefCell<Option<Paste>> = const { RefCell::new(None) };
}

/// Refuses a character, a textarea's Enter or a paste that would run the
/// focused text control past its `maxlength`, and notes a paste that may.
pub(super) fn key_down(event: &Event<KeyboardData>) {
    PASTE.take();
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
        let (Some(typed), Some((limit, text, collapsed))) =
            (typed_units(event, node), limited(node))
        else {
            return false;
        };
        let length = text.encode_utf16().count();
        if typed == 0 && length < limit {
            PASTE.set(Some(Paste {
                node: node.id,
                before: text.to_string(),
                limit,
            }));
        }
        collapsed && length + typed.max(1) > limit
    });
    if over {
        event.prevent_default();
    }
}

/// How many UTF-16 units the press would type; `0` for a paste.
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

/// A text control's `maxlength` (in UTF-16 units, as the DOM counts), its
/// editor's text, and whether its selection is collapsed - a press replaces a
/// selection, so only a collapsed one is refused.
fn limited(node: &Node) -> Option<(usize, &str, bool)> {
    let element = node.element_data()?;
    let limit = element
        .attrs()
        .iter()
        .find(|attr| &*attr.name.local == "maxlength")
        .and_then(|attr| attr.value.trim().parse::<usize>().ok())?;
    let editor = &element.text_input_data()?.editor;
    let collapsed = editor.raw_selection().text_range().is_empty();
    Some((limit, editor.raw_text(), collapsed))
}

/// The `input` a noted paste fires, cut to the limit.
pub(in crate::platform::backend) fn fit_pasted(value: String) -> String {
    PASTE.with_borrow(|paste| match paste {
        Some(paste) if value.encode_utf16().count() > paste.limit => {
            fit_insertion(&paste.before, &value, paste.limit)
        }
        _ => value,
    })
}

/// The paste has landed: an editor past its limit takes the cut text, which
/// also covers a control no component emits for.
pub(super) fn input() {
    let (Some(paste), Some(anchor)) = (PASTE.take(), anchor()) else {
        return;
    };
    run_or_defer(&anchor, move |doc| {
        let text = doc
            .get_node(paste.node)
            .and_then(limited)
            .map(|(_, text, _)| text.to_string());
        let Some(text) = text.filter(|text| text.encode_utf16().count() > paste.limit) else {
            return;
        };
        let fitted = fit_insertion(&paste.before, &text, paste.limit);
        let name = QualName::new(None, ns!(), local_name!("value"));
        doc.mutate().set_attribute(paste.node, name, &fitted);
    });
}
