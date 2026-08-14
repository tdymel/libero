use crate::theme::{
    BUTTON_FONT_SIZE_LG, BUTTON_FONT_SIZE_MD, BUTTON_FONT_SIZE_SM, BUTTON_FONT_SIZE_XL,
    BUTTON_FONT_SIZE_XS, BUTTON_HEIGHT_LG, BUTTON_HEIGHT_MD, BUTTON_HEIGHT_SM, BUTTON_HEIGHT_XL,
    BUTTON_HEIGHT_XS, BUTTON_PADDING_X_LG, BUTTON_PADDING_X_MD, BUTTON_PADDING_X_SM,
    BUTTON_PADDING_X_XL, BUTTON_PADDING_X_XS, BUTTON_RIPPLE_KEYFRAMES, CONTAINER_GUTTERS,
    CONTAINER_SIZE, Color, ColorShade, ColorValue, DIVIDER_SPACING, FLEX_COLUMN_ALIGN,
    FLEX_COLUMN_JUSTIFY, FLEX_COLUMN_SPACING, FLEX_COLUMN_WRAP, FLEX_ROW_ALIGN, FLEX_ROW_JUSTIFY,
    FLEX_ROW_SPACING, FLEX_ROW_WRAP, H1_FONT_FAMILY, H1_FONT_SIZE, H1_FONT_WEIGHT,
    H1_LETTER_SPACING, H1_LINE_HEIGHT, H2_FONT_FAMILY, H2_FONT_SIZE, H2_FONT_WEIGHT,
    H2_LETTER_SPACING, H2_LINE_HEIGHT, H3_FONT_FAMILY, H3_FONT_SIZE, H3_FONT_WEIGHT,
    H3_LETTER_SPACING, H3_LINE_HEIGHT, H4_FONT_FAMILY, H4_FONT_SIZE, H4_FONT_WEIGHT,
    H4_LETTER_SPACING, H4_LINE_HEIGHT, H5_FONT_FAMILY, H5_FONT_SIZE, H5_FONT_WEIGHT,
    H5_LETTER_SPACING, H5_LINE_HEIGHT, H6_FONT_FAMILY, H6_FONT_SIZE, H6_FONT_WEIGHT,
    H6_LETTER_SPACING, H6_LINE_HEIGHT, NamedColorCss, SizeCss, TEXT_FONT_FAMILY, TEXT_FONT_SIZE_LG,
    TEXT_FONT_SIZE_MD, TEXT_FONT_SIZE_SM, TEXT_FONT_SIZE_XL, TEXT_FONT_SIZE_XS,
    TEXT_FONT_WEIGHT_LG, TEXT_FONT_WEIGHT_MD, TEXT_FONT_WEIGHT_SM, TEXT_FONT_WEIGHT_XL,
    TEXT_FONT_WEIGHT_XS, TEXT_LETTER_SPACING_LG, TEXT_LETTER_SPACING_MD, TEXT_LETTER_SPACING_SM,
    TEXT_LETTER_SPACING_XL, TEXT_LETTER_SPACING_XS, TEXT_LINE_HEIGHT_LG, TEXT_LINE_HEIGHT_MD,
    TEXT_LINE_HEIGHT_SM, TEXT_LINE_HEIGHT_XL, TEXT_LINE_HEIGHT_XS, Theme,
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
        let mut scopes = vec![CssScope::new(":root", theme_declarations(theme))];
        scopes.extend(global_reset_scopes(theme));
        scopes.push(body_scope(theme));
        let mut css = Stylesheet::new(scopes).as_str().to_string();
        css.push_str(BUTTON_RIPPLE_KEYFRAMES);
        Stylesheet::from(css)
    }
}

fn global_reset_scopes(theme: &Theme) -> Vec<CssScope> {
    let mut html_declarations = vec![CssDeclaration::new("box-sizing", "border-box")];
    if theme.font_smoothing {
        html_declarations.push(CssDeclaration::new("-webkit-font-smoothing", "antialiased"));
        html_declarations.push(CssDeclaration::new("-moz-osx-font-smoothing", "grayscale"));
    }

    vec![
        CssScope::new("html", html_declarations),
        CssScope::new(
            "*, *::before, *::after",
            vec![CssDeclaration::new("box-sizing", "inherit")],
        ),
    ]
}

