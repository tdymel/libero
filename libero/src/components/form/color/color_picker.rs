use dioxus::prelude::*;

use super::{AlphaSlider, ColorCode, ColorSwatch, HueSlider, Swatches, saturation::Saturation};
use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, input_from_str},
        form::SliderChangeEvent,
        layout::use_box,
        variables,
    },
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{
        COLOR_PICKER_PREVIEW, COLOR_PICKER_SPACING, COLOR_PICKER_SWATCH, COLOR_PICKER_WIDTH,
        ColorPickerDefaults, CssVar, Size,
    },
    utils::warn,
};

pub use crate::theme::ColorFormat;

input_from_str!(ColorFormat);

/// The swatch row's widest extent, set only when `swatches_per_row` caps it.
const COLOR_PICKER_SWATCHES_WIDTH: CssVar = CssVar::new("--lsx-color-picker-swatches-width");

static COLOR_PICKER_SX: StaticSx = StaticSx::new(|| {
    let spacing = COLOR_PICKER_SPACING.value();
    let preview = COLOR_PICKER_PREVIEW.value();
    let swatch = COLOR_PICKER_SWATCH.value();
    ColorPickerDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .gap(spacing.clone())
        .width(COLOR_PICKER_WIDTH.value())
        .max_width("100%")
        .when("full-width", sx().width("100%"))
        .selector(
            "& > [data-slot='body']",
            sx().display("flex")
                .align_items("center")
                .gap(spacing.clone()),
        )
        .selector(
            "& > [data-slot='body'] > [data-slot='sliders']",
            sx().display("flex")
                .flex_direction("column")
                .gap(spacing.clone())
                .flex("1 1 auto")
                .min_width("0"),
        )
        // The preview keeps the picker's own scale, not the swatch's.
        .selector(
            "& [data-slot='preview'] > *",
            sx().width(preview.clone())
                .height(preview.clone())
                .min_width(preview),
        )
        .selector(
            "& > [data-slot='swatches']",
            // Wraps, so a `full_width` picker fills its width with swatches.
            // At a step's own width, seven fit per row.
            sx().display("flex")
                .flex_wrap("wrap")
                .gap(spacing)
                .max_width(COLOR_PICKER_SWATCHES_WIDTH.value_or("none")),
        )
        // Fixed per size step, never stretched to the picker's width: the
        // theme picks a swatch that seven of fit the step's own width.
        .selector(
            "& > [data-slot='swatches'] > *",
            sx().width(swatch.clone())
                .height(swatch.clone())
                .min_width(swatch),
        )
});

