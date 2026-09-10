use dioxus::prelude::*;

use super::{
    ColorCode,
    color_code::round_alpha,
    color_slider::{CHECKERBOARD, color_slider_sx},
};
use crate::{
    components::{
        Input,
        common::base_props,
        form::SliderChangeEvent,
        form::slider::{SliderCore, SliderCoreValue},
    },
    hooks::use_theme,
    theme::{CssVar, Size},
};

base_props! {
    pub struct AlphaSliderProps {
        /// `0.0-1.0`. Strictly controlled - pair it with `oninput`.
        value: f64,
        /// The color the track fades in. Its own alpha is ignored.
        color: ColorCode,
        /// `Start`/`End` bracket a drag, `Change` carries every new alpha.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent>>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default)]
        disabled: Option<bool>,
        /// Names the thumb, which is the `role="slider"` element.
        #[props(default, into)]
        aria_label: Option<String>,
        /// `false` keeps the thumb out of the tab order and a drag from
        /// focusing it. On by default.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A track from transparent to `color` over a checkerboard. The opacity half
/// of a `ColorPicker`, usable on its own.
#[component]
pub fn AlphaSlider(props: AlphaSliderProps) -> Element {
    // Both ends spelled as `rgba()`: a bare `transparent` is transparent
    // *black*, which greys the middle of the gradient in older engines.
    let track = format!(
        "linear-gradient(to right, {}, {}), {CHECKERBOARD}",
        props.color.with_alpha(0.0).to_rgba(),
        props.color.opaque().to_rgba(),
    );
    let thumb_fill = props.color.with_alpha(props.value).to_rgba();
    alpha_slider(props, track, thumb_fill)
}

/// `r, g, b` of the picker's color, set on its root: its alpha slider paints
/// with it, so a drag on the panel leaves the slider's props equal.
pub(super) const COLOR_PICKER_RGB: CssVar = CssVar::new("--lsx-color-picker-rgb");

/// `ColorPicker`'s alpha slider: [`AlphaSlider`] colored by
/// [`COLOR_PICKER_RGB`] instead of a `color` prop.
#[component]
pub(super) fn PickerAlphaSlider(
    value: f64,
    size: Size,
    aria_label: Option<String>,
    focusable: bool,
    oninput: EventHandler<SliderChangeEvent>,
) -> Element {
    let rgb = COLOR_PICKER_RGB.value();
    let track =
        format!("linear-gradient(to right, rgba({rgb}, 0), rgba({rgb}, 1)), {CHECKERBOARD}");
    let thumb_fill = format!("rgba({rgb}, {})", round_alpha(value));
    let props = AlphaSliderProps {
        value,
        color: ColorCode::default(),
        oninput: Some(oninput),
        size: Input::Value(size),
        disabled: None,
        aria_label,
        focusable: Some(focusable),
        attributes: Vec::new(),
        class: Input::default(),
        sx: Input::default(),
        states: Input::default(),
    };
    alpha_slider(props, track, thumb_fill)
}

/// Both sliders' render, past the track's colors. `props.color` is unread.
fn alpha_slider(props: AlphaSliderProps, track: String, thumb_fill: String) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.color_picker.size);
    let sx = color_slider_sx(&props.sx);

    let oninput = props.oninput;
    let emit = use_callback(move |event: SliderChangeEvent<SliderCoreValue>| {
        if let Some(oninput) = &oninput {
            oninput.call(event.map(|value| value.thumb(0)));
        }
    });

    rsx! {
        SliderCore {
            value: SliderCoreValue::Single(props.value),
            min: 0.0,
            max: 1.0,
            step: 0.01,
            attributes: props.attributes,
            class: props.class,
            sx,
            states: props.states,
            size: Input::Value(size),
            color: Input::None,
            disabled: props.disabled,
            label: None,
            marks: Vec::new(),
            aria_label: props.aria_label,
            aria_label_to: None,
            labelledby: None,
            describedby: None,
            invalid: false,
            required: false,
            name: None,
            // The same `use_callback` every render, so the core's props can
            // compare equal.
            oninput: props.oninput.is_some().then_some(emit),
            track: Some(track),
            plain: true,
            focusable: props.focusable.unwrap_or(true),
            thumb_fill: Some(thumb_fill),
        }
    }
}
