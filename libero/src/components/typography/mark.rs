use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, base_props, variables},
        layout::use_box,
    },
    hooks::{use_gradient_style, use_theme},
    sx::{ColorRole, FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{
        ANCHOR_COLOR, AnchorDefaults, Color, ColorShade, ColorValue, CssVar, FOCUS_RING_HALO,
        GRADIENT_CONTRAST, Gradient, NamedColorCss, SURFACE_LABEL, gradient_surface_sx,
    },
};

// Light enough to stay a tint, so text reads over it. The colour is `Theme::mark`.
const MARK_TINT_SHADE: ColorShade = ColorShade::S1;

const MARK_BACKGROUND_VAR: CssVar = CssVar::new("--lsx-mark-background");
const MARK_COLOR_VAR: CssVar = CssVar::new("--lsx-mark-color");

// A bare colour takes the tint shade; a palette shade paints in the fill role,
// whose `contrast-N` twin reads on it. Literals pass through.
fn mark_background_color(value: Option<&ThemeAwareValue>, default_color: Color) -> ThemeAwareValue {
    match value {
        None => ThemeAwareValue::ColorValue(ColorValue::Shade(default_color, MARK_TINT_SHADE)),
        Some(ThemeAwareValue::Color(color)) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, MARK_TINT_SHADE))
        }
        Some(other) => other.clone(),
    }
    .in_color_role(ColorRole::Fill)
}

// A palette or hex background sets the text to its twin; anything else inherits.
static MARK_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().color(format!("var({}, inherit)", MARK_COLOR_VAR.name()))
        .background(MARK_BACKGROUND_VAR.value())
        // Forced colours drop the tint; the system highlight pair keeps it visible.
        .media(FORCED_COLORS, sx().color("MarkText").background("Mark"))
        // A link in the text's colour needs its underline (1.4.1), as in `Alert` and `Header`.
        .and(AnchorDefaults::underline_at_rest())
        // The highlight as a gradient fill, its label read on every point (2132).
        .when(
            "gradient",
            gradient_surface_sx().var(ANCHOR_COLOR, GRADIENT_CONTRAST.value()),
        )
});

/// Publishes `--lsx-focus-contrast`: `sx` infers it only from a literal, and the tint is a `var()`.
/// The tint is the ring's halo (todo 630).
fn mark_variables(color: Option<&ThemeAwareValue>, default_color: Color) -> Variables {
    let background = mark_background_color(color, default_color);
    // A palette shade's `contrast-N`, or a hex's literal black or white.
    let text = background.focus_contrast();
    let fill = background.resolve(None);

    variables()
        .with(MARK_BACKGROUND_VAR, fill.clone())
        .with(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            text.clone(),
        )
        .with(FOCUS_RING_HALO, text.as_ref().and(fill))
        .with(MARK_COLOR_VAR, text.clone())
        // No one link colour reads on every tint (1.01:1 on `info.6`, todo 762).
        .with(ANCHOR_COLOR, text.clone())
        .with(SURFACE_LABEL, text)
}

base_props! {
    pub struct MarkProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Fills the highlight with a gradient from `color`, its text black or white to
        /// read on every point: `("secondary", 45)` or a [`Gradient`]. A literal stop's
        /// contrast is the caller's to check.
        #[props(default, into)]
        gradient: Option<Gradient>,
        children: Element,
    }
}

/// A `<mark>` with a themed background tint.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Mark;
/// # fn app() -> Element {
/// rsx! {
///     "Highlight "
///     Mark { color: "info", "this part" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/mark>
#[component]
pub fn Mark(props: MarkProps) -> Element {
    let theme = use_theme();
    let active = props.gradient.is_some();
    // Under a gradient the tint's inline vars would beat the fill's.
    let variables: Input<Variables> = match active {
        true => Input::None,
        false => mark_variables(props.color.as_ref(), theme.mark.color).into(),
    };
    let style = use_gradient_style(props.gradient.as_ref(), props.color.as_ref(), active, false);
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("gradient", active)
        .into();

    use_box()
        .framework_sx(&MARK_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .style(style)
        .prepare()
        .render(HtmlTag::Mark, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::ColorValue;

    /// The fill tint, also the ring's halo; its twin as the ring and as the text.
    fn palette_variables(color: Color, shade: ColorShade) -> String {
        let twin = ColorValue::Contrast(color, shade).value();
        let fill = ColorValue::Fill(color, shade).value();
        format!(
            "{}:{fill};{}:{twin};{}:{fill};{}:{twin};{}:{twin};{}:{twin};",
            MARK_BACKGROUND_VAR.name(),
            NamedColorCss::FOCUS_CONTRAST.name(),
            FOCUS_RING_HALO.name(),
            MARK_COLOR_VAR.name(),
            ANCHOR_COLOR.name(),
            SURFACE_LABEL.name(),
        )
    }

    #[test]
    fn no_color_falls_back_to_the_themed_default_tinted() {
        let variables = mark_variables(None, Color::Warning);

        assert_eq!(
            variables.to_string(),
            palette_variables(Color::Warning, MARK_TINT_SHADE)
        );
    }

    #[test]
    fn a_bare_color_is_tinted_the_same_way() {
        let color = ThemeAwareValue::Color(Color::Error);
        let variables = mark_variables(Some(&color), Color::Warning);

        assert_eq!(
            variables.to_string(),
            palette_variables(Color::Error, MARK_TINT_SHADE)
        );
    }

    /// Todo 605: an explicit shade paints `fill-N`, the colour `contrast-N` is computed on.
    #[test]
    fn an_explicit_shade_paints_its_fill_and_takes_its_twin() {
        for (color, shade) in [
            (Color::Info, ColorShade::S6),
            (Color::Error, ColorShade::S8),
        ] {
            let value = ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade));

            assert_eq!(
                mark_variables(Some(&value), Color::Warning).to_string(),
                palette_variables(color, shade)
            );
        }
    }

    /// A dark hex left the text to the page's: black on navy, 2.02:1.
    #[test]
    fn a_hex_sets_the_text_to_its_twin() {
        let hex = ThemeAwareValue::from("#1e3a8a");
        let variables = mark_variables(Some(&hex), Color::Warning).to_string();

        assert_eq!(
            variables,
            format!(
                "{}:#1e3a8a;{}:#FFFFFF;{}:#1e3a8a;{}:#FFFFFF;{}:#FFFFFF;{}:#FFFFFF;",
                MARK_BACKGROUND_VAR.name(),
                NamedColorCss::FOCUS_CONTRAST.name(),
                FOCUS_RING_HALO.name(),
                MARK_COLOR_VAR.name(),
                ANCHOR_COLOR.name(),
                SURFACE_LABEL.name()
            )
        );
    }

    #[test]
    fn an_explicit_value_is_not_tinted() {
        let raw = ThemeAwareValue::String("gold".to_string());
        let variables = mark_variables(Some(&raw), Color::Warning);

        assert_eq!(
            variables.to_string(),
            format!("{}:gold;", MARK_BACKGROUND_VAR.name())
        );
    }
}
