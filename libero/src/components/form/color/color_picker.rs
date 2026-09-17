use dioxus::prelude::*;

use super::{
    ColorCode, ColorSwatch, HueSlider, Swatches,
    alpha_slider::{COLOR_PICKER_RGB, PickerAlphaSlider},
    color_code::round_alpha,
    color_slider::CHECKERBOARD,
    saturation::{SATURATION_HUE, Saturation},
};
use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{CheckIcon, base_props, input_from_str},
        form::SliderChangeEvent,
        layout::use_box,
        variables,
    },
    hooks::{use_localization, use_theme},
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
/// The value's alpha, for the preview beside the alpha slider.
const COLOR_PICKER_ALPHA: CssVar = CssVar::new("--lsx-color-picker-alpha");

static COLOR_PICKER_SX: StaticSx = StaticSx::new(|| {
    let spacing = COLOR_PICKER_SPACING.value();
    let preview = COLOR_PICKER_PREVIEW.value();
    let swatch = COLOR_PICKER_SWATCH.value();
    let live = format!(
        "rgba({}, {})",
        COLOR_PICKER_RGB.value(),
        COLOR_PICKER_ALPHA.value()
    );
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
        // Painted from the root's vars, so a drag leaves the swatch's props
        // equal and it skips the redraw.
        .selector(
            "& [data-slot='preview'] > *",
            sx().width(preview.clone())
                .height(preview.clone())
                .min_width(preview)
                .background(format!("linear-gradient({live}, {live}), {CHECKERBOARD}")),
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
        .selector(
            "& > [data-slot='swatches'] svg",
            sx().width("60%").height("60%"),
        )
});

base_props! {
    pub struct ColorPickerProps {
        /// Strictly controlled - pair it with `oninput`.
        value: ColorCode,
        /// `Start`/`End` bracket a drag on the panel or a slider; a key press
        /// or a swatch click emits `Change` then `End`, because either
        /// settles on its color at once. So committing on `End` is enough.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<ColorCode>>>,
        /// Shows the alpha slider and the preview swatch beside it.
        #[props(default)]
        with_alpha: Option<bool>,
        /// Preset colors under the panel. Takes `ColorCode`s or CSS strings,
        /// named by their hex; `Swatches::labelled` names them. The one equal
        /// to `value` is pressed and checked.
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
        /// Names the saturation panel's thumb. Unset, the localization's
        /// `color.saturation`.
        #[props(default, into)]
        saturation_label: Option<String>,
        /// Names the hue slider's thumb. Unset, the localization's `color.hue`.
        #[props(default, into)]
        hue_label: Option<String>,
        /// Names the alpha slider's thumb. Unset, the localization's
        /// `color.alpha`.
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
/// and optional preset swatches.
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
    // The value as a signal of one identity, so the panel's props compare
    // equal and only the scopes that read it redraw - the thumb, on a drag.
    let mut color = use_signal(|| value);
    if *color.peek() != value {
        color.set(value);
    }

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
        .with(
            SATURATION_HUE,
            with_picker.then(|| ColorCode::hsva(value.hue(), 1.0, 1.0, 1.0).to_hex()),
        )
        .with(
            COLOR_PICKER_RGB,
            (with_picker && with_alpha).then(|| {
                let (r, g, b, _) = value.to_rgba_channels();
                format!("{r}, {g}, {b}")
            }),
        )
        .with(
            COLOR_PICKER_ALPHA,
            (with_picker && with_alpha).then(|| round_alpha(value.alpha()).to_string()),
        )
        .into();

    let root = use_box()
        .framework_sx(&COLOR_PICKER_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&root_variables)
        .prepare();

    let emit = move |event: SliderChangeEvent<f64>, apply: fn(ColorCode, f64) -> ColorCode| {
        if let Some(oninput) = &oninput {
            oninput.call(match event {
                SliderChangeEvent::Start(part) => SliderChangeEvent::Start(apply(value, part)),
                SliderChangeEvent::Change(part) => SliderChangeEvent::Change(apply(value, part)),
                SliderChangeEvent::End(part) => SliderChangeEvent::End(apply(value, part)),
            });
        }
    };
    // One identity across renders, so a slider whose own value did not move -
    // the hue during a drag on the panel - skips the re-render.
    let hue_input = use_callback(move |event: SliderChangeEvent<f64>| {
        emit(event, ColorCode::with_hue);
    });
    let alpha_input = use_callback(move |event: SliderChangeEvent<f64>| {
        emit(event, ColorCode::with_alpha);
    });

    let labels = use_localization().color;
    let picker = with_picker.then(|| {
        let alpha = with_alpha.then(|| {
            rsx! {
                PickerAlphaSlider {
                    value: value.alpha(),
                    size,
                    aria_label: props.alpha_label.clone().or_else(|| Some(labels.alpha.into())),
                    focusable,
                    oninput: alpha_input,
                }
            }
        });
        let preview = with_alpha.then(|| {
            rsx! {
                div { "data-slot": "preview",
                    ColorSwatch { color: ColorCode::default(), size, radius }
                }
            }
        });

        rsx! {
            Saturation {
                value: color,
                oninput,
                aria_label: props.saturation_label.clone().or_else(|| Some(labels.saturation.into())),
                focusable,
            }
            div { "data-slot": "body",
                div { "data-slot": "sliders",
                    HueSlider {
                        value: value.hue(),
                        size,
                        aria_label: props.hue_label.clone().or_else(|| Some(labels.hue.into())),
                        focusable,
                        oninput: hue_input,
                    }
                    {alpha}
                }
                {preview}
            }
        }
    });

    let onswatchclick = props.onswatchclick;
    let presets = props.swatches;
    // Only a match reaches the row, so a drag between swatches leaves its props
    // equal and it skips the redraw.
    let selected = Some(value.to_rgba()).filter(|picked| {
        presets
            .iter()
            .any(|&swatch| swatch_color(swatch, with_alpha).to_rgba() == *picked)
    });
    let swatches = (!presets.is_empty()).then(|| {
        rsx! {
            SwatchRow {
                swatches: presets,
                selected,
                with_alpha,
                size,
                radius,
                focusable,
                oninput,
                onswatchclick,
            }
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

/// The preset swatches. Its own scope with plain values: a drag on the panel
/// or a slider leaves its props equal, so it skips the re-render.
#[derive(Props, Clone, PartialEq)]
struct SwatchRowProps {
    swatches: Swatches,
    /// The value as `rgba()`, set only when a swatch equals it.
    selected: Option<String>,
    with_alpha: bool,
    size: Size,
    radius: Size,
    focusable: bool,
    oninput: Option<EventHandler<SliderChangeEvent<ColorCode>>>,
    onswatchclick: Option<EventHandler<ColorCode>>,
}

#[component]
fn SwatchRow(props: SwatchRowProps) -> Element {
    let SwatchRowProps {
        swatches,
        selected,
        with_alpha,
        size,
        radius,
        focusable,
        oninput,
        onswatchclick,
    } = props;
    let buttons = swatches.iter().copied().enumerate().map(|(index, color)| {
        let color = swatch_color(color, with_alpha);
        let pressed = selected.as_ref() == Some(&color.to_rgba());
        let name = swatches
            .label(index)
            .map_or_else(|| color.to_string(), str::to_string);
        rsx! {
            ColorSwatch {
                color,
                size,
                radius,
                aria_label: name,
                aria_pressed: pressed.to_string(),
                tabindex: (!focusable).then_some("-1"),
                onclick: move |_| {
                    if let Some(oninput) = &oninput {
                        // A pick is settled the moment it is made, so it
                        // brackets itself: a caller committing on `End` sees
                        // a swatch the same way it sees a finished drag.
                        oninput.call(SliderChangeEvent::Change(color));
                        oninput.call(SliderChangeEvent::End(color));
                    }
                    if let Some(onswatchclick) = &onswatchclick {
                        onswatchclick.call(color);
                    }
                },
                if pressed {
                    CheckIcon {}
                }
            }
        }
    });
    rsx! {
        div { "data-slot": "swatches", {buttons} }
    }
}

/// A swatch as the picker offers it: opaque unless alpha can be picked.
fn swatch_color(color: ColorCode, with_alpha: bool) -> ColorCode {
    match with_alpha {
        true => color,
        false => color.opaque(),
    }
}