fn body_scope(theme: &Theme) -> CssScope {
    // Black/white have no per-shade contrast var of their own (unlike the
    // palette colors), so the contrast is computed here the same way
    // push_color_declarations does it for palette shades.
    let text_color_var = if theme.white.contrast().rgb() == 0x00_00_00 {
        NamedColorCss::BLACK.value()
    } else {
        NamedColorCss::WHITE.value()
    };

    CssScope::new(
        "body",
        vec![
            CssDeclaration::new("margin", "0"),
            CssDeclaration::new("background-color", NamedColorCss::WHITE.value()),
            CssDeclaration::new("color", text_color_var),
            CssDeclaration::new("font-family", TEXT_FONT_FAMILY.value()),
            CssDeclaration::new("font-size", TEXT_FONT_SIZE_MD.value()),
            CssDeclaration::new("font-weight", TEXT_FONT_WEIGHT_MD.value()),
            CssDeclaration::new("line-height", TEXT_LINE_HEIGHT_MD.value()),
            CssDeclaration::new("letter-spacing", TEXT_LETTER_SPACING_MD.value()),
        ],
    )
}

fn theme_declarations(theme: &Theme) -> Vec<CssDeclaration> {
    let mut declarations = Vec::new();
    push_spacing_declarations(&mut declarations, theme);
    push_radius_declarations(&mut declarations, theme);
    push_breakpoint_declarations(&mut declarations);
    push_flex_declarations(&mut declarations, theme);
    push_container_declarations(&mut declarations, theme);
    push_divider_declarations(&mut declarations, theme);
    push_button_declarations(&mut declarations, theme);
    push_title_declarations(&mut declarations, theme);
    push_text_declarations(&mut declarations, theme);
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

fn push_radius_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    declarations.push(CssDeclaration::new(
        SizeCss::RADIUS.name(crate::theme::Size::Xs),
        format!("{}px", theme.radius.xs),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::RADIUS.name(crate::theme::Size::Sm),
        format!("{}px", theme.radius.sm),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::RADIUS.name(crate::theme::Size::Md),
        format!("{}px", theme.radius.md),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::RADIUS.name(crate::theme::Size::Lg),
        format!("{}px", theme.radius.lg),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::RADIUS.name(crate::theme::Size::Xl),
        format!("{}px", theme.radius.xl),
    ));
}

fn push_breakpoint_declarations(declarations: &mut Vec<CssDeclaration>) {
    declarations.push(CssDeclaration::new(
        SizeCss::BREAKPOINT.name(crate::theme::Size::Xs),
        crate::theme::Size::Xs.breakpoint_value(),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::BREAKPOINT.name(crate::theme::Size::Sm),
        crate::theme::Size::Sm.breakpoint_value(),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::BREAKPOINT.name(crate::theme::Size::Md),
        crate::theme::Size::Md.breakpoint_value(),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::BREAKPOINT.name(crate::theme::Size::Lg),
        crate::theme::Size::Lg.breakpoint_value(),
    ));
    declarations.push(CssDeclaration::new(
        SizeCss::BREAKPOINT.name(crate::theme::Size::Xl),
        crate::theme::Size::Xl.breakpoint_value(),
    ));
}

fn push_flex_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    declarations.push(CssDeclaration::new(
        FLEX_COLUMN_ALIGN.name(),
        theme.flex.column.align,
    ));
    declarations.push(CssDeclaration::new(
        FLEX_COLUMN_JUSTIFY.name(),
        theme.flex.column.justify,
    ));
    declarations.push(CssDeclaration::new(
        FLEX_COLUMN_SPACING.name(),
        SizeCss::SPACING.value(theme.flex.column.spacing),
    ));
    declarations.push(CssDeclaration::new(
        FLEX_COLUMN_WRAP.name(),
        if theme.flex.column.wrap {
            "wrap"
        } else {
            "nowrap"
        },
    ));
    declarations.push(CssDeclaration::new(
        FLEX_ROW_ALIGN.name(),
        theme.flex.row.align,
    ));
    declarations.push(CssDeclaration::new(
        FLEX_ROW_JUSTIFY.name(),
        theme.flex.row.justify,
    ));
    declarations.push(CssDeclaration::new(
        FLEX_ROW_SPACING.name(),
        SizeCss::SPACING.value(theme.flex.row.spacing),
    ));
    declarations.push(CssDeclaration::new(
        FLEX_ROW_WRAP.name(),
        if theme.flex.row.wrap {
            "wrap"
        } else {
            "nowrap"
        },
    ));
}

