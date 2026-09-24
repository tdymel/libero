//! What `HueSlider` and `AlphaSlider` share: both are a plain `SliderCore`
//! whose track is a gradient as tall as its thumb.

use dioxus::prelude::Attribute;

use crate::{
    components::{
        common::{Input, Parts, attr, parts_enum, parts_under_sx},
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

parts_enum! {
    /// [`HueSlider`](super::HueSlider)'s and [`AlphaSlider`](super::AlphaSlider)'s
    /// inner parts, for their `parts` prop.
    pub enum ColorSliderPart {
        /// The gradient rail.
        Track = "track" => "& > [data-slot='track']",
        /// The handle, filled with the color it points at.
        Thumb = "thumb" => "& > [data-slot='track'] > * > [data-slot='thumb']",
    }
}

/// The picker's scale on the slider's own root, so a lone slider sizes like
/// one inside the picker. `parts` merge under the caller's `sx`, which still wins.
pub(super) fn color_slider_sx(
    caller: &Input<Sx>,
    parts: &Input<Parts<ColorSliderPart>>,
) -> Input<Sx> {
    let caller = parts_under_sx(parts, caller.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::common::{Part, part_table},
        sx::sx,
    };

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<ColorSliderPart>(),
            [
                ("track", "& > [data-slot='track']"),
                ("thumb", "& > [data-slot='track'] > * > [data-slot='thumb']"),
            ]
        );
    }

    #[test]
    fn parts_merge_under_the_callers_sx_and_none_keeps_the_static() {
        let parts: Input<Parts<ColorSliderPart>> = Parts::new()
            .part(ColorSliderPart::Thumb, sx().color("red"))
            .into();
        let merged = color_slider_sx(&sx().color("blue").into(), &parts);

        let expected = COLOR_SLIDER_SX.clone().and(
            sx().selector(ColorSliderPart::Thumb.selector(), sx().color("red"))
                .and(sx().color("blue")),
        );
        assert_eq!(merged.as_ref(), Some(&expected));
        assert!(matches!(
            color_slider_sx(&Input::None, &Input::None),
            Input::Static(_)
        ));
    }
}
