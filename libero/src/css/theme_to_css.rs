use crate::{
    sx::{Declaration, DeclarationProperty, ThemeAwareValue},
    theme::{Color, ColorShade, ColorValue, HexColor, Size, Theme},
};

use super::{CssScope, Stylesheet, css_color_value::CssColorValue};

impl Theme {
    pub fn to_css(&self) -> Stylesheet {
        Stylesheet::new().append_scope(self.to_css_scope())
    }

    fn to_css_scope(&self) -> CssScope {
        let mut scope = CssScope::new(":root");
        scope = self.extend_scope_with_spacing_vars(scope);
        scope = self.extend_scope_with_color_vars(scope, Color::Primary, self.primary);
        self.extend_scope_with_color_vars(scope, Color::Secondary, self.secondary)
    }

    fn extend_scope_with_spacing_vars(&self, scope: CssScope) -> CssScope {
        scope
            .with(to_spacing_var_declaration(Size::Xs, self.spacing.xs))
            .with(to_spacing_var_declaration(Size::Sm, self.spacing.sm))
            .with(to_spacing_var_declaration(Size::Md, self.spacing.md))
            .with(to_spacing_var_declaration(Size::Lg, self.spacing.lg))
            .with(to_spacing_var_declaration(Size::Xl, self.spacing.xl))
    }

    fn extend_scope_with_color_vars(
        &self,
        scope: CssScope,
        color: Color,
        base: HexColor,
    ) -> CssScope {
        scope
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S1),
                base.shade(ColorShade::S1),
            ))
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S2),
                base.shade(ColorShade::S2),
            ))
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S3),
                base.shade(ColorShade::S3),
            ))
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S4),
                base.shade(ColorShade::S4),
            ))
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S5),
                base.shade(ColorShade::S5),
            ))
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S6),
                base.shade(ColorShade::S6),
            ))
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S7),
                base.shade(ColorShade::S7),
            ))
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S8),
                base.shade(ColorShade::S8),
            ))
            .with(to_color_var_declaration(
                ColorValue::Shade(color, ColorShade::S9),
                base.shade(ColorShade::S9),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S1),
                base.shade(ColorShade::S1).contrast(),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S2),
                base.shade(ColorShade::S2).contrast(),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S3),
                base.shade(ColorShade::S3).contrast(),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S4),
                base.shade(ColorShade::S4).contrast(),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S5),
                base.shade(ColorShade::S5).contrast(),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S6),
                base.shade(ColorShade::S6).contrast(),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S7),
                base.shade(ColorShade::S7).contrast(),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S8),
                base.shade(ColorShade::S8).contrast(),
            ))
            .with(to_color_var_declaration(
                ColorValue::Contrast(color, ColorShade::S9),
                base.shade(ColorShade::S9).contrast(),
            ))
    }
}

fn to_spacing_var_declaration(size: Size, value: u8) -> Declaration {
    Declaration {
        property: DeclarationProperty::Raw(Box::leak(
            crate::css::SizeCssVar::SPACING
                .to_string_name(size)
                .into_boxed_str(),
        )),
        value: ThemeAwareValue::Raw(Box::leak(format!("{}px", value).into_boxed_str())),
    }
}

fn to_color_var_declaration(color_value: ColorValue, color: HexColor) -> Declaration {
    Declaration {
        property: DeclarationProperty::Raw(Box::leak(
            CssColorValue(color_value)
                .to_string_var_name()
                .into_boxed_str(),
        )),
        value: ThemeAwareValue::Raw(Box::leak(color.to_string().into_boxed_str())),
    }
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
        let css = THEME.to_css();

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
