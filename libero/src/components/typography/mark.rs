use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, Variables, common::base_props, layout::use_box, variables},
    hooks::use_theme,
    sx::{ColorRole, StaticSx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, CssVar, NamedColorCss},
};

// Light enough to stay a tint rather than a fill, so text reads over it.
// Which color gets tinted is `Theme::mark`; only the shade is fixed here.
const MARK_TINT_SHADE: ColorShade = ColorShade::S1;

const MARK_BACKGROUND_VAR: CssVar = CssVar::new("--lsx-mark-background");
const MARK_COLOR_VAR: CssVar = CssVar::new("--lsx-mark-color");

// A bare color name carries no shade, so it takes the tint shade rather than
// the sx pipeline's generic default. A palette shade paints in the fill role,
// whose `contrast-N` twin reads on it; literal values pass through untouched.
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

// A palette or hex background sets the text to its twin; anything else
// inherits, never the UA's black. The background is always set.
static MARK_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().color(format!("var({}, inherit)", MARK_COLOR_VAR.name()))
        .background(MARK_BACKGROUND_VAR.value())
});

/// `--lsx-focus-contrast` is published beside the tint, because `sx`'s
/// `background()` infers it only from a literal colour and ours is a `var()`.
/// Without it a link inside a `Mark` draws its ring in `primary.6` on the
/// tint. A literal the inference cannot read either (`"gold"`) leaves it
/// unset, as `Blockquote` does.
fn mark_variables(color: Option<&ThemeAwareValue>, default_color: Color) -> Variables {
    let background = mark_background_color(color, default_color);
    // A palette shade's `contrast-N`, or a hex's literal black or white.
    let text = background.focus_contrast();

    variables()
        .with(MARK_BACKGROUND_VAR, background.resolve(None))
        .with(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            text.clone(),
        )
        .with(MARK_COLOR_VAR, text)
}

base_props! {
    pub struct MarkProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        children: Element,
    }
}

/// A real `<mark>` with a themed background tint. `color` takes any theme
/// color or literal value; unset uses a light shade of `Theme::mark`.
#[component]
pub fn Mark(props: MarkProps) -> Element {
    let theme = use_theme();
    let variables: Input<Variables> = mark_variables(props.color.as_ref(), theme.mark.color).into();

    use_box()
        .framework_sx(&MARK_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Mark, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::ColorValue;

    /// The fill tint, its twin as the ring and as the text.
    fn palette_variables(color: Color, shade: ColorShade) -> String {
        let twin = ColorValue::Contrast(color, shade).value();
        format!(
            "{}:{};{}:{twin};{}:{twin};",
            MARK_BACKGROUND_VAR.name(),
            ColorValue::Fill(color, shade).value(),
            NamedColorCss::FOCUS_CONTRAST.name(),
            MARK_COLOR_VAR.name(),
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

    /// Todo 605: an explicit shade is untinted but painted as `fill-N`, the
    /// colour `contrast-N` is computed on. On the brand `info.6` it was
    /// white at 2.78:1; with no text set, dark `error.8` was 2.36:1.
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
                "{}:#1e3a8a;{}:#FFFFFF;{}:#FFFFFF;",
                MARK_BACKGROUND_VAR.name(),
                NamedColorCss::FOCUS_CONTRAST.name(),
                MARK_COLOR_VAR.name()
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
