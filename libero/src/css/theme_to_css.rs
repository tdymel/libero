use crate::{
    common::ConstStr,
    theme::{Color, ColorShade, ColorValue, HexColor, Size, Theme},
};

impl Theme {
    pub const fn to_css(&self) -> ConstStr {
        let mut css = ConstStr::new();
        css = css.push_str(":root{");
        css = self.push_spacing_vars(css);
        css = self.push_color_vars(css, Color::Primary, self.primary);
        css = self.push_color_vars(css, Color::Secondary, self.secondary);
        css.push_char('}')
    }

    const fn push_spacing_vars(&self, mut css: ConstStr) -> ConstStr {
        css = push_spacing_var(css, Size::Xs, self.spacing.xs);
        css = push_spacing_var(css, Size::Sm, self.spacing.sm);
        css = push_spacing_var(css, Size::Md, self.spacing.md);
        css = push_spacing_var(css, Size::Lg, self.spacing.lg);
        css = push_spacing_var(css, Size::Xl, self.spacing.xl);
        css
    }

    const fn push_color_vars(&self, mut css: ConstStr, color: Color, base: HexColor) -> ConstStr {
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S1),
            base.shade(ColorShade::S1),
        );
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S2),
            base.shade(ColorShade::S2),
        );
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S3),
            base.shade(ColorShade::S3),
        );
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S4),
            base.shade(ColorShade::S4),
        );
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S5),
            base.shade(ColorShade::S5),
        );
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S6),
            base.shade(ColorShade::S6),
        );
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S7),
            base.shade(ColorShade::S7),
        );
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S8),
            base.shade(ColorShade::S8),
        );
        css = push_color_var(
            css,
            ColorValue::Shade(color, ColorShade::S9),
            base.shade(ColorShade::S9),
        );
        push_color_var(css, ColorValue::Contrast(color), base.contrast())
    }
}

const fn push_spacing_var(mut css: ConstStr, size: Size, value: u8) -> ConstStr {
    css = crate::css::SizeCssVar::SPACING.push_name(css, size);
    css = css.push_char(':');
    css = push_u8(css, value);
    css = css.push_str("px;");
    css
}

const fn push_color_var(mut css: ConstStr, color_value: ColorValue, color: HexColor) -> ConstStr {
    css = color_value.push_var_name(css);
    css = css.push_char(':');
    css = color.push_hex(css);
    css = css.push_char(';');
    css
}

const fn push_u8(mut css: ConstStr, value: u8) -> ConstStr {
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
        const CSS: ConstStr = THEME.to_css();

        assert!(CSS.as_str().contains("--lsx-spacing-xs:4px;"));
        assert!(CSS.as_str().contains("--lsx-spacing-xl:20px;"));
        assert!(CSS.as_str().contains("--lsx-primary-100:#D2E7FA;"));
        assert!(CSS.as_str().contains("--lsx-primary-contrast:#FFFFFF;"));
        assert!(CSS.as_str().contains("--lsx-secondary-700:#E03131;"));
        assert!(CSS.as_str().starts_with(":root{"));
        assert!(CSS.as_str().ends_with("}"));
    }
}
