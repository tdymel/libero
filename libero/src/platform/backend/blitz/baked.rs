//! Blitz bakes colours into boxes it builds and keeps them: an inline `<svg>`'s
//! `currentColor` (todo 478), and the style of an anonymous block, the text of
//! a flex or grid container (todo 621). Rewriting an attribute rebuilds them.
//!
//! A theme switch rebuilds every such box, a hover change the ones under what
//! gained or lost `:hover`, both before the restyle. Every other state change
//! (focus, `:checked`, an attribute) is caught after it painted: shortly after
//! each press, key or render (see `redraw`), [`check`] rebuilds each box whose
//! colour moved on. The same pass re-lays text left stale (`stale_text`).

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use blitz_dom::BaseDocument;
use dioxus_native_dom::NodeId;
use style::color::AbsoluteColor;

use super::{anchor, doc, run_or_defer, when_laid_out};

/// One document's baked boxes.
#[derive(Default)]
pub(super) struct Watch {
    /// Where hover was at the last `pointermove`. See [`on_hover_change`].
    hovered: Cell<Option<NodeId>>,
    /// Each svg's colour at its last build, `None` when a rebuild is pending.
    svgs: RefCell<HashMap<NodeId, Option<AbsoluteColor>>>,
    armed: Cell<bool>,
    waits: Cell<u32>,
    /// Text re-laid since the last press, key or render, by content width: each
    /// once per width, so a re-layout that breaks it again cannot loop.
    healed: RefCell<HashMap<NodeId, u32>>,
}

/// How many polls [`check`] waits out a pending restyle, which crates.io
/// Blitz's stale dirty bits can fake (upstream #789).
const MAX_WAITS: u32 = 8;

/// Every baked box, for a colour change across the document.
pub(super) fn rebuild_all(doc: &mut BaseDocument) {
    let Ok(elements) = doc.query_selector_all("*") else {
        return;
    };
    rebuild(doc, elements, true);
}

/// What gains or loses `:hover` rebuilds its baked boxes, before the restyle
/// this move asked for (todo 634). Blitz has moved hover by the time a
/// `pointermove` handler runs.
pub(super) fn on_hover_change() {
    let (Some(anchor), Some(state)) = (anchor(), doc()) else {
        return;
    };
    let Some(doc) = anchor.try_doc() else {
        return;
    };
    let now = doc.get_hover_node_id();
    let before = state.baked.hovered.replace(now);
    if before == now {
        return;
    }
    let chain = |id: Option<NodeId>| {
        let mut chain = id.map(|id| doc.node_chain(id)).unwrap_or_default();
        chain.reverse();
        chain
    };
    let (old, new) = (chain(before), chain(now));
    let shared = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let roots: Vec<NodeId> = old
        .get(shared)
        .into_iter()
        .chain(new.get(shared))
        .copied()
        .collect();
    drop(doc);
    if roots.is_empty() {
        return;
    }
    let rebuild_under = move |doc: &mut BaseDocument| {
        let mut subtree = Vec::new();
        let mut stack = roots;
        while let Some(id) = stack.pop() {
            if let Some(node) = doc.get_node(id) {
                subtree.push(id);
                stack.extend(node.children.iter().copied());
            }
        }
        rebuild(doc, subtree, true);
    };
    run_or_defer(&anchor, rebuild_under);
}

/// A press or key reached the wrapper, or libero moved focus: [`check`] once
/// what it changed is laid out (todo 834).
pub(super) fn check_soon() {
    let Some(state) = doc() else {
        return;
    };
    state.baked.healed.borrow_mut().clear();
    if state.baked.armed.replace(true) {
        return;
    }
    when_laid_out(Box::new(check));
}

/// [`check`] again after a stale-text re-layout, keeping what it healed.
fn recheck() {
    if let Some(state) = doc()
        && !state.baked.armed.replace(true)
    {
        when_laid_out(Box::new(check));
    }
}

