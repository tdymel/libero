use crate::css::{CssDeclaration, CssScope, Stylesheet, ToCssDeclarations};

use super::{
    BUTTON_RIPPLE_KEYFRAMES, Color, ColorShade, ColorValue, HexColor, NamedColorCss, Size, SizeCss,
    TEXT_FONT_FAMILY, TEXT_FONT_SIZE, TEXT_FONT_WEIGHT, TEXT_LETTER_SPACING, TEXT_LINE_HEIGHT,
    Theme,
};

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
            CssDeclaration::new("font-size", TEXT_FONT_SIZE.value(Size::Md)),
            CssDeclaration::new("font-weight", TEXT_FONT_WEIGHT.value(Size::Md)),
            CssDeclaration::new("line-height", TEXT_LINE_HEIGHT.value(Size::Md)),
            CssDeclaration::new("letter-spacing", TEXT_LETTER_SPACING.value(Size::Md)),
        ],
    )
}

fn theme_declarations(theme: &Theme) -> Vec<CssDeclaration> {
    let mut declarations = Vec::new();
    declarations.extend(theme.spacing.to_css_declarations(SizeCss::SPACING, "px"));
    declarations.extend(theme.radius.to_css_declarations(SizeCss::RADIUS, "px"));
    push_breakpoint_declarations(&mut declarations);
    declarations.extend(theme.dialog.to_css_declarations());
    declarations.extend(theme.drawer.to_css_declarations());
    declarations.extend(theme.flex.to_css_declarations());
    declarations.extend(theme.container.to_css_declarations());
    declarations.extend(theme.aspect_ratio.to_css_declarations());
    declarations.extend(theme.divider.to_css_declarations());
    declarations.extend(theme.button.to_css_declarations());
    declarations.extend(theme.select.to_css_declarations());
    declarations.extend(theme.list.to_css_declarations());
    declarations.extend(theme.data_list.to_css_declarations());
    declarations.extend(theme.titles.to_css_declarations());
    declarations.extend(theme.texts.to_css_declarations());
    declarations.extend(theme.code.to_css_declarations());
    declarations.extend(theme.header.to_css_declarations());
    declarations.extend(theme.icon.to_css_declarations());
    declarations.extend(theme.action_icon.to_css_declarations());
    declarations.extend(theme.qr_code.to_css_declarations());
    declarations.extend(theme.kbd.to_css_declarations());
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

// Breakpoints are interpolated directly into `@media` query strings
// (sx_to_css.rs), which can't reference CSS custom properties, so they stay
// hardcoded literals rather than a theme-configurable `Sizes<T>` scale.
fn push_breakpoint_declarations(declarations: &mut Vec<CssDeclaration>) {
    for size in super::Size::ALL {
        declarations.push(SizeCss::BREAKPOINT.declare(size, size.breakpoint_value()));
    }
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

fn push_color_declarations(declarations: &mut Vec<CssDeclaration>, color: Color, base: HexColor) {
    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Shade(color, shade).var_name(),
            base.shade(shade).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Contrast(color, shade).var_name(),
            if base.shade(shade).contrast().rgb() == 0x00_00_00 {
                ColorValue::Shade(Color::Black, shade).value()
            } else {
                ColorValue::Shade(Color::White, shade).value()
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
        assert!(css.as_str().contains("--lsx-button-font-size-md:1rem;"));
        assert!(css.as_str().contains("--lsx-button-height-md:42px;"));
        assert!(css.as_str().contains("--lsx-button-padding-x-md:18px;"));
        assert!(css.as_str().contains("--lsx-spacing-xs:4px;"));
        assert!(css.as_str().contains("--lsx-spacing-xl:20px;"));
        assert!(css.as_str().contains("--lsx-breakpoint-xs:36rem;"));
        assert!(css.as_str().contains("--lsx-breakpoint-xl:88rem;"));
        assert!(css.as_str().contains("--lsx-dialog-size-md:510px;"));
        assert!(css.as_str().contains("--lsx-drawer-size-md:280px;"));
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
        assert!(css.as_str().contains("--lsx-flex-row-wrap:nowrap;"));
        assert!(css.as_str().contains("--lsx-title-font-size-xxl:2.125rem;"));
        assert!(css.as_str().contains("--lsx-title-font-weight-xxl:400;"));
        assert!(css.as_str().contains("--lsx-title-font-size-xl:1.625rem;"));
        assert!(css.as_str().contains("--lsx-title-font-size-lg:1.375rem;"));
        assert!(css.as_str().contains("--lsx-title-font-size-md:1rem;"));
        assert!(css.as_str().contains("--lsx-title-font-size-sm:0.875rem;"));
        assert!(css.as_str().contains("--lsx-title-font-size-xs:0.75rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-xs:0.75rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-sm:0.875rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-md:1rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-lg:1.125rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-xl:1.25rem;"));
        assert!(css.as_str().contains("--lsx-text-font-size-xxl:1.375rem;"));
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
