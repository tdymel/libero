use crate::theme::{Color, ColorShade, ColorValue, Theme};

use super::{CssDeclaration, CssScope, Stylesheet, StylesheetBuilder, css_var::SizeCssVar};

const SHADES: [ColorShade; 9] = [
    ColorShade::S1,
    ColorShade::S2,
    ColorShade::S3,
    ColorShade::S4,
    ColorShade::S5,
    ColorShade::S6,
    ColorShade::S7,
    ColorShade::S8,
    ColorShade::S9,
];

impl From<&Theme> for Stylesheet {
    fn from(theme: &Theme) -> Self {
        let mut scope = CssScope::new(":root");
        scope = push_spacing_vars(scope, theme);
        scope = push_color_vars(scope, Color::Primary, theme.primary);
        scope = push_color_vars(scope, Color::Secondary, theme.secondary);
        Stylesheet::from(StylesheetBuilder::new().with_scope(scope))
    }
}

fn push_spacing_vars(mut scope: CssScope, theme: &Theme) -> CssScope {
    scope = scope.with(CssDeclaration::new(
        SizeCssVar::SPACING.name(crate::theme::Size::Xs),
        format!("{}px", theme.spacing.xs),
    ));
    scope = scope.with(CssDeclaration::new(
        SizeCssVar::SPACING.name(crate::theme::Size::Sm),
        format!("{}px", theme.spacing.sm),
    ));
    scope = scope.with(CssDeclaration::new(
        SizeCssVar::SPACING.name(crate::theme::Size::Md),
        format!("{}px", theme.spacing.md),
    ));
    scope = scope.with(CssDeclaration::new(
        SizeCssVar::SPACING.name(crate::theme::Size::Lg),
        format!("{}px", theme.spacing.lg),
    ));
    scope.with(CssDeclaration::new(
        SizeCssVar::SPACING.name(crate::theme::Size::Xl),
        format!("{}px", theme.spacing.xl),
    ))
}

fn push_color_vars(scope: CssScope, color: Color, base: crate::theme::HexColor) -> CssScope {
    let scope = SHADES.into_iter().fold(scope, |scope, shade| {
        scope.with(CssDeclaration::new(
            ColorValue::Shade(color, shade).css_var_name(),
            base.shade(shade).to_string(),
        ))
    });

    SHADES.into_iter().fold(scope, |scope, shade| {
        scope.with(CssDeclaration::new(
            ColorValue::Contrast(color, shade).css_var_name(),
            base.shade(shade).contrast().to_string(),
        ))
    })
}

#[cfg(test)]
mod tests {
    use crate::theme::{HexColor, Sizes};

    use super::*;

    #[test]
    fn theme_css_happy_path() {
        const THEME: Theme = Theme::new(
            Sizes::new(4, 8, 12, 16, 20),
            HexColor::new(0x228BE6),
            HexColor::new(0xE03131),
        );
        let css = Stylesheet::from(&THEME);

        assert!(css.as_str().contains("--lsx-spacing-xs:4px;"));
        assert!(css.as_str().contains("--lsx-spacing-xl:20px;"));
        assert!(css.as_str().contains("--lsx-primary-1:#D2E7FA;"));
        assert!(css.as_str().contains("--lsx-primary-contrast-1:#000000;"));
        assert!(css.as_str().contains("--lsx-primary-contrast-7:#FFFFFF;"));
        assert!(css.as_str().contains("--lsx-secondary-7:#E03131;"));
        assert!(css.as_str().starts_with(":root{"));
        assert!(css.as_str().ends_with("}"));
    }
}
