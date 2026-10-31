//! What `HueSlider` and `AlphaSlider` share: both are a plain `SliderCore`
//! whose track is a gradient as tall as its thumb.

use std::sync::LazyLock;

use dioxus::prelude::Attribute;

use super::saturation::THUMB_RING;
use crate::{
    components::common::{
        COARSE_POINTER, Input, Part, Parts, attr, coarse_hit_area_sx, parts_enum, parts_under_sx,
        shadow_sx,
    },
    sx::{FORCED_COLORS, StaticSx, Sx, sx},
    theme::{COLOR_PICKER_THUMB, ColorPickerDefaults, OWN_SHADOW, SLIDER_THUMB, SLIDER_TRACK},
};

/// Red round the wheel and back to red, at even sixths.
pub(super) static HUE_GRADIENT: LazyLock<String> = LazyLock::new(|| {
    let hues = [
        "#ff0000", "#ffff00", "#00ff00", "#00ffff", "#0000ff", "#ff00ff", "#ff0000",
    ];
    let stops: Vec<_> = hues
        .iter()
        .enumerate()
        .map(|(index, hue)| (*hue, index as f64 / 6.0))
        .collect();
    scale_gradient(&stops)
});

/// A left-to-right gradient over the thumb centre's travel, inset half a thumb at each
/// end, so the thumb sits on the colour it points at (todo 5120).
pub(super) fn scale_gradient(stops: &[(&str, f64)]) -> String {
    let thumb = SLIDER_THUMB.value();
    let stops: Vec<_> = stops
        .iter()
        .map(|(color, at)| format!("{color} calc({thumb} / 2 + (100% - {thumb}) * {at:.4})"))
        .collect();
    format!("linear-gradient(to right, {})", stops.join(", "))
}

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
    let thumb = ColorSliderPart::Thumb.selector();
    ColorPickerDefaults::theme_vars()
        // The saturation thumb's rings: the thumb is the track's own colour, so the
        // plain slider's surface ring is under 3:1 on some hues (WCAG 1.4.11, todo 3027).
        .selector(
            thumb,
            sx().border_color("#ffffff").var(OWN_SHADOW, THUMB_RING),
        )
        .selector(
            format!("{thumb}:not(:focus-visible)"),
            shadow_sx(THUMB_RING.to_string()),
        )
        .var(SLIDER_TRACK, COLOR_PICKER_THUMB.value())
        .var(SLIDER_THUMB, COLOR_PICKER_THUMB.value())
        // A 44px tall press under a finger (todo 3901). The track stacks above it, so a press
        // on a thumb still starts a drag rather than a jump.
        .and(coarse_hit_area_sx("::after"))
        .media(
            COARSE_POINTER,
            sx().selector(ColorSliderPart::Track.selector(), sx().z_index("1")),
        )
        // Forced colours drop a gradient, and a scale of colours is the only way to pick one.
        .media(
            FORCED_COLORS,
            sx().selector(
                ColorSliderPart::Track.selector(),
                sx().forced_color_adjust("none"),
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

    /// Todo 3901: a finger's press is 44px tall, and the track stays above it for a drag.
    #[test]
    fn a_coarse_pointer_grows_the_press_under_the_track() {
        let css = crate::css::Stylesheet::from(&COLOR_SLIDER_SX);
        let css = css.as_str();
        let coarse = css
            .find("@media (pointer: coarse)")
            .unwrap_or_else(|| panic!("no coarse query: {css}"));
        assert!(css[coarse..].contains("::after"), "{css}");
        assert!(css[coarse..].contains("z-index:1"), "{css}");
    }

    /// Todo 4205: forced colours must not blank the gradient track.
    #[test]
    fn forced_colours_keep_the_gradient_track() {
        let css = crate::css::Stylesheet::from(&COLOR_SLIDER_SX);
        let css = css.as_str();
        let forced = css
            .find("@media (forced-colors: active)")
            .unwrap_or_else(|| panic!("no forced-colours rule: {css}"));
        assert!(css[forced..].contains("forced-color-adjust:none"), "{css}");
    }

    /// Todo 5120: the ends sit half a thumb in, where the thumb's centre stops.
    #[test]
    fn the_scale_gradient_is_inset_by_half_a_thumb() {
        let thumb = SLIDER_THUMB.value();
        assert_eq!(
            scale_gradient(&[("red", 0.0), ("blue", 1.0)]),
            format!(
                "linear-gradient(to right, red calc({thumb} / 2 + (100% - {thumb}) * 0.0000), \
                 blue calc({thumb} / 2 + (100% - {thumb}) * 1.0000))"
            )
        );
        assert!(HUE_GRADIENT.contains("#00ffff calc"), "{}", *HUE_GRADIENT);
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
