//! What `HueSlider` and `AlphaSlider` share: both are a plain `SliderCore`
//! whose track is a gradient as tall as its thumb.

use dioxus::prelude::Attribute;

use crate::{
    components::{Input, common::attr, form::slider::SLIDER_HIT},
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

/// The picker's scale on the slider's own root, and the slider's track and
/// thumb pointed at it. The caller's `sx` goes after, so it still wins.
///
/// On the root rather than inherited from a `ColorPicker`, so a slider used
/// on its own sizes the same as one inside the picker.
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
        // The picker stacks its sliders one `spacing` apart, and at `xs` and
        // `sm` a 24px hit area would reach into the next slider and take its
        // presses. Capped so it never passes the middle of the gap.
        .var(
            SLIDER_HIT,
            format!(
                "min(24px, {} + {})",
                COLOR_PICKER_THUMB.value(),
                COLOR_PICKER_SPACING.value()
            ),
        )
});
