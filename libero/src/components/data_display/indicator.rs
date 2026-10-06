use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_color, base_props, contrast_color, fill_color,
            literal_contrast, variables,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
    theme::{
        CssVar, INDICATOR_BORDER_WIDTH, INDICATOR_PROCESSING_DURATION, INDICATOR_RADII,
        INDICATOR_RADIUS, IndicatorDefaults, PAPER_BACKGROUND, Size, SizeCss,
    },
};

const INDICATOR_COLOR_VAR: CssVar = CssVar::new("--lsx-indicator-color");
const INDICATOR_CONTRAST_VAR: CssVar = CssVar::new("--lsx-indicator-contrast");

static INDICATOR_BASE_SX: StaticSx = StaticSx::new(|| {
    let fill = INDICATOR_COLOR_VAR.value();

    let base = IndicatorDefaults::theme_vars()
        // Not `inline-flex`: a line box's strut makes the `Float` taller than the dot.
        .display("flex")
        .align_items("center")
        .justify_content("center")
        // The `::before` ping's containing block, not `Float`'s box.
        .position("relative")
        .flex("none")
        .width("fit-content")
        .white_space("nowrap")
        .font_weight("700")
        .user_select("none")
        .background(fill.clone())
        .color(INDICATOR_CONTRAST_VAR.value_or("inherit"))
        // Forced colours paint every fill `Canvas`, and a bare dot is nothing else.
        .media(FORCED_COLORS, sx().background("CanvasText").color("Canvas"));

    // A `box-shadow`, not a `border`: under `border-box` a 2px border eats 4px
    // of a 6px dot and moves it inside its `Float`. Forced colours drop shadows.
    let with_border = sx()
        .box_shadow(format!(
            "0 0 0 {} {}",
            INDICATOR_BORDER_WIDTH.value(),
            PAPER_BACKGROUND.value()
        ))
        .media(
            FORCED_COLORS,
            sx().outline(format!("{} solid Canvas", INDICATOR_BORDER_WIDTH.value())),
        );

    // A copy behind the dot grows and fades. The reduced-motion guard sits on
    // this same rule: anywhere else it loses on specificity.
    let processing = sx().selector(
        "&::before",
        sx().content("\"\"")
            .position("absolute")
            .inset("0")
            .z_index("-1")
            .border_radius("inherit")
            .background(fill)
            .animation(format!(
                "lsx-indicator-processing {} linear infinite",
                INDICATOR_PROCESSING_DURATION.value()
            ))
            .media(REDUCED_MOTION, sx().animation("none"))
            // Else forced to `Canvas`, invisible on the page.
            .media(FORCED_COLORS, sx().background("CanvasText")),
    );

    base.when("with-border", with_border)
        .when("processing", processing)
        .when(
            "labelled",
            sx().padding_left(format!("calc({} / 2)", SizeCss::SPACING.value(Size::Xs)))
                .padding_right(format!("calc({} / 2)", SizeCss::SPACING.value(Size::Xs))),
        )
});

/// `128` over a cap of `99` is `99+`. The cap is inclusive: `99` is `99`.
fn label_text(count: u32, max: u32) -> String {
    if count > max {
        format!("{max}+")
    } else {
        count.to_string()
    }
}

base_props! {
    pub struct IndicatorProps {
        /// The count. `None` is the bare dot.
        #[props(default, into)]
        label: Option<u32>,
        /// Above it, the label renders as `{max}+`. Defaults to the theme's cap.
        #[props(default, into)]
        max: Option<u32>,
        /// The dot's diameter, and the height of a labelled one.
        #[props(default, into)]
        size: Input<Size>,
        /// The fill. The label takes its auto-contrast twin.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// A step on the indicator's own radius scale; the default is round.
        #[props(default, into)]
        radius: Input<Size>,
        /// A ring in the surface colour, so the dot reads on a picture.
        #[props(default)]
        with_border: Option<bool>,
        /// A repeating ping: turn it off when the work ends (WCAG 2.2.2).
        #[props(default)]
        processing: Option<bool>,
    }
}

/// A dot or small count marking something else; `aria-hidden`, so name the marked element.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Float, Indicator};
/// # fn app() -> Element {
/// rsx! {
///     button { position: "relative", aria_label: "Messages, 128 unread",
///         "Messages"
///         Float { Indicator { label: 128u32 } }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/indicator>
#[component]
pub fn Indicator(props: IndicatorProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.indicator.size);
    let max = props.max.unwrap_or(theme.indicator.max);

    let requested = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or(ThemeAwareValue::Color(theme.indicator.color));
    let color = base_color(Some(&requested));
    let contrast = contrast_color(&color);
    let variables: Input<Variables> = variables()
        .with(INDICATOR_COLOR_VAR, fill_color(&color))
        .with(
            INDICATOR_CONTRAST_VAR,
            contrast
                .and_then(|c| c.resolve(None))
                .or_else(|| literal_contrast(&color)),
        )
        .with(
            INDICATOR_RADIUS.override_var(),
            props
                .radius
                .as_ref()
                .map(|radius| INDICATOR_RADII.value(*radius)),
        )
        .into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with("labelled", props.label.is_some())
        .with("with-border", props.with_border.unwrap_or(false))
        .with("processing", props.processing.unwrap_or(false))
        .into();

    let text = props.label.map(|count| label_text(count, max));

    use_box()
        .framework_sx(&INDICATOR_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .attr_default("aria-hidden", "true")
        .render(
            HtmlTag::Span,
            props.attributes,
            rsx! {
                if let Some(text) = text {
                    "{text}"
                }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::label_text;

    #[test]
    fn the_cap_is_inclusive() {
        assert_eq!(label_text(0, 99), "0");
        assert_eq!(label_text(99, 99), "99");
        assert_eq!(label_text(100, 99), "99+");
        assert_eq!(label_text(u32::MAX, 9), "9+");
    }
}
