use std::{cell::Cell, rc::Rc};

use blitz_dom::{QualName, local_name, ns};
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::{HIT_AREA_STYLE, ancestors, anchor, defer, doc, node_of, resolved_style_value};

thread_local! {
    /// The drag [`follow_pointer`] stands in pointer capture for.
    pub(super) static FOLLOW: Cell<Option<Follow>> = const { Cell::new(None) };
}

/// A drag's moves and release, for the pointer it pressed with.
#[derive(Clone, Copy)]
struct Follow {
    pointer_id: i32,
    capture: NodeId,
    onmove: Callback<Event<PointerData>>,
    onup: Callback<Event<PointerData>>,
}

pub(in crate::platform::backend) fn follow_pointer(
    event: &Event<PointerData>,
    capture: &Rc<MountedData>,
    onmove: Callback<Event<PointerData>>,
    onup: Callback<Event<PointerData>>,
) {
    // A selector-found capture too, not just a mounted one (todo 2749).
    let Some((anchor, node_id)) = node_of(capture) else {
        return;
    };
    FOLLOW.set(Some(Follow {
        pointer_id: event.pointer_id(),
        capture: node_id,
        onmove,
        onup,
    }));
    let cursor = anchor
        .try_doc()
        .map(|doc| resolved_style_value(&doc, node_id, "cursor"))
        .unwrap_or_default();
    catch_pointer(Some(cursor));
}

/// Turns [`HIT_AREA_STYLE`]'s box into the drag's catcher, with the capture
/// element's cursor, or back into a hitless one.
fn catch_pointer(cursor: Option<String>) {
    let (Some(anchor), Some(area)) = (
        anchor(),
        doc().and_then(|doc| doc.hit_area.borrow().as_ref().map(NodeHandle::node_id)),
    ) else {
        return;
    };
    let style = match cursor {
        Some(cursor) => format!("{HIT_AREA_STYLE}pointer-events:auto;cursor:{cursor};"),
        None => HIT_AREA_STYLE.to_string(),
    };
    defer(&anchor, move |doc| {
        let name = QualName::new(None, ns!(), local_name!("style"));
        doc.mutate().set_attribute(area, name, &style);
    });
}

/// Hands the followed drag what bubbled here from outside its element. A stopped
/// move is never seen; a release off the app ends it at the next move (todo 2244).
pub(super) fn followed(event: &Event<PointerData>, up: bool) {
    let Some(follow) = FOLLOW.get().filter(|f| f.pointer_id == event.pointer_id()) else {
        return;
    };
    let released = !up && event.held_buttons().is_empty();
    if up || released {
        FOLLOW.set(None);
        catch_pointer(None);
    }
    let point = event.page_coordinates();
    let outside = anchor().and_then(|anchor| {
        let doc = anchor.try_doc()?;
        // Gone with its element, and so are the handlers.
        if !doc
            .get_node(follow.capture)
            .is_some_and(|node| node.flags.is_in_document())
        {
            return None;
        }
        let inside = doc
            .hit(point.x as f32, point.y as f32)
            .is_some_and(|hit| ancestors(&doc, hit.node_id).any(|id| id == follow.capture));
        Some(!inside)
    });
    match outside {
        Some(outside) if released || (up && outside) => follow.onup.call(event.clone()),
        Some(true) => follow.onmove.call(event.clone()),
        Some(false) => {}
        None => {
            FOLLOW.set(None);
            catch_pointer(None);
        }
    }
}
