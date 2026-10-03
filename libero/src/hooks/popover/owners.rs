//! Open popups and their anchors: the portal-owner chain that lets a popup
//! opened inside an element count as inside it (`Hotkey::within`).

use std::rc::Rc;

use dioxus::core::{AttributeValue, Runtime, provide_root_context};
use dioxus::prelude::*;

use crate::{
    hooks::ElementHandle,
    platform::{OWNER_ATTR, element_contains, focus_is_in},
    utils::unique_id,
};

/// How many popups deep a chain is followed: a submenu of a submenu.
const MAX_DEPTH: usize = 8;

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

/// Every open popup in this document. Root context, not a `thread_local!`:
/// `VirtualDom`s share a thread. Copied into handles a key callback reads without a runtime.
#[derive(Clone, Copy)]
pub(crate) struct OpenPopups(CopyValue<Vec<Popup>>);

pub(crate) fn use_open_popups() -> OpenPopups {
    use_hook(open_popups)
}

pub(crate) fn open_popups() -> OpenPopups {
    try_consume_context::<OpenPopups>().unwrap_or_else(|| {
        provide_root_context(OpenPopups(CopyValue::new_in_scope(
            Vec::new(),
            ScopeId::ROOT,
        )))
    })
}

/// Lists the popup `floating`, anchored on `anchor`, while `open`.
pub(crate) fn use_popup_owner(anchor: ElementHandle, floating: ElementHandle, open: bool) {
    let id = use_hook(unique_id);
    let scope = Runtime::try_current().and_then(|runtime| runtime.try_current_scope_id());
    let OpenPopups(mut popups) = use_open_popups();
    use_effect(use_reactive!(|open| {
        let mut popups = popups.write();
        popups.retain(|popup| popup.id != id);
        if open {
            popups.push(Popup {
                id,
                scope,
                anchor,
                floating,
            });
        }
    }));
    use_drop(move || {
        // The root's registry is gone already when the whole dom drops.
        if let Ok(mut popups) = popups.try_write() {
            popups.retain(|popup| popup.id != id);
        }
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
pub(crate) fn focus_in_popup_of(open: OpenPopups, scope: &Rc<MountedData>) -> bool {
    let popups: Vec<(Rc<MountedData>, Rc<MountedData>)> = match open.0.try_read() {
        Ok(popups) => popups.iter().filter_map(Popup::mounted).collect(),
        Err(_) => return false,
    };
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
    use std::cell::Cell;

    use dioxus::prelude::*;

    use super::{owned_by, use_open_popups, use_popup_owner};
    use crate::hooks::use_element;

    thread_local! {
        static SEEN: Cell<usize> = const { Cell::new(usize::MAX) };
    }

    fn rendered(app: fn() -> Element) -> VirtualDom {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.process_events();
        dom.render_immediate(&mut dioxus::dioxus_core::NoOpMutations);
        dom
    }

    #[test]
    fn a_second_dom_on_the_thread_sees_none_of_the_first_s_popups() {
        fn opener() -> Element {
            use_popup_owner(use_element(), use_element(), true);
            let open = use_open_popups();
            use_effect(move || SEEN.set(open.0.peek().len()));
            rsx! {}
        }
        fn other() -> Element {
            SEEN.set(use_open_popups().0.peek().len());
            rsx! {}
        }

        let _first = rendered(opener);
        assert_eq!(SEEN.get(), 1, "the opener's own popup is listed");
        let _second = rendered(other);
        assert_eq!(SEEN.get(), 0);
    }

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
