//! What `HueSlider` and `AlphaSlider` share: both are a plain `SliderCore`
//! whose track is a gradient as tall as its thumb.

use crate::{
    components::Input,
    sx::Sx,
    theme::{COLOR_PICKER_THUMB, ColorPickerDefaults, SLIDER_THUMB, SLIDER_TRACK},
};

/// Red round the wheel and back to red, at even sixths.
pub(super) const HUE_GRADIENT: &str = "linear-gradient(to right, #ff0000 0%, #ffff00 16.67%, #00ff00 33.33%, #00ffff 50%, #0000ff 66.67%, #ff00ff 83.33%, #ff0000 100%)";

/// What shows through a translucent color. A background *layer*, so it goes
/// last in a comma-separated `background`.
pub(super) const CHECKERBOARD: &str =
    "repeating-conic-gradient(#e9ecef 0% 25%, #ffffff 0% 50%) 50% / 8px 8px";

/// The picker's scale on the slider's own root, and the slider's track and
/// thumb pointed at it. The caller's `sx` goes after, so it still wins.
///
/// On the root rather than inherited from a `ColorPicker`, so a slider used
/// on its own sizes the same as one inside the picker.
pub(super) fn color_slider_sx(caller: &Input<Sx>) -> Input<Sx> {
    let skin = ColorPickerDefaults::theme_vars()
        .var(SLIDER_TRACK, COLOR_PICKER_THUMB.value())
        .var(SLIDER_THUMB, COLOR_PICKER_THUMB.value());
    match caller.as_ref() {
        Some(sx) => skin.and(sx.clone()),
        None => skin,
    }
    .into()
}
