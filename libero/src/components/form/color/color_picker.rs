use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    ColorCode, ColorSwatch, HueSlider, Swatches,
    alpha_slider::{COLOR_PICKER_RGB, PickerAlphaSlider},
    color_code::round_alpha,
    color_slider::CHECKERBOARD,
    saturation::{SATURATION_HUE, Saturation},
};
use crate::{
    components::{
        common::{
            Glyph, HtmlTag, Input, Part, States, Variables, base_props, input_from_str, parts_enum,
            variables,
        },
        form::SliderChangeEvent,
        layout::use_box,
    },
    context::IconSlot,
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
/// The hue slider's last step: 360 is the same hue as 0.
const MAX_HUE: f64 = 359.0;

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
            ColorPickerPart::Body.selector(),
            sx().display("flex")
                .align_items("center")
                .gap(spacing.clone()),
        )
        .selector(
            ColorPickerPart::Sliders.selector(),
            sx().display("flex")
                .flex_direction("column")
                .gap(spacing.clone())
                .flex("1 1 auto")
                .min_width("0"),
        )
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
            ColorPickerPart::Swatches.selector(),
            // Wraps, so a `full_width` picker fills its width with swatches.
            sx().display("flex")
                .flex_wrap("wrap")
                .gap(spacing)
                .max_width(COLOR_PICKER_SWATCHES_WIDTH.value_or("none")),
        )
        // Fixed per size step: seven fit the step's own width.
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

parts_enum! {
    /// [`ColorPicker`]'s inner parts, for its `parts` prop. Each is matched from
    /// the root by child selectors.
    pub enum ColorPickerPart {
        /// The saturation and brightness panel.
        Saturation = "saturation" => "& > [data-slot='saturation']",
        /// The row under the panel: the sliders and the preview.
        Body = "body" => "& > [data-slot='body']",
        /// The hue and alpha sliders' column.
        Sliders = "sliders" => "& > [data-slot='body'] > [data-slot='sliders']",
        Hue = "hue" => "& > [data-slot='body'] > [data-slot='sliders'] > [data-slot='hue']",
        /// With `with_alpha`.
        Alpha = "alpha" => "& > [data-slot='body'] > [data-slot='sliders'] > [data-slot='alpha']",
        /// Both sliders' gradient rails.
        Track = "track" => "& > [data-slot='body'] > [data-slot='sliders'] > * > [data-slot='track']",
        /// Every handle: the panel's and the sliders'.
        Thumb = "thumb" => "& > [data-slot='saturation'] > [data-slot='thumb'], & > [data-slot='body'] > [data-slot='sliders'] > * > [data-slot='track'] > * > [data-slot='thumb']",
        /// The current color beside the sliders, with `with_alpha`.
        Preview = "preview" => "& > [data-slot='body'] > [data-slot='preview']",
        /// The preset swatches' row.
        Swatches = "swatches" => "& > [data-slot='swatches']",
        /// One preset swatch.
        Swatch = "swatch" => "& > [data-slot='swatches'] > [data-slot='swatch']",
    }
}

base_props! {
    parts(ColorPickerPart);
    pub struct ColorPickerProps {
        /// Controlled: pair it with `oninput`.
        value: ColorCode,
        /// A drag brackets its moves with `Start`/`End`; a key press or swatch
        /// emits `Change` then `End`, so committing on `End` is enough.
        #[props(default)]
        oninput: Option<EventHandler<SliderChangeEvent<ColorCode>>>,
        /// Shows the alpha slider and the preview swatch beside it.
        #[props(default)]
        with_alpha: Option<bool>,
        /// Preset colors under the panel, `ColorCode`s or CSS strings.
        #[props(default, into)]
        swatches: Swatches,
        /// Caps how many swatches share a row; unset, they wrap.
        #[props(default)]
        swatches_per_row: Option<usize>,
        /// `false` leaves only the swatches, a palette.
        #[props(default)]
        with_picker: Option<bool>,
        /// Corner radius of the swatches and the preview. Round by default.
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
        /// How the hidden input writes the color.
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
        /// `false` keeps the thumbs and swatches from taking focus, for a
        /// dropdown whose text input must keep it (`ColorField`).
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A saturation panel, a hue slider, an optional alpha slider and preset swatches.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{ColorCode, ColorPicker, SliderChangeEvent};
/// # fn app() -> Element {
/// let mut color = use_signal(ColorCode::default);
/// rsx! {
///     ColorPicker {
///         value: color(),
///         with_alpha: true,
///         oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/color-picker>
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
    // Without the alpha slider nothing can change alpha, so the picker holds
    // and emits an opaque color.
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
        .parts(&props.parts)
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
    // `ColorCode` wraps 360 to 0, which would snap the thumb from the right end to the left.
    let hue_input = use_callback(move |event: SliderChangeEvent<f64>| {
        emit(event, |color, hue| color.with_hue(hue.min(MAX_HUE)));
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
                div { "data-slot": ColorPickerPart::Preview.slot(),
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
            div { "data-slot": ColorPickerPart::Body.slot(),
                div { "data-slot": ColorPickerPart::Sliders.slot(),
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
                "data-slot": ColorPickerPart::Swatch.slot(),
                aria_label: name,
                aria_pressed: pressed.to_string(),
                tabindex: (!focusable).then_some("-1"),
                onclick: move |_| {
                    if let Some(oninput) = &oninput {
                        // Settled at once, so a caller committing on `End`
                        // sees it like a finished drag.
                        oninput.call(SliderChangeEvent::Change(color));
                        oninput.call(SliderChangeEvent::End(color));
                    }
                    if let Some(onswatchclick) = &onswatchclick {
                        onswatchclick.call(color);
                    }
                },
                if pressed {
                    Glyph { slot: IconSlot::Check, icon: lucide::check::outlined }
                }
            }
        }
    });
    rsx! {
        div { "data-slot": ColorPickerPart::Swatches.slot(), {buttons} }
    }
}

/// A swatch as the picker offers it: opaque unless alpha can be picked.
fn swatch_color(color: ColorCode, with_alpha: bool) -> ColorCode {
    match with_alpha {
        true => color,
        false => color.opaque(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let slots: Vec<_> = part_table::<ColorPickerPart>()
            .into_iter()
            .map(|(slot, _)| slot)
            .collect();
        assert_eq!(
            slots,
            [
                "saturation",
                "body",
                "sliders",
                "hue",
                "alpha",
                "track",
                "thumb",
                "preview",
                "swatches",
                "swatch",
            ]
        );
    }

    /// Every alternative of a selector starts at the root and ends on its slot.
    #[test]
    fn every_selector_targets_its_own_slot() {
        for (slot, selector) in part_table::<ColorPickerPart>() {
            let target = format!("[data-slot='{slot}']");
            for alternative in selector.split(", ") {
                assert!(alternative.starts_with("& > "), "{alternative}");
                assert!(alternative.ends_with(&target), "{alternative}");
            }
        }
    }
}