fn push_container_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    declarations.push(CssDeclaration::new(
        CONTAINER_SIZE.name(),
        SizeCss::BREAKPOINT.value(theme.container.size),
    ));
    declarations.push(CssDeclaration::new(
        CONTAINER_GUTTERS.name(),
        SizeCss::SPACING.value(theme.container.gutters),
    ));
}

fn push_divider_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    let spacing = match theme.divider.spacing {
        Some(size) => SizeCss::SPACING.value(size),
        None => "0".to_string(),
    };
    declarations.push(CssDeclaration::new(DIVIDER_SPACING.name(), spacing));
}

fn push_button_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    declarations.push(CssDeclaration::new(
        BUTTON_FONT_SIZE_XS.name(),
        theme.button.xs.font_size,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_HEIGHT_XS.name(),
        theme.button.xs.height,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_PADDING_X_XS.name(),
        theme.button.xs.padding_x,
    ));

    declarations.push(CssDeclaration::new(
        BUTTON_FONT_SIZE_SM.name(),
        theme.button.sm.font_size,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_HEIGHT_SM.name(),
        theme.button.sm.height,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_PADDING_X_SM.name(),
        theme.button.sm.padding_x,
    ));

    declarations.push(CssDeclaration::new(
        BUTTON_FONT_SIZE_MD.name(),
        theme.button.md.font_size,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_HEIGHT_MD.name(),
        theme.button.md.height,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_PADDING_X_MD.name(),
        theme.button.md.padding_x,
    ));

    declarations.push(CssDeclaration::new(
        BUTTON_FONT_SIZE_LG.name(),
        theme.button.lg.font_size,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_HEIGHT_LG.name(),
        theme.button.lg.height,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_PADDING_X_LG.name(),
        theme.button.lg.padding_x,
    ));

    declarations.push(CssDeclaration::new(
        BUTTON_FONT_SIZE_XL.name(),
        theme.button.xl.font_size,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_HEIGHT_XL.name(),
        theme.button.xl.height,
    ));
    declarations.push(CssDeclaration::new(
        BUTTON_PADDING_X_XL.name(),
        theme.button.xl.padding_x,
    ));
}

fn push_text_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    // Font family (shared)
    declarations.push(CssDeclaration::new(
        TEXT_FONT_FAMILY.name(),
        theme.texts.font_family,
    ));

    // XS
    declarations.push(CssDeclaration::new(
        TEXT_FONT_WEIGHT_XS.name(),
        theme.texts.xs.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_FONT_SIZE_XS.name(),
        theme.texts.xs.font_size,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LETTER_SPACING_XS.name(),
        theme.texts.xs.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LINE_HEIGHT_XS.name(),
        theme.texts.xs.line_height,
    ));

    // SM
    declarations.push(CssDeclaration::new(
        TEXT_FONT_WEIGHT_SM.name(),
        theme.texts.sm.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_FONT_SIZE_SM.name(),
        theme.texts.sm.font_size,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LETTER_SPACING_SM.name(),
        theme.texts.sm.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LINE_HEIGHT_SM.name(),
        theme.texts.sm.line_height,
    ));

    // MD
    declarations.push(CssDeclaration::new(
        TEXT_FONT_WEIGHT_MD.name(),
        theme.texts.md.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_FONT_SIZE_MD.name(),
        theme.texts.md.font_size,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LETTER_SPACING_MD.name(),
        theme.texts.md.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LINE_HEIGHT_MD.name(),
        theme.texts.md.line_height,
    ));

    // LG
    declarations.push(CssDeclaration::new(
        TEXT_FONT_WEIGHT_LG.name(),
        theme.texts.lg.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_FONT_SIZE_LG.name(),
        theme.texts.lg.font_size,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LETTER_SPACING_LG.name(),
        theme.texts.lg.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LINE_HEIGHT_LG.name(),
        theme.texts.lg.line_height,
    ));

    // XL
    declarations.push(CssDeclaration::new(
        TEXT_FONT_WEIGHT_XL.name(),
        theme.texts.xl.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_FONT_SIZE_XL.name(),
        theme.texts.xl.font_size,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LETTER_SPACING_XL.name(),
        theme.texts.xl.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        TEXT_LINE_HEIGHT_XL.name(),
        theme.texts.xl.line_height,
    ));
}