/// Rebuilds each box that painted another colour than its element computes:
/// an anonymous block's own style against its parent's, an svg against its
/// colour at its last build. An svg first seen is rebuilt once.
fn check() {
    let (Some(anchor), Some(state)) = (anchor(), doc()) else {
        return;
    };
    let stale = {
        let Some(doc) = anchor.try_doc() else {
            state.baked.armed.set(false);
            return;
        };
        // A restyle still pending: what it paints is known after it.
        let waits = state.baked.waits.get();
        if doc.root_node().has_dirty_descendants() && waits < MAX_WAITS {
            state.baked.waits.set(waits + 1);
            when_laid_out(Box::new(check));
            return;
        }
        state.baked.waits.set(0);
        state.baked.armed.set(false);
        let color = |id: NodeId| {
            doc.get_node(id)
                .and_then(|node| node.primary_styles())
                .map(|styles| styles.clone_color())
        };
        let mut svgs = state.baked.svgs.borrow_mut();
        let mut seen = HashMap::with_capacity(svgs.len());
        let mut stale = Vec::new();
        doc.visit(|id, node| {
            let Some(element) = node.element_data() else {
                return;
            };
            let Some(now) = color(id) else {
                return;
            };
            if &*element.name.local == "svg" {
                // `Some(None)`: rebuilt before the restyle, so with the colour computed now.
                if svgs
                    .get(&id)
                    .is_none_or(|built| built.is_some_and(|c| c != now))
                {
                    stale.push(id);
                }
                seen.insert(id, Some(now));
                return;
            }
            let layout = node.layout_children.borrow();
            let anonymous = layout
                .iter()
                .flatten()
                .filter(|id| !node.children.contains(id));
            if anonymous
                .clone()
                .any(|&block| color(block).is_some_and(|c| c != now))
            {
                stale.push(id);
            }
        });
        *svgs = seen;
        (stale, super::stale_text::stale(&doc))
    };
    let (stale, text) = stale;
    let mut healed = state.baked.healed.borrow_mut();
    let (text, again): (Vec<_>, Vec<_>) = text
        .into_iter()
        .partition(|(id, width)| healed.get(id) != Some(width));
    healed.extend(text.iter().copied());
    drop(healed);
    if cfg!(debug_assertions) && !again.is_empty() {
        warn_once("libero: Blitz broke text again after a re-layout; left as is");
    }
    let text: Vec<NodeId> = text.into_iter().map(|(id, _)| id).collect();
    if !stale.is_empty() || !text.is_empty() {
        run_or_defer(&anchor, move |doc| {
            rebuild(doc, stale, false);
            if !text.is_empty() {
                super::stale_text::relayout(doc, text);
                recheck();
            }
        });
    }
}

fn warn_once(message: &str) {
    thread_local! {
        static WARNED: Cell<bool> = const { Cell::new(false) };
    }
    if !WARNED.replace(true) {
        crate::utils::warn(message);
    }
}

/// Re-sets an attribute on each of `elements` that bakes a colour. Ahead of a
/// restyle (`ahead`), what the svgs bake is known at the next [`check`].
fn rebuild(doc: &mut BaseDocument, elements: impl IntoIterator<Item = NodeId>, ahead: bool) {
    let attrs: Vec<_> = elements
        .into_iter()
        .filter_map(|id| {
            let node = doc.get_node(id)?;
            let element = node.element_data()?;
            let anonymous = node
                .layout_children
                .borrow()
                .as_ref()
                .is_some_and(|boxes| boxes.iter().any(|id| !node.children.contains(id)));
            if !anonymous && &*element.name.local != "svg" {
                return None;
            }
            Some((id, element.attrs().first()?.clone()))
        })
        .collect();
    if ahead && let Some(state) = self::doc() {
        let mut svgs = state.baked.svgs.borrow_mut();
        for (id, _) in &attrs {
            if let Some(built) = svgs.get_mut(id) {
                *built = None;
            }
        }
        drop(svgs);
        check_soon();
    }
    // Quiet: a box Blitz will not repaint must not re-arm its own check.
    super::redraw::quiet(|| {
        let mut mutator = doc.mutate();
        for (id, attr) in attrs {
            mutator.set_attribute(id, attr.name, &attr.value);
        }
    });
}
