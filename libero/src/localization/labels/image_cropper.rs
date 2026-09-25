/// An `ImageCropper`'s strings, and the crop dialog a `FileField` with `crop` opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageCropperLabels {
    /// Names the crop box.
    pub label: &'static str,
    /// The box's and corners' description: their keys.
    pub keys: &'static str,
    /// `aria-valuetext`: `{width}`, `{height}`, `{x}` and `{y}` in percent of the image.
    pub value: &'static str,
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
}

impl ImageCropperLabels {
    pub const ENGLISH: Self = Self {
        label: "Crop area",
        keys: "Arrow keys move the crop area, or resize it from a corner. Shift moves further.",
        value: "{width}% by {height}%, at {x}%, {y}%",
        top_left: "Top left corner",
        top_right: "Top right corner",
        bottom_right: "Bottom right corner",
        bottom_left: "Bottom left corner",
        title: "Crop image",
        apply: "Apply",
        cancel: "Cancel",
    };

    pub const GERMAN: Self = Self {
        label: "Zuschnitt",
        keys: "Pfeiltasten verschieben den Zuschnitt oder ändern an einer Ecke seine Größe. Mit Umschalt weiter.",
        value: "{width} % mal {height} %, bei {x} %, {y} %",
        top_left: "Ecke oben links",
        top_right: "Ecke oben rechts",
        bottom_right: "Ecke unten rechts",
        bottom_left: "Ecke unten links",
        title: "Bild zuschneiden",
        apply: "Übernehmen",
        cancel: "Abbrechen",
    };
}
