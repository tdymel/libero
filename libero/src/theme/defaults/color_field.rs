use crate::theme::Size;

/// What `ColorField` does not share with every other field. The frame's
/// numbers live on `FieldDefaults` and the dropdown's picker on
/// `ColorPickerDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorFieldDefaults {
    pub size: Size,
    pub radius: Size,
    /// The swatch in the leading slot.
    pub with_preview: bool,
    /// The eyedropper button in the trailing slot, where the platform has one.
    pub with_eye_dropper: bool,
    /// Text that does not parse goes back to the last valid color on blur.
    pub fix_on_blur: bool,
    /// Picking a swatch closes the dropdown.
    pub close_on_swatch_click: bool,
}

impl ColorFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        with_preview: true,
        with_eye_dropper: true,
        fix_on_blur: true,
        close_on_swatch_click: false,
    };
}
