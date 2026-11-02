use std::{cell::Cell, rc::Rc};

use blitz_dom::{BaseDocument, local_name};
use dioxus::prelude::*;
use dioxus_native_dom::{NodeHandle, NodeId};

use super::FreeDoc;
use super::{
    INTERACTIVE, activate, ancestors, anchor, doc, focus, refused, removed, resolved_style_value,
};
use crate::platform::scroll::{EDGE_SWIPE_MARK, SWIPE_CLOSE_ATTR, SWIPE_HANDLE_ATTR, SwipePress};

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
        let doc = anchor.free_doc()?;
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
    if event.key() != Key::Named(NamedKey::Tab) {
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
    let Some(doc) = anchor.free_doc() else {
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

/// The script's walk (`EDGE_PAN_JS`): from the hit up while the inherited mark holds, a box
/// with `touch-action: none` keeps the press.
pub(in crate::platform::backend) fn refuses_edge_swipe(event: &Event<PointerData>) -> bool {
    let point = event.page_coordinates();
    let Some(anchor) = anchor() else {
        return false;
    };
    let Some(doc) = anchor.free_doc() else {
        return false;
    };
    let Some(hit) = doc.hit(point.x as f32, point.y as f32) else {
        return false;
    };
    ancestors(&doc, hit.node_id)
        .filter(|&id| doc.get_node(id).is_some_and(|node| node.is_element()))
        .take_while(|&id| resolved_style_value(&doc, id, EDGE_SWIPE_MARK).trim() == "1")
        .any(|id| resolved_style_value(&doc, id, "touch-action") == "none")
}

/// The script's walk (`SHEET_PAN_JS`): from the hit up to the box marked `id`.
pub(in crate::platform::backend) fn swipe_press(event: &Event<PointerData>, id: u64) -> SwipePress {
    let point = event.page_coordinates();
    let read = || -> Option<SwipePress> {
        let anchor = anchor()?;
        let doc = anchor.free_doc()?;
        let hit = doc.hit(point.x as f32, point.y as f32)?;
        let attr = |node_id: NodeId, name: &str| {
            doc.get_node(node_id)?
                .element_data()?
                .attrs
                .iter()
                .find(|attr| *attr.name.local == *name)
                .map(|attr| attr.value.clone())
        };
        let id = id.to_string();
        let path: Vec<NodeId> = ancestors(&doc, hit.node_id)
            .filter(|&node_id| doc.get_node(node_id).is_some_and(|node| node.is_element()))
            .collect();
        let at = path
            .iter()
            .position(|&node_id| attr(node_id, SWIPE_CLOSE_ATTR).as_deref() == Some(&id))?;
        let selector = format!("[{SWIPE_HANDLE_ATTR}]");
        let shown = doc
            .query_selector_all_in(path[at], &selector)
            .ok()?
            .first()
            .is_some_and(|&strip| resolved_style_value(&doc, strip, "display") != "none");
        let inner = &path[..at];
        let grip = inner
            .iter()
            .any(|&node_id| attr(node_id, SWIPE_HANDLE_ATTR).is_some());
        let refused = !grip
            && inner.iter().any(|&node_id| {
                let name = doc
                    .get_node(node_id)
                    .and_then(|node| node.element_data())
                    .map(|element| element.name.local.to_string())
                    .unwrap_or_default();
                matches!(name.as_str(), "input" | "textarea" | "select")
                    || attr(node_id, "contenteditable")
                        .is_some_and(|value| !value.eq_ignore_ascii_case("false"))
                    || {
                        let action = resolved_style_value(&doc, node_id, "touch-action");
                        !matches!(action.as_str(), "auto" | "manipulation")
                            && !action.split(' ').any(|part| part == "pan-y")
                    }
            });
        let scrolled = !grip
            && path[..=at].iter().any(|&node_id| {
                doc.get_node(node_id).is_some_and(|node| {
                    node.scroll_offset().y > 0.5
                        && matches!(
                            resolved_style_value(&doc, node_id, "overflow-y").as_str(),
                            "auto" | "scroll"
                        )
                })
            });
        Some(SwipePress {
            shown,
            refused,
            scrolled,
        })
    };
    read().unwrap_or_default()
}

pub(super) fn forget_press() {
    PRESS.set(None);
    HIT.set(None);
}

/// A prevented click stopped short of [`Listener`]: Blitz's default moves no focus, so the
/// press's move happens here, as the wrapper would have made it.
pub(in crate::platform::backend) fn click_stopped() {
    if let Some((before, target)) = PRESS.get().filter(|_| !focus::press_kept_focus()) {
        activate::focus_pressed(before, target);
    }
    forget_press();
}

/// [`HIT`], walked up to the nearest `boundary`: nested when an interactive
/// element sits on the way, as the web's `closest` pair answers.
pub(in crate::platform::backend) fn nested_interactive(boundary: &str) -> bool {
    let nested = || -> Option<bool> {
        let hit = HIT.get()?;
        let anchor = anchor()?;
        let doc = anchor.free_doc()?;
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
    let Some(doc) = handle.free_doc() else {
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
