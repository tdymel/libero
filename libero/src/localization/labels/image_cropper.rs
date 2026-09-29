/// An `ImageCropper`'s strings, and the crop dialog a `FileField` with `crop` opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageCropperLabels {
    /// Names the crop box.
    pub label: &'static str,
    /// The box's and corners' description: their keys.
    pub keys: &'static str,
    /// `aria-valuetext`: `{width}`, `{height}`, `{x}` and `{y}` in percent of the image.
    pub value: &'static str,
    /// The pan mode's `keys`.
    pub pan_keys: &'static str,
    /// The pan mode's `value`: `value`'s holes and `{zoom}`, in percent of the
    /// starting zoom.
    pub pan_value: &'static str,
    /// Names the pan mode's zoom slider.
    pub zoom: &'static str,
    /// The zoom slider's `aria-valuetext`: `{zoom}` in percent of the starting zoom.
    pub zoom_value: &'static str,
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_right: &'static str,
    pub bottom_left: &'static str,
    /// The crop dialog's title.
    pub title: &'static str,
    /// The crop dialog's confirm button.
    pub apply: &'static str,
    /// The crop dialog's button that drops the picked file.
    pub cancel: &'static str,
    /// The crop dialog's error when the picked image does not load.
    pub load_failed: &'static str,
}

impl ImageCropperLabels {
    pub const ENGLISH: Self = Self {
        label: "Crop area",
        keys: "Arrow keys move the crop area, or resize it from a corner. Shift moves further.",
        value: "{width}% by {height}%, at {x}%, {y}%",
        pan_keys: "Arrow keys move the crop area, plus and minus zoom the image. Shift moves further.",
        pan_value: "{width}% by {height}%, at {x}%, {y}%, zoom {zoom}%",
        zoom: "Zoom",
        zoom_value: "{zoom}%",
        top_left: "Top left corner",
        top_right: "Top right corner",
        bottom_right: "Bottom right corner",
        bottom_left: "Bottom left corner",
        title: "Crop image",
        apply: "Apply",
        cancel: "Cancel",
        load_failed: "This image could not be loaded. Cancel and pick another file.",
    };

    pub const GERMAN: Self = Self {
        label: "Zuschnitt",
        keys: "Pfeiltasten verschieben den Zuschnitt oder ändern an einer Ecke seine Größe. Mit Umschalt weiter.",
        value: "{width} % mal {height} %, bei {x} %, {y} %",
        pan_keys: "Pfeiltasten verschieben den Zuschnitt, Plus und Minus zoomen das Bild. Mit Umschalt weiter.",
        pan_value: "{width} % mal {height} %, bei {x} %, {y} %, Zoom {zoom} %",
        zoom: "Zoom",
        zoom_value: "{zoom} %",
        top_left: "Ecke oben links",
        top_right: "Ecke oben rechts",
        bottom_right: "Ecke unten rechts",
        bottom_left: "Ecke unten links",
        title: "Bild zuschneiden",
        apply: "Übernehmen",
        cancel: "Abbrechen",
        load_failed: "Dieses Bild lässt sich nicht laden. Abbrechen und eine andere Datei wählen.",
    };
}
