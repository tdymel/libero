/// A `Lightbox`'s strings. Its close button reads [`CommonLabels::close`](super::CommonLabels::close).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightboxLabels {
    pub label: &'static str,
    /// Names the thumbnail strip, which is a region of its own.
    pub thumbnails: &'static str,
    /// Names a thumbnail: `{n}` is the slide number.
    pub thumbnail: &'static str,
    /// The zoomable picture's description: its keys.
    pub keys: &'static str,
    /// Announced after a zoom: `{n}` is the scale in percent of the fitted size.
    pub zoomed: &'static str,
    /// Announced once a zoom is back to the fitted size.
    pub fitted: &'static str,
    /// Names the toolbar's zoom-in button.
    pub zoom_in: &'static str,
    /// Names the toolbar's zoom-out button.
    pub zoom_out: &'static str,
}

impl LightboxLabels {
    pub const ENGLISH: Self = Self {
        label: "Gallery",
        thumbnails: "Thumbnails",
        thumbnail: "Go to slide {n}",
        keys: "Z, plus or minus to zoom. Arrow keys pan a zoomed picture, or change the picture.",
        zoomed: "Zoomed to {n}%",
        fitted: "Zoom reset",
        zoom_in: "Zoom in",
        zoom_out: "Zoom out",
    };

    pub const GERMAN: Self = Self {
        label: "Galerie",
        thumbnails: "Vorschaubilder",
        thumbnail: "Zu Bild {n}",
        keys: "Z, Plus oder Minus zum Zoomen. Pfeiltasten verschieben ein vergrößertes Bild oder wechseln das Bild.",
        zoomed: "Auf {n} % gezoomt",
        fitted: "Zoom zurückgesetzt",
        zoom_in: "Vergrößern",
        zoom_out: "Verkleinern",
    };
}
