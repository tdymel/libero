use super::backend;

/// The page's CSS media queries, answered live.
pub trait MediaQueryApi {
    /// Calls `callback` with whether `query` matches, as soon as the platform can
    /// say and on every change, until the returned subscription is dropped.
    fn watch(&self, query: &str, callback: Box<dyn Fn(bool)>) -> Box<dyn MediaQuerySubscription>;
}

/// Dropping it stops the callbacks.
pub trait MediaQuerySubscription {}

/// The web's `matchMedia`, a WebView's over eval, and Blitz's stylo, re-asked on each
/// window resize; `None` on a server.
///
/// ```no_run
/// if let Some(media) = libero::platform::media_query() {
///     let _watch = media.watch("(max-width: 767px)", Box::new(|_narrow| {}));
/// }
/// ```
pub fn media_query() -> Option<&'static dyn MediaQueryApi> {
    backend::media_query()
}

/// Whether a fullstack server rendered this page: the client's first render must then draw
/// what the server drew, so a media answer waits for mount. Only the web hydrates.
pub(crate) fn hydrating() -> bool {
    backend::hydrating()
}
