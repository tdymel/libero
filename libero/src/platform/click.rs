use std::rc::Rc;

use dioxus::prelude::{Callback, Event, MountedData, MouseData, PointerData};

use super::{ElementApi, PressSubscription};

/// Whether this click hit something interactive (link, button, field) nested in
/// the nearest `boundary`, which then leaves the click to it, as a `<label>` does.
///
/// Web and Blitz answer ([[codebase/dioxus-event-data]]); elsewhere `false`, the
/// whole boundary one target.
pub(crate) fn nested_interactive(event: &Event<MouseData>, boundary: &str) -> bool {
    super::backend::nested_interactive(event, boundary)
}

/// [`nested_interactive`] for a pointer press: whether it landed on something
/// interactive nested in the nearest `boundary`.
pub(crate) fn nested_interactive_pointer(event: &Event<PointerData>, boundary: &str) -> bool {
    super::backend::nested_interactive_pointer(event, boundary)
}

/// Whether [`nested_interactive`] can answer. The Android WebView cannot: it
/// reads no click target (958).
pub(crate) fn reads_click_targets() -> bool {
    cfg!(any(target_arch = "wasm32", feature = "native"))
}

/// Whether an inline box in a padded block takes the pointer. Blitz's hit test
/// reaches one only when positioned with a positive `z-index` (889).
pub(crate) fn hits_inline_boxes() -> bool {
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"))
}

/// Where `set_pointer_capture` failed, hands `capture`'s drag the moves and the
/// release of `event`'s pointer that land outside it, until that release.
/// Only Blitz does, from `LiberoProvider`'s wrapper; elsewhere a no-op.
pub(crate) fn follow_pointer(
    event: &Event<PointerData>,
    capture: &Rc<MountedData>,
    onmove: Callback<Event<PointerData>>,
    onup: Callback<Event<PointerData>>,
) {
    super::backend::follow_pointer(event, capture, onmove, onup)
}

/// Calls `onrelease` once `pointer_id` lifts or cancels anywhere on the page, so a drag
/// ends when iOS WebKit sends a touch's release past its capture (3603). WebView only.
pub(crate) fn on_pointer_release(pointer_id: i32, onrelease: Callback<()>) {
    super::backend::on_pointer_release(pointer_id, onrelease)
}

/// Whether a press on this target lands outside, for [`swallow_outside_clicks`].
pub(crate) type OutsidePress = Box<dyn Fn(&Rc<MountedData>) -> bool>;

/// Until dropped, while `media` matches, swallows the click of each press whose target `outside`
/// accepts; one pressed before the drop still loses its click (4826). Blitz keeps the click of a
/// release over 2px from its press. A WebView cannot call `outside`: it keeps the click of a press
/// in a portaled layer, an open trigger or the element of WebView tag `kept` (4839). On the web the
/// swallowed press moves no focus, and `onswallow` hears its click go (not on a WebView, 4907).
pub(crate) fn swallow_outside_clicks(
    media: &str,
    outside: OutsidePress,
    onswallow: Rc<dyn Fn()>,
    kept: Option<u64>,
) -> Option<Box<dyn PressSubscription>> {
    super::backend::swallow_outside_clicks(media, outside, onswallow, kept)
}

/// The control a press on a frame's padding belongs to: the first tab stop in
/// `boundary`'s child that is no `[data-slot]` but `control`, nor `[data-ring]`.
///
/// `None` when the press hit that control or anything interactive, and off wasm32.
pub(crate) fn padding_press(
    event: &Event<MouseData>,
    boundary: &str,
) -> Option<Box<dyn ElementApi>> {
    super::backend::padding_press(event, boundary)
}
