use std::{borrow::Cow, rc::Rc};

use dioxus::prelude::{Event, FocusData, MountedData, PointerData};

use super::{ElementApi, backend};

/// Gives a pressed drag handle the focus its cancelled pointerdown took: the
/// outermost tab stop from the target up to `within`, unless focus is inside.
pub(crate) fn focus_pressed(event: &Event<PointerData>, within: &Rc<MountedData>) {
    backend::focus_pressed(event, within);
}

/// Whether focus is on `mounted` or inside it; a WebView answers `false`.
pub(crate) fn focus_is_in(mounted: &Rc<MountedData>) -> bool {
    backend::focus_is_in(mounted)
}

/// Whether `inner` is `outer` or inside it; a WebView answers `false`.
pub(crate) fn element_contains(outer: &Rc<MountedData>, inner: &Rc<MountedData>) -> bool {
    backend::element_contains(outer, inner)
}

/// Focus moves a renderer makes without firing any focus event.
pub(crate) trait SilentFocusApi {
    /// Calls `callback` after such a move has landed, until the returned
    /// subscription is dropped.
    fn on_move(&self, callback: OnMove) -> Box<dyn SilentFocusSubscription>;
}

pub(crate) type OnMove = Box<dyn Fn(&dyn FocusMove)>;

/// One silent move, the `focusout`/`focusin` pair it stands in for.
pub(crate) trait FocusMove {
    /// Whether focus was on `element` or inside it before the move.
    fn was_in(&self, element: &Rc<MountedData>) -> bool;
    /// Whether it is now.
    fn is_in(&self, element: &Rc<MountedData>) -> bool;
    /// [`focus_entered_from`], for the element focus moved into.
    fn entered_from(&self, boundary: &str) -> Option<Option<Box<dyn ElementApi>>>;
}

/// Dropping it stops the callbacks.
pub(crate) trait SilentFocusSubscription {}

/// `None` where every focus move fires its events (the web). Blitz fires none
/// for Tab, Shift+Tab and libero's own `focus()`/`blur()`, and reports those.
pub(crate) fn silent_focus() -> Option<&'static dyn SilentFocusApi> {
    backend::silent_focus()
}

/// `css` as the renderer matches it. Blitz's `:focus-visible` and
/// `:focus-within` never match, so it names attributes libero keeps instead.
pub(crate) fn focus_selectors(css: &str) -> Cow<'_, str> {
    backend::focus_selectors(css)
}

/// Whether a field closing on blur should act on `event`. Not Blitz's blur for a
/// press that cancelled its `mousedown`: libero moves focus back there.
pub(crate) fn blur_counts(event: &Event<FocusData>) -> bool {
    let _ = event;
    !backend::press_kept_focus()
}

/// Whether this `focusin`'s element matches `:focus-visible`, the browser's call
/// on keyboard vs pointer. `None` off the web: the caller keeps its heuristic.
pub(crate) fn focus_visible(event: &Event<FocusData>) -> Option<bool> {
    backend::focus_visible(event)
}

/// For a `focusin` into `boundary`: the element focus left, `Some(None)` from
/// `<body>`, `None` for a move within it or off wasm32 (no `relatedTarget`).
///
/// Blitz answers for its silent moves through [`FocusMove::entered_from`].
pub(crate) fn focus_entered_from(
    event: &Event<FocusData>,
    boundary: &str,
) -> Option<Option<Box<dyn ElementApi>>> {
    backend::focus_entered_from(event, boundary)
}
