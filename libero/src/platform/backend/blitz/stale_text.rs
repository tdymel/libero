//! A cached taffy layout keeps Blitz's min-content text break, so one-line boxes
//! paint broken text (627, 886-888). [`stale`] finds it, [`relayout`] re-lays it.

use std::collections::HashSet;

use blitz_dom::{BaseDocument, Node};
use dioxus_native_dom::NodeId;
use style::computed_values::white_space_collapse::T as WhiteSpaceCollapse;

/// Elements whose text breaks where two lines would fit side by side, with the
/// content width as bits. Preserved white space is skipped.
pub(super) fn stale(doc: &BaseDocument) -> Vec<(NodeId, u32)> {
    let scale = doc.viewport().scale();
    let mut stale = Vec::new();
    doc.visit(|id, node| {
        let collapses = node.primary_styles().is_some_and(|style| {
            style.get_inherited_text().white_space_collapse == WhiteSpaceCollapse::Collapse
        });
        if collapses && texts(doc, node).any(|text| broken_too_narrow(text, scale)) {
            let width = node.unrounded_layout().content_box_width();
            stale.push((id, width.to_bits()));
        }
    });
    stale
}

/// Rebuilds every text box in the flex or grid container around each element:
/// only the stale one would leave its siblings stale.
pub(super) fn relayout(doc: &mut BaseDocument, elements: Vec<NodeId>) {
    let mut scopes = HashSet::new();
    for id in elements {
        let mut scope = id;
        let mut parent = doc.get_node(id).and_then(|node| node.layout_parent.get());
        while let Some(node) = parent.and_then(|id| doc.get_node(id)) {
            if node
                .primary_styles()
                .is_some_and(|style| style.clone_display().is_item_container())
            {
                scope = node.id;
                break;
            }
            parent = node.layout_parent.get();
        }
        scopes.insert(scope);
    }
    let mut attrs = Vec::new();
    let mut stack: Vec<NodeId> = scopes.into_iter().collect();
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get_node(id) else {
            continue;
        };
        if texts(doc, node).next().is_some()
            && let Some(attr) = node.element_data().and_then(|data| data.attrs().first())
        {
            attrs.push((id, attr.clone()));
        }
        stack.extend(node.children.iter().copied());
    }
    super::redraw::quiet(|| {
        let mut mutator = doc.mutate();
        for (id, attr) in attrs {
            mutator.set_attribute(id, attr.name, &attr.value);
        }
    });
}

/// An element's own inline layout and its anonymous blocks'.
fn texts<'a>(doc: &'a BaseDocument, node: &'a Node) -> impl Iterator<Item = &'a Node> + 'a {
    let anonymous: Vec<NodeId> = if node.element_data().is_some() {
        node.layout_children
            .borrow()
            .iter()
            .flatten()
            .filter(|id| !node.children.contains(id))
            .copied()
            .collect()
    } else {
        Vec::new()
    };
    std::iter::once(node)
        .filter(|node| node.element_data().is_some())
        .chain(anonymous.into_iter().filter_map(|id| doc.get_node(id)))
        .filter(|node| {
            node.data
                .downcast_element()
                .is_some_and(|data| data.inline_layout_data.is_some())
        })
}

fn broken_too_narrow(node: &Node, scale: f32) -> bool {
    let Some(inline) = node
        .data
        .downcast_element()
        .and_then(|data| data.inline_layout_data.as_ref())
    else {
        return false;
    };
    // Parley lays out in device pixels.
    let width = node.unrounded_layout().content_box_width() * scale;
    let lines: Vec<_> = inline.layout.lines().collect();
    lines.windows(2).any(|pair| {
        // An empty line's range is `usize::MAX..`, past the text.
        let explicit = inline
            .text
            .get(pair[0].text_range())
            .is_some_and(|text| text.ends_with('\n'));
        let (first, next) = (pair[0].metrics(), pair[1].metrics());
        // A quarter pixel of slack for rounding.
        !explicit && first.advance + next.advance - next.trailing_whitespace + scale / 4.0 <= width
    })
}
