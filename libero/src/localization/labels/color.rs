/// `ColorPicker`, its sliders and `ColorField`. The channel names stand in when
/// the matching `*_label` prop is unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorLabels {
    pub saturation: &'static str,
    pub hue: &'static str,
    pub alpha: &'static str,
    /// The panel's value: `{s}` saturation and `{v}` brightness, in percent.
    pub saturation_value: &'static str,
    /// The hue slider's value: `{value}` in degrees.
    pub hue_value: &'static str,
    /// The alpha slider's value: `{value}` in percent.
    pub alpha_value: &'static str,
    /// Names `ColorField`'s dropdown.
    pub choose: &'static str,
    /// Names `ColorField`'s eye dropper button.
    pub eye_dropper: &'static str,
    /// `ColorField`'s error while its typed text is no color.
    pub invalid: &'static str,
}

impl ColorLabels {
    pub const ENGLISH: Self = Self {
        saturation: "Saturation",
        hue: "Hue",
        alpha: "Alpha",
        saturation_value: "Saturation {s}%, brightness {v}%",
        hue_value: "{value} degrees",
        alpha_value: "{value}%",
        choose: "Choose color",
        eye_dropper: "Pick a color from the screen",
        invalid: "Not a valid color",
    };

    pub const GERMAN: Self = Self {
        saturation: "Sättigung",
        hue: "Farbton",
        alpha: "Deckkraft",
        saturation_value: "Sättigung {s} %, Helligkeit {v} %",
        hue_value: "{value} Grad",
        alpha_value: "{value} %",
        choose: "Farbe wählen",
        eye_dropper: "Farbe vom Bildschirm aufnehmen",
        invalid: "Keine gültige Farbe",
    };
}
