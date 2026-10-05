use dioxus::prelude::*;

use super::{ColorCode, color_slider::CHECKERBOARD};
use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_props, disabled_look_sx, names_itself,
            shadow_sx, use_name_warning, variables,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{COLOR_SWATCH_RADIUS, COLOR_SWATCH_SIZE, ColorSwatchDefaults, CssVar, Size},
    tokens::HexColor,
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
        // A clickable swatch is a `<button>`, which inherits neither (todo 2376).
        .font_family("inherit")
        .letter_spacing("inherit")
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
        .selector("&:disabled", disabled_look_sx("not-allowed"))
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
    use_name_warning(
        !clickable || names_itself(&props.attributes),
        "ColorSwatch: an `onclick` without an `aria-label`, so it is announced as just \"button\".",
    );

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
    // Judge what shows: the colour over the light checkerboard. Above 0.179 black out-contrasts white.
    let over_white = |c: u8| (c as f64 * a + 255.0 * (1.0 - a)).round() as u32;
    let rgb = (over_white(r) << 16) | (over_white(g) << 8) | over_white(b);
    HexColor::new(rgb).relative_luminance() > 0.179
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 1544: white on these mid-light greens and cyans was under 3:1.
    #[test]
    fn the_mark_takes_the_colour_with_the_better_contrast() {
        let black = HexColor::new(0x00_00_00);
        let white = HexColor::new(0xFF_FF_FF);
        for rgb in [
            0x40_c0_57, 0x12_b8_86, 0x15_aa_bf, 0x22_8b_e6, 0xfa_52_52, 0xfd_7e_14, 0x79_50_f2,
            0xff_ff_ff, 0x00_00_00, 0x86_8e_96,
        ] {
            let fill = HexColor::new(rgb);
            let color = ColorCode::rgba((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 1.0);
            let (mark, other) = match on_light(color) {
                true => (black, white),
                false => (white, black),
            };
            assert!(
                fill.contrast_ratio(mark) >= fill.contrast_ratio(other),
                "{rgb:06x}"
            );
            assert!(fill.contrast_ratio(mark) >= 3.0, "{rgb:06x}");
        }
    }

    /// Todo 2292: rgba(0, 128, 0, 0.6) shows about #66b366 over white, where white was 2.56:1.
    #[test]
    fn a_translucent_swatch_is_judged_by_what_shows() {
        assert!(on_light(ColorCode::rgba(0, 128, 0, 0.6)));
        assert!(on_light(ColorCode::rgba(0, 0, 0, 0.3)));
        assert!(!on_light(ColorCode::rgba(0, 0, 0, 0.9)));
        assert!(!on_light(ColorCode::rgba(0, 128, 0, 1.0)));
    }
}
