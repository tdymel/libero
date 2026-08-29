use dioxus::prelude::*;

use super::{
    ColorCode,
    color_slider::{HUE_GRADIENT, color_slider_sx},
};
use crate::{
    components::{
        Input,
        common::base_props,
        form::SliderChangeEvent,
        form::slider::{SliderCore, SliderCoreValue},
    },
    hooks::use_theme,
    theme::Size,
};

base_props! {
    pub struct HueSliderProps {
        /// Degrees, `0-360`. Strictly controlled - pair it with `oninput`.
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
        /// `false` keeps the thumb out of the tab order and a drag from
        /// focusing it. On by default.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A track of every hue, with the thumb showing the one picked. The hue half
/// of a `ColorPicker`, usable on its own.
#[component]
pub fn HueSlider(props: HueSliderProps) -> Element {
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
            max: 360.0,
            step: 1.0,
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
            oninput: props.oninput.is_some().then(|| EventHandler::new(move |event| emit.call(event))),
            track: Some(HUE_GRADIENT.to_string()),
            plain: true,
            focusable: props.focusable.unwrap_or(true),
            thumb_fill: Some(ColorCode::hsva(props.value, 1.0, 1.0, 1.0).to_hex()),
        }
    }
}
