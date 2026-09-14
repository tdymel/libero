use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, Variables, common::base_props, layout::use_box, variables},
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, CssVar, NamedColorCss},
};

// Light enough to stay a tint rather than a fill, so text reads over it.
// Which color gets tinted is `Theme::mark`; only the shade is fixed here.
const MARK_TINT_SHADE: ColorShade = ColorShade::S1;

const MARK_BACKGROUND_VAR: CssVar = CssVar::new("--lsx-mark-background");
const MARK_COLOR_VAR: CssVar = CssVar::new("--lsx-mark-color");

// A bare color name carries no shade, so it takes the tint shade rather than
// the sx pipeline's generic default. Explicit shades and literal values pass
// through untouched.
fn mark_background_color(value: Option<&ThemeAwareValue>, default_color: Color) -> ThemeAwareValue {
    match value {
        None => ThemeAwareValue::ColorValue(ColorValue::Shade(default_color, MARK_TINT_SHADE)),
        Some(ThemeAwareValue::Color(color)) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, MARK_TINT_SHADE))
        }
        Some(other) => other.clone(),
    }
}

// A hex background sets the text to black or white; anything else inherits,
// never the UA's black. The background is always set.
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
    // Literal black or white: a hex does not flip with the scheme, and `--lsx-ink` does.
    let text = match &background {
        ThemeAwareValue::RawColor(_, hex) => Some(hex.contrast().to_string()),
        _ => None,
    };

    variables()
        .with(MARK_BACKGROUND_VAR, background.resolve(None))
        .with(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            text.clone().or_else(|| background.focus_contrast()),
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

    #[test]
    fn no_color_falls_back_to_the_themed_default_tinted() {
        let variables = mark_variables(None, Color::Warning);

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};{}:{};",
                MARK_BACKGROUND_VAR.name(),
                ColorValue::Shade(Color::Warning, MARK_TINT_SHADE).value(),
                NamedColorCss::FOCUS_CONTRAST.name(),
                ColorValue::Contrast(Color::Warning, MARK_TINT_SHADE).value()
            )
        );
    }

    #[test]
    fn a_bare_color_is_tinted_the_same_way() {
        let color = ThemeAwareValue::Color(Color::Error);
        let variables = mark_variables(Some(&color), Color::Warning);

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};{}:{};",
                MARK_BACKGROUND_VAR.name(),
                ColorValue::Shade(Color::Error, MARK_TINT_SHADE).value(),
                NamedColorCss::FOCUS_CONTRAST.name(),
                ColorValue::Contrast(Color::Error, MARK_TINT_SHADE).value()
            )
        );
    }

    /// Anything that isn't a bare `Color` is the caller being explicit, so
    /// it goes through untinted. No contrast twin can be read off a name like
    /// this one, so the ring is left to the inherited value.
    #[test]
    fn an_explicit_shade_takes_its_own_contrast_twin() {
        let color = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Info, ColorShade::S6));
        let variables = mark_variables(Some(&color), Color::Warning).to_string();

        assert!(variables.contains(&format!(
            "{}:{};",
            NamedColorCss::FOCUS_CONTRAST.name(),
            ColorValue::Contrast(Color::Info, ColorShade::S6).value()
        )));
        assert!(!variables.contains(MARK_COLOR_VAR.name()), "{variables}");
    }

    /// A dark hex left the text to the page's: black on navy, 2.02:1.
    #[test]
    fn a_hex_sets_the_text_to_its_twin() {
        let hex = ThemeAwareValue::from("#1e3a8a");
        let variables = mark_variables(Some(&hex), Color::Warning).to_string();

        assert!(
            variables.contains(&format!("{}:#FFFFFF;", MARK_COLOR_VAR.name())),
            "{variables}"
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