fn push_title_declarations(declarations: &mut Vec<CssDeclaration>, theme: &Theme) {
    // H1
    declarations.push(CssDeclaration::new(
        H1_FONT_FAMILY.name(),
        theme.titles.h1.font_family,
    ));
    declarations.push(CssDeclaration::new(
        H1_FONT_WEIGHT.name(),
        theme.titles.h1.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        H1_FONT_SIZE.name(),
        theme.titles.h1.font_size,
    ));
    declarations.push(CssDeclaration::new(
        H1_LETTER_SPACING.name(),
        theme.titles.h1.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        H1_LINE_HEIGHT.name(),
        theme.titles.h1.line_height,
    ));

    // H2
    declarations.push(CssDeclaration::new(
        H2_FONT_FAMILY.name(),
        theme.titles.h2.font_family,
    ));
    declarations.push(CssDeclaration::new(
        H2_FONT_WEIGHT.name(),
        theme.titles.h2.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        H2_FONT_SIZE.name(),
        theme.titles.h2.font_size,
    ));
    declarations.push(CssDeclaration::new(
        H2_LETTER_SPACING.name(),
        theme.titles.h2.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        H2_LINE_HEIGHT.name(),
        theme.titles.h2.line_height,
    ));

    // H3
    declarations.push(CssDeclaration::new(
        H3_FONT_FAMILY.name(),
        theme.titles.h3.font_family,
    ));
    declarations.push(CssDeclaration::new(
        H3_FONT_WEIGHT.name(),
        theme.titles.h3.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        H3_FONT_SIZE.name(),
        theme.titles.h3.font_size,
    ));
    declarations.push(CssDeclaration::new(
        H3_LETTER_SPACING.name(),
        theme.titles.h3.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        H3_LINE_HEIGHT.name(),
        theme.titles.h3.line_height,
    ));

    // H4
    declarations.push(CssDeclaration::new(
        H4_FONT_FAMILY.name(),
        theme.titles.h4.font_family,
    ));
    declarations.push(CssDeclaration::new(
        H4_FONT_WEIGHT.name(),
        theme.titles.h4.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        H4_FONT_SIZE.name(),
        theme.titles.h4.font_size,
    ));
    declarations.push(CssDeclaration::new(
        H4_LETTER_SPACING.name(),
        theme.titles.h4.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        H4_LINE_HEIGHT.name(),
        theme.titles.h4.line_height,
    ));

    // H5
    declarations.push(CssDeclaration::new(
        H5_FONT_FAMILY.name(),
        theme.titles.h5.font_family,
    ));
    declarations.push(CssDeclaration::new(
        H5_FONT_WEIGHT.name(),
        theme.titles.h5.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        H5_FONT_SIZE.name(),
        theme.titles.h5.font_size,
    ));
    declarations.push(CssDeclaration::new(
        H5_LETTER_SPACING.name(),
        theme.titles.h5.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        H5_LINE_HEIGHT.name(),
        theme.titles.h5.line_height,
    ));

    // H6
    declarations.push(CssDeclaration::new(
        H6_FONT_FAMILY.name(),
        theme.titles.h6.font_family,
    ));
    declarations.push(CssDeclaration::new(
        H6_FONT_WEIGHT.name(),
        theme.titles.h6.font_weight,
    ));
    declarations.push(CssDeclaration::new(
        H6_FONT_SIZE.name(),
        theme.titles.h6.font_size,
    ));
    declarations.push(CssDeclaration::new(
        H6_LETTER_SPACING.name(),
        theme.titles.h6.letter_spacing,
    ));
    declarations.push(CssDeclaration::new(
        H6_LINE_HEIGHT.name(),
        theme.titles.h6.line_height,
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

    use super::*;

    #[test]
    fn theme_css_happy_path() {
        let theme = &Theme::DEFAULT;
        let css = Stylesheet::from(theme);

        assert!(css.as_str().contains(BUTTON_RIPPLE_KEYFRAMES));
        assert!(css.as_str().contains("--lsx-spacing-xs:4px;"));
        assert!(css.as_str().contains("--lsx-spacing-xl:20px;"));
        assert!(css.as_str().contains("--lsx-breakpoint-xs:36rem;"));
        assert!(css.as_str().contains("--lsx-breakpoint-xl:88rem;"));
        assert!(
            css.as_str()
                .contains("--lsx-container-size:var(--lsx-breakpoint-lg);")
        );
        assert!(
            css.as_str()
                .contains("--lsx-container-gutters:var(--lsx-spacing-md);")
        );
        assert!(css.as_str().contains("--lsx-divider-spacing:0;"));
        assert!(css.as_str().contains("--lsx-flex-column-align:stretch;"));
        assert!(
            css.as_str()
                .contains("--lsx-flex-column-justify:flex-start;")
        );
        assert!(
            css.as_str()
                .contains("--lsx-flex-column-spacing:var(--lsx-spacing-md);")
        );
        assert!(css.as_str().contains("--lsx-flex-column-wrap:nowrap;"));
        assert!(css.as_str().contains("--lsx-flex-row-align:center;"));
        assert!(css.as_str().contains("--lsx-flex-row-justify:flex-start;"));
        assert!(
            css.as_str()
                .contains("--lsx-flex-row-spacing:var(--lsx-spacing-md);")
        );
        assert!(css.as_str().contains("--lsx-flex-row-wrap:wrap;"));
        assert!(css.as_str().contains("--lsx-h1-font-size:2.125rem;"));
        assert!(css.as_str().contains("--lsx-h1-font-weight:700;"));
        assert!(css.as_str().contains("--lsx-h2-font-size:1.625rem;"));
        assert!(css.as_str().contains("--lsx-h3-font-size:1.375rem;"));
        assert!(css.as_str().contains("--lsx-h4-font-size:1rem;"));
        assert!(css.as_str().contains("--lsx-h5-font-size:0.875rem;"));
        assert!(css.as_str().contains("--lsx-h6-font-size:0.75rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-xs:0.75rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-sm:0.875rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-md:1rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-lg:1.125rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-xl:1.25rem;"));
        assert!(css.as_str().contains("--lsx-text-font-weight-xs:400;"));
        assert!(css.as_str().contains("--lsx-text-line-height-md:1.5;"));

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

    #[test]
    fn theme_css_sets_body_and_global_box_sizing_defaults() {
        let theme = &Theme::DEFAULT;
        let css = Stylesheet::from(theme);
        let css = css.as_str();

        assert!(css.contains("html{box-sizing:border-box;"));
        assert!(css.contains("-webkit-font-smoothing:antialiased;"));
        assert!(css.contains("-moz-osx-font-smoothing:grayscale;"));
        assert!(css.contains("*, *::before, *::after{box-sizing:inherit;}"));

        assert!(css.contains("body{margin:0;"));
        assert!(css.contains("background-color:var(--lsx-white);"));
        // Default theme's white has high luminance, so its contrast is black.
        assert!(css.contains("color:var(--lsx-black);"));
        assert!(css.contains("font-family:var(--lsx-text-font-family);"));
        assert!(css.contains("font-size:var(--lsx-text-font-size-md);"));
    }

    #[test]
    fn theme_css_omits_font_smoothing_when_disabled() {
        let mut theme = Theme::DEFAULT;
        theme.font_smoothing = false;
        let css = Stylesheet::from(&theme);
        let css = css.as_str();

        assert!(css.contains("html{box-sizing:border-box;}"));
        assert!(!css.contains("font-smoothing"));
    }
}
