use dioxus::prelude::*;

use super::{
    ColorCode,
    color_picker::ColorPickerPart,
    color_slider::{ColorSliderPart, HUE_GRADIENT, color_slider_sx, ltr_scale},
};
use crate::{
    components::{
        common::{Input, Part, base_props},
        form::SliderChangeEvent,
        form::slider::{SliderCore, SliderCoreValue},
    },
    hooks::{use_localization, use_theme},
    localization::fill,
    theme::Size,
};

base_props! {
    parts(ColorSliderPart);
    pub struct HueSliderProps {
        /// Degrees, `0-360`. Controlled: pair it with `oninput`.
        value: f64,
        /// `Start`/`End` bracket a drag, `Change` carries every new hue.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent>>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default)]
        disabled: Option<bool>,
        /// Names the thumb, which is the `role="slider"` element.
        #[props(default, into)]
        aria_label: Option<String>,
        /// `false` keeps the thumb from taking focus.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A track of every hue: a `ColorPicker`'s hue slider, on its own.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{HueSlider, SliderChangeEvent};
/// # fn app() -> Element {
/// let mut hue = use_signal(|| 210.0);
/// rsx! {
///     HueSlider {
///         value: hue(),
///         oninput: move |event: SliderChangeEvent| hue.set(event.value()),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/color-picker>
#[component]
pub fn HueSlider(props: HueSliderProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.color_picker.size);
    let sx = color_slider_sx(&props.sx, &props.parts);

    let oninput = props.oninput;
    let emit = use_callback(move |event: SliderChangeEvent<SliderCoreValue>| {
        if let Some(oninput) = &oninput {
            oninput.call(event.map(|value| value.thumb(0)));
        }
    });
    // A bare 0-360 reads as a number; the unit makes it an angle.
    let template = use_localization().color.hue_value;
    let valuetext = use_callback(move |hue: f64| fill(template, &[("value", &hue.round())]));

    rsx! {
        SliderCore {
            value: SliderCoreValue::Single(props.value),
            min: 0.0,
            max: 360.0,
            step: 1.0,
            attributes: ltr_scale(props.attributes),
            class: props.class,
            sx,
            states: props.states,
            size: Input::Value(size),
            color: Input::None,
            disabled: props.disabled,
            label: Some(valuetext),
            marks: Vec::new(),
            aria_label: props.aria_label,
            aria_label_to: None,
            labelledby: None,
            describedby: None,
            invalid: false,
            name: None,
            // The same `use_callback` every render, so the core's props can
            // compare equal.
            oninput: props.oninput.is_some().then_some(emit),
            track: Some(HUE_GRADIENT.to_string()),
            plain: true,
            focusable: props.focusable.unwrap_or(true),
            thumb_fill: Some(ColorCode::hsva(props.value, 1.0, 1.0, 1.0).to_hex()),
            slot: ColorPickerPart::Hue.slot(),
        }
    }
}
