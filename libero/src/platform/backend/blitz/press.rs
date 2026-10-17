use std::{cell::Cell, rc::Rc};

use blitz_dom::{BaseDocument, local_name};
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::{INTERACTIVE, activate, ancestors, anchor, doc, refused, removed};

thread_local! {
    /// A press whose focus move Blitz has not made yet: the focus owner at the
    /// press, and the focusable it hit. See [`Listener`].
    pub(super) static PRESS: Cell<Option<(Option<NodeId>, NodeId)>> = const { Cell::new(None) };

    /// A press that hit nothing focusable, until its click. See [`refocus_wrapper`].
    pub(super) static BLANK_PRESS: Cell<bool> = const { Cell::new(false) };

    /// What the last press on nothing focusable hit, until a Tab. See [`tab_from_start`].
    pub(super) static START: Cell<Option<NodeId>> = const { Cell::new(None) };

    /// The node the last press hit, until its click has bubbled. See
    /// [`nested_interactive`].
    pub(super) static HIT: Cell<Option<NodeId>> = const { Cell::new(None) };
}

/// The node a click just activated: the press's target, or for a key the
/// focused node. `None` after a press on nothing focusable.
pub(super) fn activated(doc: &BaseDocument) -> Option<NodeId> {
    match PRESS.get() {
        Some((_, target)) => Some(target),
        None if BLANK_PRESS.get() => None,
        None => doc.get_focussed_node_id(),
    }
}

pub(super) fn pressed(event: &Event<PointerData>) {
    // Blitz hit-tests page points: a client point misses once the root scrolls.
    let point = event.page_coordinates();
    let wrapper = self::doc().and_then(|doc| doc.wrapper_id());
    HIT.set(None);
    let press = anchor().and_then(|anchor| {
        let doc = anchor.try_doc()?;
        let hit = doc.hit(point.x as f32, point.y as f32)?.node_id;
        HIT.set(Some(hit));
        // The wrapper is no press target: Blitz sends such a press's focus to `<html>`.
        let target = focusable_ancestor(&doc, hit).filter(|&target| Some(target) != wrapper);
        removed::record(&doc, target);
        Some(target.map(|target| (doc.get_focussed_node_id(), target)))
    });
    let blank = matches!(press, Some(None));
    BLANK_PRESS.set(blank);
    START.set(HIT.get().filter(|_| blank));
    PRESS.set(press.flatten());
}

/// The web starts Tab from a clicked node that takes no focus, Blitz from the
/// wrapper: focus goes to that node first, and Blitz walks on from it.
pub(super) fn tab_from_start(event: &Event<KeyboardData>) {
    if event.key() != Key::Tab {
        return;
    }
    let Some(start) = START.take() else {
        return;
    };
    if !event.default_action_enabled() {
        return;
    }
    let Some(doc) = doc() else {
        return;
    };
    let (Some(anchor), Some(wrapper)) = (doc.anchor(), doc.wrapper_id()) else {
        return;
    };
    let Some(doc) = anchor.try_doc() else {
        return;
    };
    // Focus moved since the press: the web starts from the new owner too.
    let unmoved = doc
        .get_focussed_node_id()
        .is_none_or(|focused| focused == wrapper || focused == doc.root_element().id);
    let present = doc
        .get_node(start)
        .is_some_and(|node| node.flags.is_in_document());
    drop(doc);
    if unmoved && present {
        anchor.doc_mut().set_focus_to(start);
    }
}

pub(super) fn forget_press() {
    PRESS.set(None);
    HIT.set(None);
}

/// [`HIT`], walked up to the nearest `boundary`: nested when an interactive
/// element sits on the way, as the web's `closest` pair answers.
pub(in crate::platform::backend) fn nested_interactive(boundary: &str) -> bool {
    let nested = || -> Option<bool> {
        let hit = HIT.get()?;
        let anchor = anchor()?;
        let doc = anchor.try_doc()?;
        let boundaries = doc.query_selector_all(boundary).ok()?;
        let path: Vec<NodeId> = ancestors(&doc, hit).collect();
        let within = path.iter().position(|id| boundaries.contains(id))?;
        let interactive = doc.query_selector_all_in(path[within], INTERACTIVE).ok()?;
        Some(path[..within].iter().any(|id| interactive.contains(id)))
    };
    nested().unwrap_or(false)
}

/// The web's click focus, `tabindex="-1"` included, which Blitz's Tab-walk
/// `is_focussable` skips. Disabled or `inert` presses count from outside.
pub(super) fn focusable_ancestor(doc: &BaseDocument, node_id: NodeId) -> Option<NodeId> {
    let mut node_id = refused::press_origin(doc, node_id)?;
    loop {
        let node = doc.get_node(node_id)?;
        let negative = node.element_data().is_some_and(|element| {
            element.attr(local_name!("disabled")).is_none()
                && element
                    .attr(local_name!("tabindex"))
                    .is_some_and(|index| index.trim().parse::<i32>().is_ok())
        });
        if node.is_focussable() || negative {
            return Some(node_id);
        }
        node_id = node.parent?;
    }
}

/// The web arm's walk: the outermost tab stop from the hit up to `within`,
/// focused unless focus is inside it already.
pub(in crate::platform::backend) fn focus_pressed(
    event: &Event<PointerData>,
    within: &Rc<MountedData>,
) {
    let Some(handle) = within.downcast::<NodeHandle>() else {
        return;
    };
    let point = event.page_coordinates();
    let Some(doc) = handle.try_doc() else {
        return;
    };
    let Some(hit) = doc.hit(point.x as f32, point.y as f32) else {
        return;
    };
    let tab_stop = |id: NodeId| {
        doc.get_node(id).is_some_and(|node| {
            node.is_focussable()
                && node
                    .element_data()
                    .and_then(|element| element.attr(local_name!("tabindex")))
                    .is_none_or(|index| index.trim().parse::<i32>().is_ok_and(|i| i >= 0))
        })
    };
    let mut target = None;
    let mut inside = false;
    for id in ancestors(&doc, hit.node_id) {
        if tab_stop(id) {
            target = Some(id);
        }
        if id == handle.node_id() {
            inside = true;
            break;
        }
    }
    let (Some(target), true) = (target, inside) else {
        return;
    };
    let focused = doc.get_focussed_node_id();
    if focused.is_some_and(|focused| ancestors(&doc, focused).any(|id| id == target)) {
        return;
    }
    drop(doc);
    activate::focus_pressed(focused, target);
}
