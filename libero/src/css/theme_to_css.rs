use crate::theme::{
    Color, ColorShade, ColorValue, NamedColorCss, STACK_COLUMN_ALIGN, STACK_COLUMN_JUSTIFY,
    STACK_COLUMN_SPACING, STACK_COLUMN_WRAP, STACK_ROW_ALIGN, STACK_ROW_JUSTIFY, STACK_ROW_SPACING,
    STACK_ROW_WRAP, SizeCss, Theme,
};

use super::{CssDeclaration, CssScope, Stylesheet, css_color_value::CssColorValue};

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
        Stylesheet::new(vec![CssScope::new(":root", theme_declarations(theme))])
    }
}

fn theme_declarations(theme: &Theme) -> Vec<CssDeclaration> {
    let mut declarations = Vec::new();
    push_spacing_declarations(&mut declarations, theme);
    push_stack_declarations(&mut declarations, theme);
    push_named_color_declarations(&mut declarations, theme);
    push_color_declarations(&mut declarations, Color::Primary, theme.primary);
    push_color_declarations(&mut declarations, Color::Secondary, theme.secondary);
    push_color_declarations(&mut declarations, Color::Error, theme.error);
    push_color_declarations(&mut declarations, Color::Warning, theme.warning);
    push_color_declarations(&mut declarations, Color::Info, theme.info);
    push_color_declarations(&mut declarations, Color::Success, theme.success);
    push_color_declarations(&mut declarations, Color::Grey, theme.grey);
    declarations
}

fn push_spacing_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    declarations.push(CssDeclaration::new(
        SizeCss::SPACING.name(crate::theme::Size::Xs),
        format!("{}px", theme.spacing.xs),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::SPACING.name(crate::theme::Size::Sm),
        format!("{}px", theme.spacing.sm),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::SPACING.name(crate::theme::Size::Md),
        format!("{}px", theme.spacing.md),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::SPACING.name(crate::theme::Size::Lg),
        format!("{}px", theme.spacing.lg),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::SPACING.name(crate::theme::Size::Xl),
        format!("{}px", theme.spacing.xl),
    ));
}

fn push_stack_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    declarations.push(CssDeclaration::new(
        STACK_COLUMN_ALIGN.name(),
        theme.stack.column.align,
    ));
    declarations.push(CssDeclaration::new(
        STACK_COLUMN_JUSTIFY.name(),
        theme.stack.column.justify,
    ));
    declarations.push(CssDeclaration::new(
        STACK_COLUMN_SPACING.name(),
        SizeCss::SPACING.value(theme.stack.column.spacing),
    ));
    declarations.push(CssDeclaration::new(
        STACK_COLUMN_WRAP.name(),
        if theme.stack.column.wrap {
            "wrap"
        } else {
            "nowrap"
        },
    ));
    declarations.push(CssDeclaration::new(
        STACK_ROW_ALIGN.name(),
        theme.stack.row.align,
    ));
    declarations.push(CssDeclaration::new(
        STACK_ROW_JUSTIFY.name(),
        theme.stack.row.justify,
    ));
    declarations.push(CssDeclaration::new(
        STACK_ROW_SPACING.name(),
        SizeCss::SPACING.value(theme.stack.row.spacing),
    ));
    declarations.push(CssDeclaration::new(
        STACK_ROW_WRAP.name(),
        if theme.stack.row.wrap {
            "wrap"
        } else {
            "nowrap"
        },
    ));
}

fn push_named_color_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    declarations.push(CssDeclaration::new(
        NamedColorCss::BLACK.name(),
        theme.black.to_string(),
    ));
    declarations.push(CssDeclaration::new(
        NamedColorCss::WHITE.name(),
        theme.white.to_string(),
    ));
}

fn push_color_declarations(
    declarations: &mut Vec<CssDeclaration>,
    color: Color,
    base: crate::theme::HexColor,
) {
    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            CssColorValue(ColorValue::Shade(color, shade)).var_name(),
            base.shade(shade).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            CssColorValue(ColorValue::Contrast(color, shade)).var_name(),
            if base.shade(shade).contrast().rgb() == 0x00_00_00 {
                CssColorValue(ColorValue::Shade(Color::Black, shade)).value()
            } else {
                CssColorValue(ColorValue::Shade(Color::White, shade)).value()
            },
        ));
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
            crate::theme::StackDefaults::new(
                crate::theme::StackAxisDefaults::new(
                    "stretch",
                    "flex-start",
                    crate::theme::Size::Md,
                    false,
                ),
                crate::theme::StackAxisDefaults::new(
                    "center",
                    "flex-start",
                    crate::theme::Size::Md,
                    true,
                ),
            ),
            HexColor::new(0x228BE6),
            HexColor::new(0xE03131),
            HexColor::new(0xE03131),
            HexColor::new(0xF08C00),
            HexColor::new(0x228BE6),
            HexColor::new(0x2F9E44),
            HexColor::new(0x868E96),
            HexColor::new(0x000000),
            HexColor::new(0xFFFFFF),
        );
        let css = Stylesheet::from(&THEME);

        assert!(css.as_str().contains("--lsx-spacing-xs:4px;"));
        assert!(css.as_str().contains("--lsx-spacing-xl:20px;"));
        assert!(css.as_str().contains("--lsx-stack-column-align:stretch;"));
        assert!(
            css.as_str()
                .contains("--lsx-stack-column-justify:flex-start;")
        );
        assert!(
            css.as_str()
                .contains("--lsx-stack-column-spacing:var(--lsx-spacing-md);")
        );
        assert!(css.as_str().contains("--lsx-stack-column-wrap:nowrap;"));
        assert!(css.as_str().contains("--lsx-stack-row-align:center;"));
        assert!(css.as_str().contains("--lsx-stack-row-justify:flex-start;"));
        assert!(
            css.as_str()
                .contains("--lsx-stack-row-spacing:var(--lsx-spacing-md);")
        );
        assert!(css.as_str().contains("--lsx-stack-row-wrap:wrap;"));
        assert!(css.as_str().contains("--lsx-primary-1:#D2E7FA;"));
        assert!(css.as_str().contains("--lsx-black:#000000;"));
        assert!(css.as_str().contains("--lsx-white:#FFFFFF;"));
        assert!(
            css.as_str()
                .contains("--lsx-primary-contrast-1:var(--lsx-black);")
        );
        assert!(
            css.as_str()
                .contains("--lsx-primary-contrast-7:var(--lsx-white);")
        );
        assert!(css.as_str().contains("--lsx-secondary-7:#E03131;"));
        assert!(css.as_str().contains("--lsx-error-7:#E03131;"));
        assert!(css.as_str().contains("--lsx-warning-7:#F08C00;"));
        assert!(css.as_str().contains("--lsx-info-7:#228BE6;"));
        assert!(css.as_str().contains("--lsx-success-7:#2F9E44;"));
        assert!(css.as_str().contains("--lsx-grey-7:#868E96;"));
        assert!(
            css.as_str()
                .contains("--lsx-grey-contrast-7:var(--lsx-black);")
        );
        assert!(css.as_str().starts_with(":root{"));
        assert!(css.as_str().ends_with("}"));
    }
}
