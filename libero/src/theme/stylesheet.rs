use crate::css::{CssDeclaration, CssScope, Stylesheet, ToCssDeclarations};

use super::{
    Color, ColorShade, ColorValue, HexColor, NamedColorCss, RIPPLE_KEYFRAMES, Size, SizeCss,
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
        css.push_str(RIPPLE_KEYFRAMES);
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
    // Black/white have no per-shade contrast var, so it's computed here the
    // way `push_color_declarations` does for palette shades.
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
    // Exhaustive (no `..`) on purpose: a new `Theme` field won't compile
    // until it's declared or explicitly ignored, so vars can't go unemitted.
    let Theme {
        spacing,
        radius,
        elevation,
        dialog,
        drawer,
        sidebar,
        flex,
        grid,
        center,
        container,
        aspect_ratio,
        float,
        overlay,
        z_index,
        popover,
        divider,
        splitter,
        button,
        chip,
        switch,
        checkbox,
        radio,
        field,
        form,
        fieldset,
        combobox,
        slider,
        list,
        data_list,
        table,
        tabs,
        titles,
        texts,
        tooltip,
        code,
        code_block,
        header,
        icon,
        action_icon,
        qr_code,
        image,
        kbd,
        anchor,
        file_field,
        pin_field,
        color_picker,
        color_swatch,
        primary,
        secondary,
        error,
        warning,
        info,
        success,
        neutral,
        grey,
        black,
        white,
        // Plain values read from Rust - no CSS vars of their own.
        text_field: _,
        textarea: _,
        number_field: _,
        native_select: _,
        select: _,
        multi_select: _,
        autocomplete: _,
        color_field: _,
        scroll_area: _,
        tree: _,
        mark: _,
        nav_link: _,
        // Emitted by global_reset_scopes, not as a `:root` var.
        font_smoothing: _,
    } = theme;

    let mut declarations = Vec::new();
    declarations.extend(spacing.to_css_declarations(SizeCss::SPACING, "px"));
    declarations.extend(radius.to_css_declarations(SizeCss::RADIUS, "px"));
    declarations.extend(elevation.to_css_declarations(SizeCss::SHADOW, ""));
    push_breakpoint_declarations(&mut declarations);
    declarations.extend(dialog.to_css_declarations());
    declarations.extend(drawer.to_css_declarations());
    declarations.extend(sidebar.to_css_declarations());
    declarations.extend(flex.to_css_declarations());
    declarations.extend(grid.to_css_declarations());
    declarations.extend(center.to_css_declarations());
    declarations.extend(container.to_css_declarations());
    declarations.extend(aspect_ratio.to_css_declarations());
    declarations.extend(float.to_css_declarations());
    declarations.extend(overlay.to_css_declarations());
    declarations.extend(z_index.to_css_declarations());
    declarations.extend(popover.to_css_declarations());
    declarations.extend(divider.to_css_declarations());
    declarations.extend(splitter.to_css_declarations());
    declarations.extend(button.to_css_declarations());
    declarations.extend(chip.to_css_declarations());
    declarations.extend(switch.to_css_declarations());
    declarations.extend(checkbox.to_css_declarations());
    declarations.extend(radio.to_css_declarations());
    declarations.extend(field.to_css_declarations());
    declarations.extend(form.to_css_declarations());
    declarations.extend(fieldset.to_css_declarations());
    declarations.extend(file_field.to_css_declarations());
    declarations.extend(pin_field.to_css_declarations());
    declarations.extend(color_picker.to_css_declarations());
    declarations.extend(color_swatch.to_css_declarations());
    declarations.extend(combobox.to_css_declarations());
    declarations.extend(slider.to_css_declarations());
    declarations.extend(list.to_css_declarations());
    declarations.extend(data_list.to_css_declarations());
    declarations.extend(table.to_css_declarations());
    declarations.extend(tabs.to_css_declarations());
    declarations.extend(titles.to_css_declarations());
    declarations.extend(texts.to_css_declarations());
    declarations.extend(tooltip.to_css_declarations());
    declarations.extend(code.to_css_declarations());
    declarations.extend(code_block.to_css_declarations());
    declarations.extend(header.to_css_declarations());
    declarations.extend(icon.to_css_declarations());
    declarations.extend(action_icon.to_css_declarations());
    declarations.extend(qr_code.to_css_declarations());
    declarations.extend(kbd.to_css_declarations());
    declarations.extend(image.to_css_declarations());
    declarations.extend(anchor.to_css_declarations());
    push_named_color_declarations(&mut declarations, *black, *white);
    push_color_declarations(&mut declarations, Color::Primary, *primary);
    push_color_declarations(&mut declarations, Color::Secondary, *secondary);
    push_color_declarations(&mut declarations, Color::Error, *error);
    push_color_declarations(&mut declarations, Color::Warning, *warning);
    push_color_declarations(&mut declarations, Color::Info, *info);
    push_color_declarations(&mut declarations, Color::Success, *success);
    push_color_declarations(&mut declarations, Color::Neutral, *neutral);
    push_color_declarations(&mut declarations, Color::Grey, *grey);
    declarations
}

// `@media` can't reference custom properties, and breakpoints go straight
// into query strings, so they stay literals rather than a themed scale.
fn push_breakpoint_declarations(declarations: &mut Vec<CssDeclaration>) {
    for size in super::Size::ALL {
        declarations.push(SizeCss::BREAKPOINT.declare(size, size.breakpoint_value()));
    }
}

fn push_named_color_declarations(
    declarations: &mut Vec<CssDeclaration>,
    black: HexColor,
    white: HexColor,
) {
    declarations.push(CssDeclaration::new(
        NamedColorCss::BLACK.name(),
        black.to_string(),
    ));
    declarations.push(CssDeclaration::new(
        NamedColorCss::WHITE.name(),
        white.to_string(),
    ));
}

