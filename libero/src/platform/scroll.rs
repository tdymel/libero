use super::backend;

/// A live scroll subscription. **Dropping it unsubscribes** - that is the whole
/// contract, which is why the trait has no methods.
///
/// Held by whatever needs the callback to stop: a popover keeps one while it is
/// open and drops it on close, so a closed dropdown costs nothing.
pub trait ScrollSubscription {}

/// Being told that something scrolled.
///
/// The first callback-shaped capability here - everything else is a command or
/// a [`Read`](super::Read), because everything else is a question with an
/// answer. This one is the platform talking back.
///
/// **Anything that scrolls counts, not just the page.** A scroll event does not
/// bubble, so a listener on the window alone misses an element scrolling - and
/// an anchor inside a `ScrollArea` is the common case, not the exotic one. An
/// implementation has to catch both.
pub trait ScrollApi {
    /// Calls `callback` whenever anything scrolls, until the returned
    /// subscription is dropped. No coordinates: the callback's job is to
    /// re-measure, and it has the handles to do that with.
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription>;
}

/// `None` where the renderer cannot report a scroll - everything but the web
/// today, which leaves an open popover drifting off its anchor there, exactly
/// as it did everywhere before this existed.
///
/// A Blitz arm is implementable and deliberately not written yet: Blitz owns
/// the scroll (`scroll_node_by` is how libero scrolls a node at all), so the
/// hook it needs is a document-side notification the shell can drain - the
/// shape `UiEvent::Activate` already took for `click()`. That is fork work,
/// which this change did not open.
pub fn scroll() -> Option<Box<dyn ScrollApi>> {
    backend::scroll()
}
