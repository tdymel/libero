use super::backend;
use crate::tokens::{ColorScheme, ColorSchemeSetting};

/// The platform's colour scheme, and where an app's override is kept. One
/// trait, two capabilities that fail apart (69's plan rejected a storage API).
pub trait ColorSchemeApi {
    /// What the platform is set to right now.
    fn system(&self) -> ColorScheme;

    /// Calls `callback` when the platform's setting changes, until the returned
    /// subscription is dropped.
    fn on_change(&self, callback: Box<dyn Fn(ColorScheme)>) -> Box<dyn ColorSchemeSubscription>;

    /// The app's stored override, `None` where nothing is stored.
    fn stored(&self) -> Option<ColorSchemeSetting>;

    /// Persists the override. Where nothing can be stored, it lasts the session.
    fn store(&self, setting: ColorSchemeSetting);
}

/// Dropping it stops the callbacks.
pub trait ColorSchemeSubscription {}

/// The colour scheme, `None` on a server or headless build. Blitz hears a live
/// change within half a second; a WebView answers light until its media query
/// replies. The override is kept in local storage, see [`set_storage_dir`](super::set_storage_dir).
///
/// ```no_run
/// if let Some(scheme) = libero::platform::color_scheme() {
///     let _now = scheme.system();
/// }
/// ```
pub fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    backend::color_scheme()
}
