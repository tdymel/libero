use crate::{
    common::ConstStr,
    theme::{Color, ColorShade, ColorValue, HexColor, Size, Theme},
};

use super::Stylesheet;

impl Theme {
    pub const fn to_css(&self) -> Stylesheet {
        let mut css = Stylesheet::new();
        css = css.start_root();
        css = self.push_spacing_vars(css);
        css = self.push_color_vars(css, Color::Primary, self.primary);
        css = self.push_color_vars(css, Color::Secondary, self.secondary);
        css.end_block()
    }

    const fn push_spacing_vars(&self, mut css: Stylesheet) -> Stylesheet {
        css = push_spacing_var(css, Size::Xs, self.spacing.xs);
        css = push_spacing_var(css, Size::Sm, self.spacing.sm);
        css = push_spacing_var(css, Size::Md, self.spacing.md);
        css = push_spacing_var(css, Size::Lg, self.spacing.lg);
        css = push_spacing_var(css, Size::Xl, self.spacing.xl);
        css
    }

    const fn push_color_vars(
        &self,
        mut css: Stylesheet,
        color: Color,
        base: HexColor,
    ) -> Stylesheet {
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
        css = push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S1),
            base.shade(ColorShade::S1).contrast(),
        );
        css = push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S2),
            base.shade(ColorShade::S2).contrast(),
        );
        css = push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S3),
            base.shade(ColorShade::S3).contrast(),
        );
        css = push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S4),
            base.shade(ColorShade::S4).contrast(),
        );
        css = push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S5),
            base.shade(ColorShade::S5).contrast(),
        );
        css = push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S6),
            base.shade(ColorShade::S6).contrast(),
        );
        css = push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S7),
            base.shade(ColorShade::S7).contrast(),
        );
        css = push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S8),
            base.shade(ColorShade::S8).contrast(),
        );
        push_color_var(
            css,
            ColorValue::Contrast(color, ColorShade::S9),
            base.shade(ColorShade::S9).contrast(),
        )
    }
}

const fn push_spacing_var(mut css: Stylesheet, size: Size, value: u8) -> Stylesheet {
    css = Stylesheet::from_const_str(
        crate::css::SizeCssVar::SPACING.push_name(css.into_const_str(), size),
    );
    css = css.push_char(':');
    css = Stylesheet::from_const_str(push_u8(css.into_const_str(), value));
    css = css.end_declaration();
    css
}

const fn push_color_var(
    mut css: Stylesheet,
    color_value: ColorValue,
    color: HexColor,
) -> Stylesheet {
    css = Stylesheet::from_const_str(color_value.push_var_name(css.into_const_str()));
    css = css.push_char(':');
    css = Stylesheet::from_const_str(color.push_hex(css.into_const_str()));
    css = css.end_declaration();
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
