use dioxus::prelude::*;

use super::{ColorCode, color_slider::CHECKERBOARD};
use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, base_props, shadow_sx, variables},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{COLOR_SWATCH_RADIUS, COLOR_SWATCH_SIZE, ColorSwatchDefaults, CssVar, Size},
};

const COLOR_SWATCH_COLOR: CssVar = CssVar::new("--lsx-color-swatch-color");

static COLOR_SWATCH_SX: StaticSx = StaticSx::new(|| {
    let color = COLOR_SWATCH_COLOR.value();
    ColorSwatchDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .width(COLOR_SWATCH_SIZE.value())
        .height(COLOR_SWATCH_SIZE.value())
        .min_width(COLOR_SWATCH_SIZE.value())
        .border_radius(COLOR_SWATCH_RADIUS.value())
        // A one-stop gradient layers over the checkerboard, so a translucent
        // color shows it through.
        .background(format!("linear-gradient({color}, {color}), {CHECKERBOARD}"))
        .border_style("none")
        .padding("0")
        .when(
            "shadow",
            shadow_sx(
                "inset 0 0 0 1px rgba(0, 0, 0, 0.1), inset 0 0 4px rgba(0, 0, 0, 0.1)".to_string(),
            ),
        )
        // Fixed, not the scheme's `surface`/`ink`: the swatch's color does not
        // flip with the scheme, so neither may the mark on it.
        .color("#fff")
        .when("on-light", sx().color("#000"))
        .when("clickable", sx().cursor("pointer"))
        // A disabled `Fieldset` disables a clickable swatch's `<button>` (todo 514).
        .selector("&:disabled", sx().opacity("0.5").cursor("not-allowed"))
});

base_props! {
    pub struct ColorSwatchProps {
        color: ColorCode,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        radius: Input<Size>,
        /// A faint inner ring, so a color close to the background has an edge.
        #[props(default)]
        with_shadow: Option<bool>,
        /// Makes the swatch a `<button>`.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Drawn on the color in black or white, whichever reads.
        #[props(default)]
        children: Element,
    }
}

/// A patch of one color, with a checkerboard behind a translucent one.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{ColorCode, ColorSwatch};
/// # fn app() -> Element {
/// rsx! {
///     ColorSwatch { color: ColorCode::rgba(34, 139, 230, 0.5) }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/color-picker>
#[component]
pub fn ColorSwatch(props: ColorSwatchProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.color_swatch.size);
    let radius = props.radius.copied_or(theme.color_swatch.radius);
    let clickable = props.onclick.is_some();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("shadow", props.with_shadow.unwrap_or(true))
        .with("clickable", clickable)
        .with("on-light", on_light(props.color))
        .into();

    let variables: Input<Variables> = variables()
        .with(COLOR_SWATCH_COLOR, props.color.to_rgba())
        .into();

    let swatch = use_box()
        .framework_sx(&COLOR_SWATCH_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare();

    match props.onclick {
        Some(onclick) => swatch
            .attr_default("type", "button")
            .event("onclick", move |event: MouseEvent| onclick.call(event))
            .render(HtmlTag::Button, props.attributes, props.children),
        None => swatch.render(HtmlTag::Div, props.attributes, props.children),
    }
}

/// Whether `color` is light enough that the children need to be black.
fn on_light(color: ColorCode) -> bool {
    let (r, g, b, a) = color.to_rgba_channels();
    // Mostly transparent shows the light checkerboard.
    a < 0.5 || (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000 >= 150
}
