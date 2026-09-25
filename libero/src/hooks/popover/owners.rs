//! Open popups and their anchors: the portal-owner chain that lets a popup
//! opened inside an element count as inside it (`Hotkey::within`).

use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::core::{AttributeValue, Runtime};
use dioxus::prelude::*;

use crate::{
    hooks::ElementHandle,
    platform::{OWNER_ATTR, element_contains, focus_is_in},
};

/// How many popups deep a chain is followed: a submenu of a submenu.
const MAX_DEPTH: usize = 8;

static NEXT_POPUP: AtomicU64 = AtomicU64::new(0);

struct Popup {
    id: u64,
    /// The hook's scope, below the owners of both handles: they are read in it.
    scope: Option<ScopeId>,
    anchor: ElementHandle,
    floating: ElementHandle,
}

impl Popup {
    fn mounted(&self) -> Option<(Rc<MountedData>, Rc<MountedData>)> {
        let read = || Some((self.anchor.try_mounted()?, self.floating.try_mounted()?));
        match Runtime::try_current().zip(self.scope) {
            Some((runtime, scope)) => runtime.in_scope(scope, read),
            None => read(),
        }
    }
}

thread_local! {
    static OPEN: RefCell<Vec<Popup>> = const { RefCell::new(Vec::new()) };
}

/// Lists the popup `floating`, anchored on `anchor`, while `open`.
pub(crate) fn use_popup_owner(anchor: ElementHandle, floating: ElementHandle, open: bool) {
    let id = use_hook(|| NEXT_POPUP.fetch_add(1, Ordering::Relaxed));
    let scope = Runtime::try_current().and_then(|runtime| runtime.try_current_scope_id());
    use_effect(use_reactive!(|open| {
        OPEN.with_borrow_mut(|popups| {
            popups.retain(|popup| popup.id != id);
            if open {
                popups.push(Popup {
                    id,
                    scope,
                    anchor,
                    floating,
                });
            }
        });
    }));
    use_drop(move || {
        // A thread-local torn down at exit is gone already.
        let _ = OPEN.try_with(|popups| popups.borrow_mut().retain(|popup| popup.id != id));
    });
}

/// Spread on the box, with `anchor.attributes()` on the anchor: the link a
/// WebView's walk follows back to it. Empty elsewhere.
pub(crate) fn owner_link(anchor: &ElementHandle) -> Vec<Attribute> {
    anchor
        .tag()
        .map(|tag| {
            Attribute::new(
                OWNER_ATTR,
                AttributeValue::Text(tag.to_string()),
                None,
                false,
            )
        })
        .into_iter()
        .collect()
}

/// Whether focus is in an open popup whose anchor lies in `scope`, directly or
/// through the popups it was opened from.
pub(crate) fn focus_in_popup_of(scope: &Rc<MountedData>) -> bool {
    let popups: Vec<(Rc<MountedData>, Rc<MountedData>)> =
        OPEN.with_borrow(|popups| popups.iter().filter_map(Popup::mounted).collect());
    if popups.is_empty() {
        return false;
    }
    let focused = popups
        .iter()
        .position(|(_, floating)| focus_is_in(floating));
    focused.is_some_and(|at| {
        owned_by(
            at,
            &popups,
            |anchor| element_contains(scope, anchor),
            element_contains,
        )
    })
}

/// Whether popup `at`'s anchor lies in the scope, or in a popup whose own does.
fn owned_by<E>(
    at: usize,
    popups: &[(E, E)],
    in_scope: impl Fn(&E) -> bool,
    contains: impl Fn(&E, &E) -> bool,
) -> bool {
    let mut at = at;
    for _ in 0..MAX_DEPTH {
        let anchor = &popups[at].0;
        if in_scope(anchor) {
            return true;
        }
        match popups
            .iter()
            .position(|(_, floating)| contains(floating, anchor))
        {
            Some(parent) if parent != at => at = parent,
            _ => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::owned_by;

    /// Elements are paths: `a` contains `b` when `b` starts with `a`.
    fn check(popups: &[(&str, &str)], at: usize, scope: &str) -> bool {
        owned_by(
            at,
            popups,
            |anchor| anchor.starts_with(scope),
            |floating, anchor| anchor.starts_with(floating),
        )
    }

    #[test]
    fn a_popup_anchored_in_the_scope_is_inside() {
        assert!(check(&[("editor/menu", "portal/1")], 0, "editor"));
        assert!(!check(&[("sidebar/menu", "portal/1")], 0, "editor"));
    }

    #[test]
    fn a_submenu_follows_the_chain_to_its_root() {
        let popups = [("editor/menu", "portal/1"), ("portal/1/more", "portal/2")];
        assert!(check(&popups, 1, "editor"));
        assert!(!check(&popups, 1, "sidebar"));
    }

    #[test]
    fn a_popup_inside_its_own_box_ends_the_walk() {
        assert!(!check(&[("portal/1/x", "portal/1")], 0, "editor"));
    }
}
