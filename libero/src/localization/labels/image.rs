/// A zoomable `Image`'s button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageLabels {
    /// Names the button of a picture without `alt`.
    pub zoom: &'static str,
    /// Names it after the picture: `{alt}`.
    pub zoom_named: &'static str,
}

impl ImageLabels {
    pub const ENGLISH: Self = Self {
        zoom: "Zoom in",
        zoom_named: "Zoom in: {alt}",
    };

    pub const GERMAN: Self = Self {
        zoom: "Vergrößern",
        zoom_named: "Vergrößern: {alt}",
    };
}
