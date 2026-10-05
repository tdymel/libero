use super::{Dimensions, ElementApi, Read, backend};

/// The parts of a platform no element owns: focus, the viewport, `:root`.
pub trait DocumentApi {
    /// Whatever has focus. Read in an event handler, the element acted on: how
    /// an overlay learns where to return focus.
    fn active_element(&self) -> Option<Box<dyn ElementApi>>;

    /// The visible viewport, in CSS pixels.
    fn viewport(&self) -> Read<Dimensions>;

    /// Sets an attribute on `:root`, `None` removes it. `false` where the root
    /// is unreachable: the theme switch then rebuilds the sheet instead.
    fn set_root_attribute(&self, name: &str, value: Option<&str>) -> bool;

    /// The theme's colours changed wholesale. A renderer that baked colours in
    /// redraws them; the web paints from live styles and does nothing.
    fn colors_changed(&self) {}
}

/// The document, `None` where Rust holds none: a WebView, a headless build.
///
/// ```no_run
/// if let Some(document) = libero::platform::document() {
///     document.set_root_attribute("data-mode", Some("compact"));
/// }
/// ```
pub fn document() -> Option<&'static dyn DocumentApi> {
    backend::document()
}

/// The visual viewport's edges in layout coordinates, where the browser panned it
/// (a focused field over a soft keyboard, a pinch zoom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VisibleBand {
    pub(crate) left: f64,
    pub(crate) top: f64,
    /// `f64::INFINITY` where no visual viewport narrows the layout one.
    pub(crate) right: f64,
}

impl VisibleBand {
    pub(crate) const WHOLE: Self = Self {
        left: 0.0,
        top: 0.0,
        right: f64::INFINITY,
    };
}

/// See [`VisibleBand`]: the whole viewport where nothing is panned.
pub(crate) fn visible_band() -> Read<VisibleBand> {
    backend::visible_band()
}

/// `:root`'s computed `padding-right` in px, where the renderer can tell (the web).
pub(crate) fn root_padding_right() -> Option<f64> {
    backend::root_padding_right()
}
