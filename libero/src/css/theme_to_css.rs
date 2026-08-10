use crate::{
    sx::{Declaration, DeclarationProperty, ThemeAwareValue},
    theme::{Color, ColorShade, ColorValue, HexColor, Size, Theme},
};

use super::{CssScope, Stylesheet, css_color_value::CssColorValue};

impl Theme {
    pub const fn to_css(&self) -> Stylesheet {
        Stylesheet::new().append_scope(self.to_css_scope())
    }

    const fn to_css_scope(&self) -> CssScope {
        let mut scope = CssScope::new(":root");
        scope = self.push_spacing_vars(scope);
        scope = self.push_color_vars(scope, Color::Primary, self.primary);
        self.push_color_vars(scope, Color::Secondary, self.secondary)
    }

    const fn push_spacing_vars(&self, mut scope: CssScope) -> CssScope {
        scope = push_spacing_var(scope, Size::Xs, self.spacing.xs);
        scope = push_spacing_var(scope, Size::Sm, self.spacing.sm);
        scope = push_spacing_var(scope, Size::Md, self.spacing.md);
        scope = push_spacing_var(scope, Size::Lg, self.spacing.lg);
        scope = push_spacing_var(scope, Size::Xl, self.spacing.xl);
        scope
    }

    const fn push_color_vars(&self, mut scope: CssScope, color: Color, base: HexColor) -> CssScope {
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S1),
            base.shade(ColorShade::S1),
        );
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S2),
            base.shade(ColorShade::S2),
        );
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S3),
            base.shade(ColorShade::S3),
        );
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S4),
            base.shade(ColorShade::S4),
        );
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S5),
            base.shade(ColorShade::S5),
        );
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S6),
            base.shade(ColorShade::S6),
        );
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S7),
            base.shade(ColorShade::S7),
        );
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S8),
            base.shade(ColorShade::S8),
        );
        scope = push_color_var(
            scope,
            ColorValue::Shade(color, ColorShade::S9),
            base.shade(ColorShade::S9),
        );
        scope = push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S1),
            base.shade(ColorShade::S1).contrast(),
        );
        scope = push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S2),
            base.shade(ColorShade::S2).contrast(),
        );
        scope = push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S3),
            base.shade(ColorShade::S3).contrast(),
        );
        scope = push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S4),
            base.shade(ColorShade::S4).contrast(),
        );
        scope = push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S5),
            base.shade(ColorShade::S5).contrast(),
        );
        scope = push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S6),
            base.shade(ColorShade::S6).contrast(),
        );
        scope = push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S7),
            base.shade(ColorShade::S7).contrast(),
        );
        scope = push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S8),
            base.shade(ColorShade::S8).contrast(),
        );
        push_color_var(
            scope,
            ColorValue::Contrast(color, ColorShade::S9),
            base.shade(ColorShade::S9).contrast(),
        )
    }
}

const fn push_spacing_var(scope: CssScope, size: Size, value: u8) -> CssScope {
    scope.with(Declaration {
        property: DeclarationProperty::RawConstStr(
            crate::css::SizeCssVar::SPACING.to_const_str_name(size),
        ),
        value: ThemeAwareValue::RawConstStr(
            append_u8(crate::common::ConstStr::new(), value).push_str("px"),
        ),
    })
}

const fn push_color_var(scope: CssScope, color_value: ColorValue, color: HexColor) -> CssScope {
    scope.with(Declaration {
        property: DeclarationProperty::RawConstStr(
            CssColorValue(color_value).to_const_str_var_name(),
        ),
        value: ThemeAwareValue::RawConstStr(color.to_const_str()),
    })
}

const fn append_u8(mut css: crate::common::ConstStr, value: u8) -> crate::common::ConstStr {
    if value >= 100 {
        css = css.push_char((b'0' + (value / 100)) as char);
        css = css.push_char((b'0' + ((value / 10) % 10)) as char);
        css = css.push_char((b'0' + (value % 10)) as char);
        return css;
    }

    if value >= 10 {
        css = css.push_char((b'0' + (value / 10)) as char);
        css = css.push_char((b'0' + (value % 10)) as char);
        return css;
    }

    css.push_char((b'0' + value) as char)
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
        const CSS: Stylesheet = THEME.to_css();

        assert!(CSS.as_str().contains("--lsx-spacing-xs:4px;"));
        assert!(CSS.as_str().contains("--lsx-spacing-xl:20px;"));
        assert!(CSS.as_str().contains("--lsx-primary-1:#D2E7FA;"));
        assert!(CSS.as_str().contains("--lsx-primary-contrast-1:#000000;"));
        assert!(CSS.as_str().contains("--lsx-primary-contrast-7:#FFFFFF;"));
        assert!(CSS.as_str().contains("--lsx-secondary-7:#E03131;"));
        assert!(CSS.as_str().starts_with(":root{"));
        assert!(CSS.as_str().ends_with("}"));
    }
}
