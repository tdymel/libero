use super::backend;
use crate::tokens::{ColorScheme, ColorSchemeSetting};

/// The platform's own colour scheme, and the one place an app's override is
/// kept.
///
/// Two capabilities rather than one because they fail apart: a renderer can
/// know the system scheme and have nowhere to persist a choice. Both live
/// here because both are the colour scheme's, and neither is worth a platform
/// API of its own (todo 69's plan rejected a general storage capability for
/// exactly this).
pub trait ColorSchemeApi {
    /// What the platform is set to right now.
    fn system(&self) -> ColorScheme;

    /// Calls `callback` when the platform's own setting changes, until the
    /// returned subscription is dropped.
    fn on_change(&self, callback: Box<dyn Fn(ColorScheme)>) -> Box<dyn ColorSchemeSubscription>;

    /// The app's stored override, `None` where nothing was stored or where
    /// this platform stores nothing.
    fn stored(&self) -> Option<ColorSchemeSetting>;

    /// Persists the override. A platform that cannot store it does nothing,
    /// and the setting then lives for the session only.
    fn store(&self, setting: ColorSchemeSetting);
}

/// Dropping it stops the callbacks, the way every other platform
/// subscription works.
pub trait ColorSchemeSubscription {}

/// `None` where the renderer cannot tell what the platform is set to - a
/// desktop webview and a headless build. Blitz answers from its viewport, and
/// hears a live change within half a second. Android's WebView answers light
/// until its media query first replies, then reports a dark answer as a change.
pub fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    backend::color_scheme()
}
