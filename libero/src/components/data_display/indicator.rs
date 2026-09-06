use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_color, base_props, contrast_color, fill_color, variables},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        CssVar, INDICATOR_BORDER_WIDTH, INDICATOR_PROCESSING_DURATION, INDICATOR_RADII,
        INDICATOR_RADIUS, IndicatorDefaults, PAPER_BACKGROUND, Size, SizeCss,
    },
};

const INDICATOR_COLOR_VAR: CssVar = CssVar::new("--lsx-indicator-color");
const INDICATOR_CONTRAST_VAR: CssVar = CssVar::new("--lsx-indicator-contrast");

/// `Burger`'s and `Loader`'s spelling, deliberately identical.
const REDUCED_MOTION: &str = "(prefers-reduced-motion: reduce)";

static INDICATOR_BASE_SX: StaticSx = StaticSx::new(|| {
    let fill = INDICATOR_COLOR_VAR.value();

    let base = IndicatorDefaults::theme_vars()
        // Block-level, not `inline-flex`: inside a `Float` an inline root sits
        // in a line box, and the line box's strut makes the `Float` taller than
        // the dot - a bottom placement would then pin the strut, not the dot.
        .display("flex")
        .align_items("center")
        .justify_content("center")
        // The `::before` ping's containing block. Without it the ping resolves
        // against whatever positioned ancestor is nearest - `Float`'s box, which
        // only coincides with the dot while the dot is its sole unpadded child.
        .position("relative")
        .flex("none")
        .width("fit-content")
        .white_space("nowrap")
        .font_weight("700")
        .user_select("none")
        .background(fill.clone())
        .color(INDICATOR_CONTRAST_VAR.value_or("inherit"));

    // A `box-shadow`, not Mantine's `border`: under the global `border-box` a
    // 2px border eats 4px of a 6px dot, and a border moves the dot inside its
    // `Float`. The shadow is drawn outside the box and changes no geometry.
    let with_border = sx().box_shadow(format!(
        "0 0 0 {} {}",
        INDICATOR_BORDER_WIDTH.value(),
        PAPER_BACKGROUND.value()
    ));

    // Behind the dot (`z-index: -1`), so the dot itself never moves - only
    // the copy under it grows and fades. The reduced-motion arm is nested here,
    // on the same per-instance `::before` rule, because a guard written
    // anywhere else loses to this rule on specificity. With the animation
    // cancelled the copy sits exactly under the dot, so nothing is left to see.
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
            .media(REDUCED_MOTION, sx().animation("none")),
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
        /// A step on the indicator's own radius scale. The theme's default,
        /// `xxl`, is round.
        #[props(default, into)]
        radius: Input<Size>,
        /// A ring in the surface colour, so the dot reads on top of a picture.
        #[props(default)]
        with_border: Option<bool>,
        /// A ping behind the dot. Stops under `prefers-reduced-motion`.
        #[props(default)]
        processing: Option<bool>,
    }
}

/// A dot or a small count pinned to something else - an unread marker on an
/// avatar, a pending count on a button.
///
/// Presentational only. It has no positioning of its own: put it in a `Float`
/// inside a `position: relative` parent, which is what owns the corner, the
/// offset and the layer. To hide it, do not render it.
///
/// Always `aria-hidden`. The visible `99+` is a truncation and names nothing,
/// so the meaning belongs on the element the indicator marks - its
/// `aria-label` says "Messages, 128 unread". A caller who really wants the
/// indicator itself announced can override `aria-hidden` from `attributes`.
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
            contrast.and_then(|c| c.resolve(None)),
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
