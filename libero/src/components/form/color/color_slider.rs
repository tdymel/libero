//! What `HueSlider` and `AlphaSlider` share: both are a plain `SliderCore`
//! whose track is a gradient as tall as its thumb.

use dioxus::prelude::Attribute;

use crate::{
    components::{
        common::{Input, attr},
        form::slider::SLIDER_HIT,
    },
    sx::{StaticSx, Sx},
    theme::{
        COLOR_PICKER_SPACING, COLOR_PICKER_THUMB, ColorPickerDefaults, SLIDER_THUMB, SLIDER_TRACK,
    },
};

/// Red round the wheel and back to red, at even sixths.
pub(super) const HUE_GRADIENT: &str = "linear-gradient(to right, #ff0000 0%, #ffff00 16.67%, #00ff00 33.33%, #00ffff 50%, #0000ff 66.67%, #ff00ff 83.33%, #ff0000 100%)";

/// What shows through a translucent color. A background *layer*, so it goes
/// last in a comma-separated `background`.
pub(super) const CHECKERBOARD: &str =
    "repeating-conic-gradient(#e9ecef 0% 25%, #ffffff 0% 50%) 50% / 8px 8px";

/// A colour scale runs left to right on an RTL page too, as its gradient
/// does; the caller's own `dir` still wins, coming later.
pub(super) fn ltr_scale(mut attributes: Vec<Attribute>) -> Vec<Attribute> {
    attributes.insert(0, attr("dir", "ltr"));
    attributes
}

/// The picker's scale on the slider's own root, so a lone slider sizes like
/// one inside the picker. The caller's `sx` still wins.
pub(super) fn color_slider_sx(caller: &Input<Sx>) -> Input<Sx> {
    match caller.as_ref() {
        Some(sx) => COLOR_SLIDER_SX.clone().and(sx.clone()).into(),
        // The same static every render, so the slider's props compare by
        // address instead of walking the style.
        None => Input::Static(&COLOR_SLIDER_SX),
    }
}

static COLOR_SLIDER_SX: StaticSx = StaticSx::new(|| {
    ColorPickerDefaults::theme_vars()
        .var(SLIDER_TRACK, COLOR_PICKER_THUMB.value())
        .var(SLIDER_THUMB, COLOR_PICKER_THUMB.value())
        // At `xs`/`sm` a 24px hit area would take the next slider's presses,
        // so it stops at the middle of the gap.
        .var(
            SLIDER_HIT,
            format!(
                "min(24px, {} + {})",
                COLOR_PICKER_THUMB.value(),
                COLOR_PICKER_SPACING.value()
            ),
        )
});
