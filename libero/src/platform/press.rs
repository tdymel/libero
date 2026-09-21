use super::backend;

/// Carries a floating box's markers, space separated, on each element a press
/// inside counts for. Rendered only where [`press`] answers.
pub(crate) const PRESS_MARKER_ATTR: &str = "data-lsx-press";

/// Presses anywhere in the document, for a box that cannot learn from focus
/// where a press went.
pub(crate) trait PressApi {
    /// Calls `callback` with the markers on and around each press's target,
    /// until the returned subscription is dropped.
    fn on_press(&self, callback: Box<dyn Fn(Vec<u64>)>) -> Box<dyn PressSubscription>;
}

/// Dropping it stops the callbacks.
pub(crate) trait PressSubscription {}

/// `None` where focus tells a press outside apart (the web, Blitz). A WebView
/// cannot ask whether focus is inside an element, and a tap leaves it on the trigger.
pub(crate) fn press() -> Option<&'static dyn PressApi> {
    backend::press()
}
