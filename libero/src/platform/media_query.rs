use super::backend;

/// The page's CSS media queries, answered live.
pub trait MediaQueryApi {
    /// Calls `callback` with whether `query` matches, as soon as the platform can
    /// say and on every change, until the returned subscription is dropped.
    fn watch(&self, query: &str, callback: Box<dyn Fn(bool)>) -> Box<dyn MediaQuerySubscription>;
}

/// Dropping it stops the callbacks.
pub trait MediaQuerySubscription {}

/// The web's `matchMedia`, and a WebView's over eval; `None` on Blitz, which
/// has no media query to ask, and on a server.
///
/// ```no_run
/// if let Some(media) = libero::platform::media_query() {
///     let _watch = media.watch("(max-width: 767px)", Box::new(|_narrow| {}));
/// }
/// ```
pub fn media_query() -> Option<&'static dyn MediaQueryApi> {
    backend::media_query()
}