fn push_color_declarations(declarations: &mut Vec<CssDeclaration>, color: Color, base: HexColor) {
    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Shade(color, shade).var_name(),
            base.shade(shade, color.shade_ramp()).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Contrast(color, shade).var_name(),
            if base.shade(shade, color.shade_ramp()).contrast().rgb() == 0x00_00_00 {
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

    fn assert_declares(pairs: &[(&str, &str)]) {
        let css = Stylesheet::from(&Theme::DEFAULT);
        let css = css.as_str();

        for (var, value) in pairs {
            assert!(
                css.contains(&format!("{var}:{value};")),
                "missing {var}:{value}"
            );
        }
    }

    #[test]
    fn theme_css_is_one_root_block() {
        let css = Stylesheet::from(&Theme::DEFAULT);

        assert!(css.as_str().starts_with(":root{"));
        assert!(css.as_str().ends_with("}"));
        assert!(css.as_str().contains(RIPPLE_KEYFRAMES));
    }

    #[test]
    fn theme_css_declares_the_global_scales() {
        assert_declares(&[
            ("--lsx-spacing-xs", "4px"),
            ("--lsx-spacing-xl", "20px"),
            ("--lsx-breakpoint-xs", "36rem"),
            ("--lsx-breakpoint-xl", "88rem"),
        ]);
    }

    #[test]
    fn theme_css_declares_component_defaults() {
        assert_declares(&[
            ("--lsx-switch-track-width-md", "42px"),
            ("--lsx-switch-track-height-md", "22px"),
            ("--lsx-switch-thumb-size-md", "18px"),
            ("--lsx-button-font-size-md", "1rem"),
            ("--lsx-button-height-md", "42px"),
            ("--lsx-button-padding-x-md", "18px"),
            ("--lsx-dialog-size-md", "510px"),
            ("--lsx-drawer-size-md", "280px"),
            ("--lsx-sidebar-size-md", "280px"),
            ("--lsx-container-size", "var(--lsx-breakpoint-lg)"),
            ("--lsx-container-gutters", "var(--lsx-spacing-md)"),
            ("--lsx-divider-spacing", "0"),
        ]);
    }

    #[test]
    fn theme_css_declares_both_flex_axes() {
        assert_declares(&[
            ("--lsx-flex-column-align", "stretch"),
            ("--lsx-flex-column-justify", "flex-start"),
            ("--lsx-flex-column-spacing", "var(--lsx-spacing-md)"),
            ("--lsx-flex-column-wrap", "nowrap"),
            ("--lsx-flex-row-align", "center"),
            ("--lsx-flex-row-justify", "flex-start"),
            ("--lsx-flex-row-spacing", "var(--lsx-spacing-md)"),
            ("--lsx-flex-row-wrap", "nowrap"),
        ]);
    }

    #[test]
    fn theme_css_declares_the_typography_scales() {
        assert_declares(&[
            ("--lsx-title-font-size-xxl", "2.125rem"),
            ("--lsx-title-font-size-xl", "1.625rem"),
            ("--lsx-title-font-size-lg", "1.375rem"),
            ("--lsx-title-font-size-md", "1rem"),
            ("--lsx-title-font-size-sm", "0.875rem"),
            ("--lsx-title-font-size-xs", "0.75rem"),
            ("--lsx-title-font-weight-xxl", "400"),
            ("--lsx-text-font-size-xs", "0.75rem"),
            ("--lsx-text-font-size-sm", "0.875rem"),
            ("--lsx-text-font-size-md", "1rem"),
            ("--lsx-text-font-size-lg", "1.125rem"),
            ("--lsx-text-font-size-xl", "1.25rem"),
            ("--lsx-text-font-size-xxl", "1.375rem"),
            ("--lsx-text-font-weight-xs", "400"),
            ("--lsx-text-line-height-md", "1.5"),
        ]);
    }

    #[test]
    fn theme_css_declares_the_palette_and_its_contrasts() {
        assert_declares(&[
            ("--lsx-black", "#000000"),
            ("--lsx-white", "#FFFFFF"),
            ("--lsx-secondary-6", "#7950F2"),
            ("--lsx-error-6", "#FA5252"),
            ("--lsx-warning-6", "#FAB005"),
            ("--lsx-info-6", "#15AABF"),
            ("--lsx-success-6", "#40C057"),
            ("--lsx-grey-6", "#868E96"),
            ("--lsx-primary-contrast-1", "var(--lsx-black)"),
            ("--lsx-primary-contrast-6", "var(--lsx-white)"),
            ("--lsx-grey-contrast-6", "var(--lsx-black)"),
        ]);
    }

    /// Both ramps are fitted to Mantine's palettes; this locks them so a
    /// tweak to one can't silently reshape the other.
    #[test]
    fn theme_css_palette_ramps_track_mantine() {
        let css = Stylesheet::from(&Theme::DEFAULT);
        let css = css.as_str();

        for (var, hex) in [
            ("--lsx-primary-1", "#D2E7FA"),
            ("--lsx-primary-3", "#7CBAF0"),
            ("--lsx-primary-6", "#228BE6"),
            ("--lsx-primary-9", "#1968AC"),
        ] {
            assert!(css.contains(&format!("{var}:{hex};")), "{var} != {hex}");
        }

        for (var, hex) in [
            ("--lsx-grey-1", "#F2F3F4"),
            ("--lsx-grey-3", "#E0E2E4"),
            ("--lsx-grey-6", "#868E96"),
            ("--lsx-grey-9", "#212325"),
        ] {
            assert!(css.contains(&format!("{var}:{hex};")), "{var} != {hex}");
        }
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
