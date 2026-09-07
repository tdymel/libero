use crate::CssLayer;
use crate::css::{CssDeclaration, CssScope, Stylesheet, ToCssDeclarations};

use super::{
    Color, ColorShade, ColorValue, HexColor, INDICATOR_KEYFRAMES, LOADER_KEYFRAMES,
    MARQUEE_KEYFRAMES, NOTIFICATION_KEYFRAMES, NamedColorCss, PROGRESS_BAR_KEYFRAMES,
    RIPPLE_KEYFRAMES, SKELETON_KEYFRAMES, Size, SizeCss, TEXT_FONT_FAMILY, TEXT_FONT_SIZE,
    TEXT_FONT_WEIGHT, TEXT_LETTER_SPACING, TEXT_LINE_HEIGHT, Theme,
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
    /// The vars and the keyframes stay unlayered - a custom property and an
    /// `@keyframes` name are not cascaded rules, so a layer would only make
    /// them harder to override. The reset and the `body` rules go into
    /// `lsx-base`, the first layer: unlayered they would beat *every* layered
    /// rule, so an app whose own base styles sit in a layer (Tailwind v4's
    /// `@layer base`, any `@layer reset`) could not restyle `body` without
    /// `!important`.
    fn from(theme: &Theme) -> Self {
        let mut css = Stylesheet::new(vec![CssScope::new(":root", theme_declarations(theme))])
            .as_str()
            .to_string();

        let mut base_scopes = global_reset_scopes(theme);
        base_scopes.push(body_scope(theme));
        css.push_str(&format!(
            "@layer {}{{{}}}",
            CssLayer::Base.css_name(),
            Stylesheet::new(base_scopes).as_str()
        ));

        css.push_str(RIPPLE_KEYFRAMES);
        css.push_str(PROGRESS_BAR_KEYFRAMES);
        css.push_str(LOADER_KEYFRAMES);
        css.push_str(INDICATOR_KEYFRAMES);
        css.push_str(SKELETON_KEYFRAMES);
        css.push_str(MARQUEE_KEYFRAMES);
        css.push_str(NOTIFICATION_KEYFRAMES);
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
        carousel,
        scroller,
        center,
        container,
        aspect_ratio,
        collapse,
        accordion,
        float,
        overlay,
        z_index,
        popover,
        progress_bar,
        focus_ring,
        paper,
        divider,
        splitter,
        blockquote,
        burger,
        button,
        chip,
        badge,
        alert,
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
        timeline,
        tabs,
        stepper,
        title,
        text,
        tooltip,
        code,
        code_block,
        header,
        icon,
        action_icon,
        qr_code,
        image,
        image_list,
        lightbox,
        avatar,
        avatar_group,
        kbd,
        menu,
        menubar,
        pagination,
        pagination_labels: _,
        anchor,
        file_field,
        pin_field,
        color_picker,
        color_swatch,
        date_picker,
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
        floating_window: _,
        phone_field: _,
        tags_field: _,
        text_field: _,
        textarea: _,
        number_field: _,
        date: _,
        date_field: _,
        time_picker: _,
        native_select: _,
        select: _,
        multi_select: _,
        cascader: _,
        autocomplete: _,
        segmented_control: _,
        password_field: _,
        color_field: _,
        scroll_area: _,
        tree: _,
        mark: _,
        hover_card: _,
        nav_link: _,
        loader,
        indicator,
        skeleton,
        spotlight,
        marquee,
        notifications,
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
    declarations.extend(carousel.to_css_declarations());
    declarations.extend(scroller.to_css_declarations());
    declarations.extend(center.to_css_declarations());
    declarations.extend(container.to_css_declarations());
    declarations.extend(aspect_ratio.to_css_declarations());
    declarations.extend(collapse.to_css_declarations());
    declarations.extend(accordion.to_css_declarations());
    declarations.extend(float.to_css_declarations());
    declarations.extend(overlay.to_css_declarations());
    declarations.extend(z_index.to_css_declarations());
    declarations.extend(popover.to_css_declarations());
    declarations.extend(progress_bar.to_css_declarations());
    declarations.extend(focus_ring.to_css_declarations());
    declarations.extend(paper.to_css_declarations());
    declarations.extend(divider.to_css_declarations());
    declarations.extend(splitter.to_css_declarations());
    declarations.extend(blockquote.to_css_declarations());
    declarations.extend(burger.to_css_declarations());
    declarations.extend(button.to_css_declarations());
    declarations.extend(chip.to_css_declarations());
    declarations.extend(badge.to_css_declarations());
    declarations.extend(alert.to_css_declarations());
    declarations.extend(loader.to_css_declarations());
    declarations.extend(indicator.to_css_declarations());
    declarations.extend(skeleton.to_css_declarations());
    declarations.extend(spotlight.to_css_declarations());
    declarations.extend(marquee.to_css_declarations());
    declarations.extend(notifications.to_css_declarations());
    declarations.extend(lightbox.to_css_declarations());
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
    declarations.extend(date_picker.to_css_declarations());
    declarations.extend(combobox.to_css_declarations());
    declarations.extend(slider.to_css_declarations());
    declarations.extend(list.to_css_declarations());
    declarations.extend(data_list.to_css_declarations());
    declarations.extend(table.to_css_declarations());
    declarations.extend(timeline.to_css_declarations());
    declarations.extend(tabs.to_css_declarations());
    declarations.extend(stepper.to_css_declarations());
    declarations.extend(title.to_css_declarations());
    declarations.extend(text.to_css_declarations());
    declarations.extend(tooltip.to_css_declarations());
    declarations.extend(code.to_css_declarations());
    declarations.extend(code_block.to_css_declarations());
    declarations.extend(header.to_css_declarations());
    declarations.extend(icon.to_css_declarations());
    declarations.extend(action_icon.to_css_declarations());
    declarations.extend(qr_code.to_css_declarations());
    declarations.extend(kbd.to_css_declarations());
    declarations.extend(menu.to_css_declarations());
    declarations.extend(menubar.to_css_declarations());
    declarations.extend(pagination.to_css_declarations());
    declarations.extend(image.to_css_declarations());
    declarations.extend(image_list.to_css_declarations());
    declarations.extend(avatar.to_css_declarations());
    declarations.extend(avatar_group.to_css_declarations());
    declarations.extend(anchor.to_css_declarations());
    push_named_color_declarations(&mut declarations, *black, *white);
    // Dimmed text is the grey ramp's own text role, so a re-themed `grey`
    // carries it - see `NamedColorCss::TEXT_DIMMED`.
    declarations.push(CssDeclaration::new(
        NamedColorCss::TEXT_DIMMED.name(),
        ColorValue::Text(Color::Grey, ColorShade::DEFAULT).value(),
    ));
    push_color_declarations(&mut declarations, Color::Primary, *primary, *white);
    push_color_declarations(&mut declarations, Color::Secondary, *secondary, *white);
    push_color_declarations(&mut declarations, Color::Error, *error, *white);
    push_color_declarations(&mut declarations, Color::Warning, *warning, *white);
    push_color_declarations(&mut declarations, Color::Info, *info, *white);
    push_color_declarations(&mut declarations, Color::Success, *success, *white);
    push_color_declarations(&mut declarations, Color::Neutral, *neutral, *white);
    push_color_declarations(&mut declarations, Color::Grey, *grey, *white);
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

/// Three ramps per palette colour, plus the foreground that pairs with the
/// fill ramp.
///
/// `Shade` is the brand colour, untouched - `primary` is `blue.6` wherever a
/// border, a ring or a decoration asks for it. The other two are the same
/// ramp **re-based** on an accessible step rather than a lookup that snaps
/// each step to the first passing one: re-basing keeps the ramp a ramp, so a
/// filled control's `:hover` (one step darker) is still visibly darker than
/// its rest state. Snapping collapses `fill-6` and `fill-7` onto the same
/// colour and the hover disappears.
///
/// - `text-N` walks from `base.shade(text_shade(N))`, the first step that
///   clears 4.5:1 on `surface`.
/// - `fill-N` walks from `base.shade(fill_shade(N))`, the first step whose
///   auto-contrast foreground clears 4.5:1 on it.
/// - `contrast-N` is that foreground, computed on `fill-N` - the two are
///   always used as a pair.
///
/// `surface` is the theme's paper. A dark surface would want the ramp walked
/// the other way; nothing in the library has one yet (todo 69).
fn push_color_declarations(
    declarations: &mut Vec<CssDeclaration>,
    color: Color,
    base: HexColor,
    surface: HexColor,
) {
    let ramp = color.shade_ramp();
    let text_base = base.shade(base.text_shade(ramp, ColorShade::DEFAULT, surface), ramp);
    let fill_base = base.shade(base.fill_shade(ramp, ColorShade::DEFAULT), ramp);

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Shade(color, shade).var_name(),
            base.shade(shade, ramp).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Text(color, shade).var_name(),
            text_base.shade(shade, ramp).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Fill(color, shade).var_name(),
            fill_base.shade(shade, ramp).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Contrast(color, shade).var_name(),
            if fill_base.shade(shade, ramp).readable_contrast().rgb() == 0x00_00_00 {
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
    use crate::tokens::TEXT_CONTRAST;

    /// Every palette name, so the check below cannot be defeated by a colour
    /// nobody thought of.
    const PALETTE: &[&str] = &[
        "primary",
        "secondary",
        "error",
        "warning",
        "info",
        "success",
        "neutral",
        "grey",
    ];

    /// A `*Defaults` that declares a colour as `"primary.6"` rather than
    /// resolving it ships a bare palette token into `:root`. That is not a CSS
    /// colour, and because a custom property accepts any tokens the var is
    /// *set* - so no `var()` fallback fires, and any shorthand reading it is
    /// invalid at computed-value time and is dropped whole.
    ///
    /// `Timeline` shipped exactly this: its rail was invisible under the
    /// default theme, with every test green, because the tests asserted the
    /// `Sx` expressions and nothing looked at what `:root` resolved them to.
    ///
    /// This is the cheap general guard - one test covering every component,
    /// including the ones not written yet.
    /// Reads the var out of the rendered `:root`, so these assert on what
    /// ships rather than on the helper that built it.
    fn root_var(css: &str, name: &str) -> String {
        let root = css
            .split_once(":root{")
            .and_then(|(_, rest)| rest.split_once('}'))
            .map(|(block, _)| block)
            .expect("a :root block");

        root.split(';')
            .find_map(|declaration| declaration.strip_prefix(&format!("{name}:")))
            .unwrap_or_else(|| panic!("{name} is declared"))
            .to_string()
    }

    /// The numbers the Maintainer approved on 2026-09-19 (todo 239): the
    /// brand colour stays `blue.6`, text on paper lands on `primary.8` at
    /// 4.86:1, and the fill lands there too, because white on `blue.6` is
    /// 3.56:1.
    #[test]
    fn primary_keeps_its_brand_shade_and_moves_only_in_its_two_roles() {
        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();
        let white = HexColor::new(0xFF_FF_FF);

        assert_eq!(root_var(&css, "--lsx-primary-6"), "#228BE6");
        assert_eq!(root_var(&css, "--lsx-primary-text-6"), "#1C74C1");
        assert_eq!(root_var(&css, "--lsx-primary-fill-6"), "#1C74C1");

        let text = HexColor::parse(&root_var(&css, "--lsx-primary-text-6")).expect("a hex");
        assert!(
            text.contrast_ratio(white) >= TEXT_CONTRAST,
            "primary as text measured {:.2}:1 on paper",
            text.contrast_ratio(white)
        );
    }

    /// Snapping each step to the first passing one would give `fill-6` and
    /// `fill-7` the same colour, and every filled control would lose its
    /// hover. Re-basing the ramp keeps the steps apart.
    #[test]
    fn the_fill_ramp_is_still_a_ramp() {
        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();

        assert_ne!(
            root_var(&css, "--lsx-primary-fill-6"),
            root_var(&css, "--lsx-primary-fill-7")
        );
        assert_ne!(
            root_var(&css, "--lsx-primary-fill-7"),
            root_var(&css, "--lsx-primary-fill-8")
        );
    }

    /// Every palette colour, so a re-themed one cannot quietly ship a fill
    /// nobody can label. `warning` and `success` are the two the ramp cannot
    /// rescue as *text* - see the test below.
    #[test]
    fn every_default_fill_carries_its_own_foreground() {
        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();

        for name in PALETTE {
            let fill =
                HexColor::parse(&root_var(&css, &format!("--lsx-{name}-fill-6"))).expect("a hex");
            let foreground = match root_var(&css, &format!("--lsx-{name}-contrast-6")).as_str() {
                "var(--lsx-black)" => HexColor::new(0x00_00_00),
                "var(--lsx-white)" => HexColor::new(0xFF_FF_FF),
                other => panic!("unexpected contrast value {other}"),
            };

            let ratio = fill.contrast_ratio(foreground);
            assert!(
                ratio >= TEXT_CONTRAST,
                "{name} fill {fill} labelled at {ratio:.2}:1"
            );
        }
    }

    /// `warning` and `success` are yellow and green: no step of a 25%-black
    /// mix reaches 4.5:1 on white, so the text role takes the darkest step it
    /// has and still falls short. Recorded rather than asserted away - the
    /// only fix is a different base colour, which is the Maintainer's call.
    #[test]
    fn the_text_role_is_the_best_the_ramp_can_do_and_two_colors_fall_short() {
        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();
        let white = HexColor::new(0xFF_FF_FF);
        let short: Vec<&str> = PALETTE
            .iter()
            .filter(|name| {
                let text = HexColor::parse(&root_var(&css, &format!("--lsx-{name}-text-6")))
                    .expect("a hex");
                text.contrast_ratio(white) < TEXT_CONTRAST
            })
            .copied()
            .collect();

        assert_eq!(short, ["warning", "success"]);
    }

    /// Todo 240: the one name for quieter text, `grey.7` at 8.12:1, not the
    /// `grey.6` the library used to set real text in.
    #[test]
    fn dimmed_text_is_the_grey_ramps_text_role() {
        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();
        let white = HexColor::new(0xFF_FF_FF);

        assert_eq!(
            root_var(&css, NamedColorCss::TEXT_DIMMED.name()),
            "var(--lsx-grey-text-6)"
        );
        assert_eq!(root_var(&css, "--lsx-grey-text-6"), "#4C5055");
        assert_eq!(root_var(&css, "--lsx-grey-7"), "#4C5055");

        let dimmed = HexColor::parse(&root_var(&css, "--lsx-grey-text-6")).expect("a hex");
        assert!(dimmed.contrast_ratio(white) >= TEXT_CONTRAST);
    }

    #[test]
    fn no_root_declaration_ships_a_bare_palette_token() {
        let css = Stylesheet::from(&Theme::DEFAULT);
        let root = css
            .as_str()
            .split_once(":root{")
            .and_then(|(_, rest)| rest.split_once('}'))
            .map(|(block, _)| block)
            .expect("a :root block");

        for declaration in root.split(';').filter(|part| !part.is_empty()) {
            let Some((name, value)) = declaration.split_once(':') else {
                continue;
            };
            let value = value.trim();
            for colour in PALETTE {
                // `primary.6` - a palette token. `var(--lsx-primary-6)` is the
                // resolved form and starts with `var(`, so it never matches.
                assert!(
                    !value.starts_with(colour) || !value[colour.len()..].starts_with('.'),
                    "{name} declares the bare palette token {value:?}; \
                     use ColorValue and declare its .value() instead"
                );
            }
        }
    }

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
            ("--lsx-carousel-gap", "var(--lsx-spacing-md)"),
            ("--lsx-carousel-per-view", "1"),
            ("--lsx-carousel-control-size", "28px"),
            ("--lsx-paper-radius", "var(--lsx-radius-md)"),
            ("--lsx-paper-shadow", "var(--lsx-shadow-sm)"),
            ("--lsx-paper-background", "#fff"),
            ("--lsx-paper-contrast", "var(--lsx-black)"),
            ("--lsx-paper-border-color", "var(--lsx-grey-3)"),
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

    /// The two-tone ring's whole premise: the pair reads against itself, so
    /// it owes 3:1 between its own tones rather than against a surface the
    /// caller owns (todo 368).
    #[test]
    fn the_focus_ring_carries_its_own_contrast() {
        assert_declares(&[
            ("--lsx-focus-ring-halo", "#fff"),
            ("--lsx-focus-ring-width", "2px"),
            ("--lsx-focus-ring-offset", "2px"),
            ("--lsx-focus-ring-halo-width", "2px"),
            (
                "--lsx-focus-ring-halo-spread",
                "calc(var(--lsx-focus-ring-offset) + var(--lsx-focus-ring-width) + \
                 var(--lsx-focus-ring-halo-width))",
            ),
        ]);

        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();
        // The stripe is a var pointing at a palette shade; follow the one hop.
        let stripe = root_var(&css, "--lsx-focus-ring-color");
        let stripe = stripe
            .strip_prefix("var(")
            .and_then(|rest| rest.strip_suffix(')'))
            .expect("the stripe names a palette var");
        let stripe = HexColor::parse(&root_var(&css, stripe)).expect("a hex");
        let halo = HexColor::parse("#fff").expect("a hex");

        assert!(
            stripe.contrast_ratio(halo) >= 3.0,
            "the ring's two tones measure {:.2}:1 against each other",
            stripe.contrast_ratio(halo)
        );
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

    /// The vars and the keyframes stay unlayered, the reset and `body` do
    /// not: unlayered they outrank every layered rule an app writes.
    #[test]
    fn theme_css_layers_the_reset_and_body_but_not_the_vars() {
        let css = Stylesheet::from(&Theme::DEFAULT);
        let css = css.as_str();

        let layer = css
            .split_once("@layer lsx-base{")
            .expect("the theme sheet opens an lsx-base layer")
            .1;
        let layer = &layer[..layer.find("}}").expect("the layer block closes") + 2];

        assert!(layer.contains("html{box-sizing:border-box;"));
        assert!(layer.contains("*, *::before, *::after{box-sizing:inherit;}"));
        assert!(layer.contains("body{margin:0;"));
        assert!(!layer.contains(":root{"));
        assert!(!layer.contains("@keyframes"));

        assert!(css.starts_with(":root{"));
        assert!(css.contains(RIPPLE_KEYFRAMES));
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
