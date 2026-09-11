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

/// `None` where the renderer cannot report a scroll - a webview and a
/// headless build, where an open popover drifts off its anchor. Blitz reports
/// a wheel inside `LiberoProvider` and libero's own scroll commands.
pub fn scroll() -> Option<&'static dyn ScrollApi> {
    backend::scroll()
}