base_props! {
    pub struct ColorPickerProps {
        /// Strictly controlled - pair it with `oninput`.
        value: ColorCode,
        /// `Start`/`End` bracket a drag on the panel or a slider; a key press
        /// or a swatch click emits a lone `Change`.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<ColorCode>>>,
        /// Shows the alpha slider and the preview swatch beside it.
        #[props(default)]
        with_alpha: Option<bool>,
        /// Preset colors under the panel. Takes `ColorCode`s or CSS strings.
        #[props(default, into)]
        swatches: Swatches,
        /// Caps how many swatches share a row. Unset, they wrap to fill the
        /// width - seven per row at a size step's own width.
        #[props(default)]
        swatches_per_row: Option<usize>,
        /// `false` leaves only the swatches - a palette. Without `swatches`
        /// that draws nothing, which warns in a debug build.
        #[props(default)]
        with_picker: Option<bool>,
        /// Corner radius of the swatches and the preview. Round by default;
        /// `xs` makes them square.
        #[props(default, into)]
        radius: Input<Size>,
        /// A swatch was clicked. `oninput` fires with the same color first.
        #[props(default)]
        onswatchclick: Option<EventHandler<ColorCode>>,
        /// Takes the container's width instead of the size step's.
        #[props(default)]
        full_width: Option<bool>,
        #[props(default, into)]
        size: Input<Size>,
        /// Emits a hidden input of that name, so the color posts with a form.
        #[props(default, into)]
        name: Option<String>,
        /// How the hidden input writes the color. Hex by default, hexa
        /// `with_alpha`.
        #[props(default, into)]
        format: Input<ColorFormat>,
        /// Names the saturation panel's thumb.
        #[props(default, into)]
        saturation_label: Option<String>,
        /// Names the hue slider's thumb.
        #[props(default, into)]
        hue_label: Option<String>,
        /// Names the alpha slider's thumb.
        #[props(default, into)]
        alpha_label: Option<String>,
        /// `false` keeps the thumbs and swatches out of the tab order and
        /// stops a drag or a click from focusing them - for a picker inside a
        /// dropdown whose text input must keep focus, the way `ColorField`
        /// uses it. On by default.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A saturation panel, a hue slider, an optional alpha slider with a preview,
/// and optional preset swatches - Mantine's `ColorPicker`.
///
/// Controlled: it renders `value` and asks for a new one through `oninput`.
/// The value is a [`ColorCode`], which converts to any CSS form afterwards.
#[component]
pub fn ColorPicker(props: ColorPickerProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.color_picker.size);
    let with_alpha = props.with_alpha.unwrap_or(false);
    let with_picker = props.with_picker.unwrap_or(true);
    let radius = props.radius.copied_or(theme.color_picker.radius);
    if !with_picker && props.swatches.is_empty() {
        warn("ColorPicker: `with_picker: false` without `swatches` renders nothing.");
    }
    // Without the alpha slider there is no way to see or change alpha, so the
    // picker holds a solid color: whatever alpha `value` carried is dropped,
    // and everything it emits - panel, hue, swatch - is opaque.
    let value = match with_alpha {
        true => props.value,
        false => props.value.opaque(),
    };
    let oninput = props.oninput;
    let focusable = props.focusable.unwrap_or(true);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("full-width", props.full_width.unwrap_or(false))
        .into();
    let root_variables: Input<Variables> = variables()
        .with(
            COLOR_PICKER_SWATCHES_WIDTH,
            props
                .swatches_per_row
                .or(theme.color_picker.swatches_per_row)
                .map(|count| {
                    let count = count.max(1);
                    format!(
                        "calc({count} * {} + {} * {})",
                        COLOR_PICKER_SWATCH.value(),
                        count - 1,
                        COLOR_PICKER_SPACING.value()
                    )
                }),
        )
        .into();

    let root = use_box()
        .framework_sx(&COLOR_PICKER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&root_variables)
        .prepare();

    let picker = with_picker.then(|| {
        let emit = move |event: SliderChangeEvent<f64>, apply: fn(ColorCode, f64) -> ColorCode| {
            if let Some(oninput) = &oninput {
                oninput.call(match event {
                    SliderChangeEvent::Start(part) => SliderChangeEvent::Start(apply(value, part)),
                    SliderChangeEvent::Change(part) => {
                        SliderChangeEvent::Change(apply(value, part))
                    }
                    SliderChangeEvent::End(part) => SliderChangeEvent::End(apply(value, part)),
                });
            }
        };
        let alpha = with_alpha.then(|| {
            rsx! {
                AlphaSlider {
                    value: value.alpha(),
                    color: value,
                    size,
                    aria_label: props.alpha_label.clone(),
                    focusable,
                    oninput: move |event| emit(event, ColorCode::with_alpha),
                }
            }
        });
        let preview = with_alpha.then(|| {
            rsx! {
                div { "data-slot": "preview",
                    ColorSwatch { color: value, size, radius }
                }
            }
        });

        rsx! {
            Saturation {
                value,
                oninput,
                aria_label: props.saturation_label.clone(),
                focusable,
            }
            div { "data-slot": "body",
                div { "data-slot": "sliders",
                    HueSlider {
                        value: value.hue(),
                        size,
                        aria_label: props.hue_label.clone(),
                        focusable,
                        oninput: move |event| emit(event, ColorCode::with_hue),
                    }
                    {alpha}
                }
                {preview}
            }
        }
    });

    let onswatchclick = props.onswatchclick;
    let swatches = (!props.swatches.is_empty()).then(|| {
        let buttons = props.swatches.iter().copied().map(|color| {
            let color = match with_alpha {
                true => color,
                false => color.opaque(),
            };
            rsx! {
                ColorSwatch {
                    color,
                    size,
                    radius,
                    aria_label: color.to_string(),
                    tabindex: (!focusable).then_some("-1"),
                    onclick: move |_| {
                        if let Some(oninput) = &oninput {
                            oninput.call(SliderChangeEvent::Change(color));
                        }
                        if let Some(onswatchclick) = &onswatchclick {
                            onswatchclick.call(color);
                        }
                    },
                }
            }
        });
        rsx! {
            div { "data-slot": "swatches", {buttons} }
        }
    });

    let format = props.format.copied_or(match with_alpha {
        true => ColorFormat::Hexa,
        false => ColorFormat::Hex,
    });
    let hidden = props.name.map(|name| {
        rsx! {
            input { r#type: "hidden", name, value: value.to_format(format) }
        }
    });

    root.render(
        HtmlTag::Div,
        props.attributes,
        rsx! {
            {picker}
            {swatches}
            {hidden}
        },
    )
}
