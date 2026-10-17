use std::rc::Rc;

use blitz_dom::BaseDocument;
use dioxus::prelude::*;
use dioxus_native_dom::NodeId;

use super::{anchor, node_of};

/// Taffy lays a row out right to left under `direction: rtl`, as the web does.
pub(in crate::platform::backend) fn is_rtl(mounted: &Rc<MountedData>) -> bool {
    let Some((handle, node_id)) = node_of(mounted) else {
        return false;
    };
    handle
        .try_doc()
        .is_some_and(|doc| node_is_rtl(&doc, node_id))
}

/// Stylo's computed `direction`, named by its `Debug`: libero has no `stylo`
/// dependency to name the enum by.
pub(in crate::platform::backend) fn node_is_rtl(doc: &BaseDocument, node_id: NodeId) -> bool {
    doc.get_node(node_id)
        .and_then(|node| node.primary_styles())
        .is_some_and(|styles| format!("{:?}", styles.clone_direction()) == "Rtl")
}

/// [`node_is_rtl`] on the focused node, where Blitz sends a key press.
pub(in crate::platform::backend) fn rtl_target() -> bool {
    let Some(anchor) = anchor() else {
        return false;
    };
    anchor.try_doc().is_some_and(|doc| {
        doc.get_focussed_node_id()
            .is_some_and(|id| node_is_rtl(&doc, id))
    })
}
