//! Blitz draws no `placeholder`. A framed field draws its own natively, an
//! `aria-hidden` span right before its control; [`sync`] marks each span whose
//! control holds no text, and the frame's stylesheet shows only marked ones.

use std::cell::Cell;

use blitz_dom::{BaseDocument, Node, QualName, ns};
use dioxus_native_dom::NodeId;

use super::{anchor, run_or_defer, when_laid_out};
use crate::platform::{PLACEHOLDER_ATTR, PLACEHOLDER_SHOWN_ATTR};

thread_local! {
    static ARMED: Cell<bool> = const { Cell::new(false) };
}

/// What typing changed is in the editor already: sync now, or at the next flush.
pub(super) fn sync_now() {
    if let Some(anchor) = anchor() {
        run_or_defer(&anchor, sync);
    }
}

/// A press, a key or a newly drawn span: [`sync`] once what it changed is in
/// the tree (a controlled field's cleared `value`, a reset, a mounted field).
pub(in crate::platform::backend) fn sync_soon() {
    if ARMED.replace(true) {
        return;
    }
    when_laid_out(Box::new(|| {
        ARMED.set(false);
        sync_now();
    }));
}

/// Marks each placeholder span whose control is empty, unmarks the rest.
pub(super) fn sync(doc: &mut BaseDocument) {
    let Ok(spans) = doc.query_selector_all(&format!("[{PLACEHOLDER_ATTR}]")) else {
        return;
    };
    let changes: Vec<(NodeId, bool)> = spans
        .into_iter()
        .filter_map(|span| {
            let shown = control(doc, span).is_some_and(|id| is_empty(doc, id));
            let marked = doc
                .get_node(span)?
                .element_data()?
                .attrs
                .iter()
                .any(|attr| &*attr.name.local == PLACEHOLDER_SHOWN_ATTR);
            (shown != marked).then_some((span, shown))
        })
        .collect();
    if changes.is_empty() {
        return;
    }
    let name = QualName::new(None, ns!(), PLACEHOLDER_SHOWN_ATTR.into());
    let mut mutator = doc.mutate();
    for (id, shown) in changes {
        match shown {
            true => mutator.set_attribute(id, name.clone(), ""),
            false => mutator.clear_attribute(id, name.clone()),
        }
    }
}

/// The text control the span stands before: its next element, or the first
/// `input`/`textarea` inside it.
fn control(doc: &BaseDocument, span: NodeId) -> Option<NodeId> {
    let parent = doc.get_node(doc.get_node(span)?.parent?)?;
    let at = parent.children.iter().position(|&id| id == span)?;
    let next = parent.children[at + 1..]
        .iter()
        .copied()
        .find(|&id| doc.get_node(id).is_some_and(Node::is_element))?;
    let mut stack = vec![next];
    while let Some(id) = stack.pop() {
        let node = doc.get_node(id)?;
        if node
            .element_data()
            .is_some_and(|element| matches!(&*element.name.local, "input" | "textarea"))
        {
            return Some(id);
        }
        stack.extend(node.children.iter().rev().copied());
    }
    None
}

/// Blitz keeps typed text in the editor, off the `value` attribute; a control
/// not laid out yet has no editor, and its `value` is all there is.
fn is_empty(doc: &BaseDocument, id: NodeId) -> bool {
    let Some(element) = doc.get_node(id).and_then(Node::element_data) else {
        return false;
    };
    match element.text_input_data() {
        Some(input) => input.editor.text() == "",
        None => element
            .attr(blitz_dom::local_name!("value"))
            .is_none_or(str::is_empty),
    }
}
