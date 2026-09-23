use crate::CssLayer;
use crate::css::{CssDeclaration, CssScope, Stylesheet, ToCssDeclarations};
use crate::tokens::{Ends, HOVER_TINT_SHADE, SELECTED_TINT_SHADE, ShadeRamp, TEXT_CONTRAST};

use super::{
    ANCHOR_CODE_COLOR, CODE_BLOCK_BACKGROUND, CODE_BLOCK_COPY_HOVER_BACKGROUND,
    CODE_BLOCK_COPY_HOVER_TEXT, CODE_BLOCK_LINE_NUMBER, CODE_BLOCK_MUTED_TEXT, CODE_TOK_ATTRIBUTE,
    CODE_TOK_COMMENT, CODE_TOK_CONSTANT, CODE_TOK_FUNCTION, CODE_TOK_HEADING, CODE_TOK_KEYWORD,
    CODE_TOK_NUMBER, CODE_TOK_STRING, CODE_TOK_TAG, CODE_TOK_TYPE, Color, ColorCss, ColorShade,
    ColorValue, CssVar, HexColor, INDICATOR_KEYFRAMES, KBD_BACKGROUND, KBD_COLOR, LOADER_KEYFRAMES,
    MARQUEE_KEYFRAMES, NOTIFICATION_KEYFRAMES, NamedColorCss, PAPER_BACKGROUND,
    PROGRESS_BAR_KEYFRAMES, RIPPLE_KEYFRAMES, SCROLL_AREA_KEYFRAMES, SKELETON_KEYFRAMES,
    SORTABLE_KEYFRAMES, Size, SizeCss, TEXT_FONT_FAMILY, TEXT_FONT_SIZE, TEXT_FONT_WEIGHT,
    TEXT_LETTER_SPACING, TEXT_LINE_HEIGHT, TOOLTIP_KEYFRAMES, Theme, ThemeSet,
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
    /// One theme's sheet: its vars at `:root`, then reset, `body` and keyframes.
    /// `color-scheme` follows this theme's surface.
    fn from(theme: &Theme) -> Self {
        let mut css = root_block(":root", theme);
        css.push_str(&base_layer_and_keyframes(
            theme,
            theme.surface.color_scheme(),
        ));
        Stylesheet::from(css)
    }
}

impl From<&ThemeSet> for Stylesheet {
    /// The pair up front, so switching is one root attribute: no re-render, right
    /// on first paint, `prefers-color-scheme` without JS.
    ///
    /// ```css
    /// :root { … }                            /* the light theme */
    /// @media (prefers-color-scheme: dark) {
    ///   :root:not([data-lsx-theme]) { … }    /* the dark theme, system case */
    /// }
    /// :root[data-lsx-theme="light"] { … }
    /// :root[data-lsx-theme="dark"]  { … }
    /// ```
    ///
    /// Without a dark theme, one theme's sheet. [`ThemeSet::named`] themes are not in
    /// here: selecting one rebuilds the sheet.
    fn from(set: &ThemeSet) -> Self {
        let light = set.light_theme();
        let Some(dark) = set.dark_theme() else {
            return Stylesheet::from(light);
        };

        let mut css = root_block(":root", light);
        // The system case; `:not(..)` lets an explicit choice win, as `@media` adds no specificity.
        css.push_str(&format!(
            "@media {DARK_SCHEME_QUERY}{{{}}}",
            root_block(&format!(":root:not([{THEME_ATTRIBUTE}])"), dark)
        ));
        css.push_str(&root_block(&theme_selector(ThemeSet::LIGHT), light));
        css.push_str(&root_block(&theme_selector(ThemeSet::DARK), dark));

        // From the light theme: its colours are vars, and `font_smoothing` is not a scheme choice.
        css.push_str(&base_layer_and_keyframes(light, "light dark"));
        Stylesheet::from(css)
    }
}

/// The attribute a document root carries to pin one theme of the pair.
pub(crate) const THEME_ATTRIBUTE: &str = "data-lsx-theme";

/// The dark half's media query, which the web backend also reads the platform scheme from.
pub(crate) const DARK_SCHEME_QUERY: &str = "(prefers-color-scheme: dark)";

fn theme_selector(name: &str) -> String {
    format!(":root[{THEME_ATTRIBUTE}=\"{name}\"]")
}

fn root_block(selector: &str, theme: &Theme) -> String {
    Stylesheet::new(vec![CssScope::new(selector, theme_declarations(theme))])
        .as_str()
        .to_string()
}

/// Everything but the vars, emitted once per sheet. Reset and `body` go in `lsx-base`, or
/// they'd beat an app's layered base styles (Tailwind v4); vars and keyframes stay unlayered.
fn base_layer_and_keyframes(theme: &Theme, color_scheme: &str) -> String {
    let mut base_scopes = global_reset_scopes(theme, color_scheme);
    base_scopes.push(body_scope(theme));

    let mut css = format!(
        "@layer {}{{{}}}",
        CssLayer::Base.css_name(),
        Stylesheet::new(base_scopes).as_str()
    );

    css.push_str(RIPPLE_KEYFRAMES);
    css.push_str(PROGRESS_BAR_KEYFRAMES);
    css.push_str(LOADER_KEYFRAMES);
    css.push_str(INDICATOR_KEYFRAMES);
    css.push_str(SKELETON_KEYFRAMES);
    css.push_str(MARQUEE_KEYFRAMES);
    css.push_str(NOTIFICATION_KEYFRAMES);
    css.push_str(TOOLTIP_KEYFRAMES);
    css.push_str(SCROLL_AREA_KEYFRAMES);
    css.push_str(SORTABLE_KEYFRAMES);
    css
}

fn global_reset_scopes(theme: &Theme, color_scheme: &str) -> Vec<CssScope> {
    let mut html_declarations = vec![
        CssDeclaration::new("box-sizing", "border-box"),
        // Canvas, scrollbars and native controls follow the theme; a pair passes `light dark`.
        CssDeclaration::new("color-scheme", color_scheme),
    ];
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

/// A physical `text-align` per `dir`, in the base layer, for a renderer that
/// aligns the initial `start` left under `rtl` (Blitz).
pub(crate) fn physical_text_align() -> String {
    let scopes = [("rtl", "right"), ("ltr", "left")]
        .into_iter()
        .map(|(dir, side)| {
            CssScope::new(
                format!(":where([dir={dir}])"),
                vec![CssDeclaration::new("text-align", side)],
            )
        })
        .collect();
    format!(
        "@layer {}{{{}}}",
        CssLayer::Base.css_name(),
        Stylesheet::new(scopes).as_str()
    )
}

/// A field's paper and border for raw form controls, in the base layer, for a
/// renderer that paints them white whatever `color-scheme` says (Blitz).
pub(crate) fn themed_form_controls() -> String {
    // Toggles and file/button inputs keep the UA look; text stays inherited.
    let controls = ":where(input:not([type=checkbox], [type=radio], [type=range], \
        [type=color], [type=file], [type=image], [type=submit], [type=reset], \
        [type=button]), textarea, select)";
    let scope = CssScope::new(
        controls,
        vec![
            CssDeclaration::new("background-color", PAPER_BACKGROUND.value()),
            CssDeclaration::new("border-color", ColorCss::MUTED.value(ColorShade::S6)),
        ],
    );
    format!(
        "@layer {}{{{}}}",
        CssLayer::Base.css_name(),
        Stylesheet::new(vec![scope]).as_str()
    )
}

fn body_scope(theme: &Theme) -> CssScope {
    // Ink and surface have no contrast var, so computed as `push_color_declarations` does.
    let text_color_var = foreground_var(theme.surface.contrast(), theme_ends(theme));

    CssScope::new(
        "body",
        vec![
            CssDeclaration::new("margin", "0"),
            CssDeclaration::new("background-color", NamedColorCss::SURFACE.value()),
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
    // Exhaustive (no `..`): a new `Theme` field won't compile until it's handled.
    let Theme {
        spacing,
        radius,
        elevation,
        font_size,
        // Measured against the palette, so declared with it below.
        gradient: _,
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
        transition,
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
        tree,
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
        // Chrome only - read by the component, never a var.
        theme_toggle: _,
        repo_button: _,
        direction_toggle: _,
        tldr: _,
        anchor,
        file_field,
        pin_field,
        color_picker,
        color_swatch,
        chrono_picker,
        primary,
        secondary,
        error,
        warning,
        info,
        success,
        neutral,
        muted,
        ink,
        surface,
        // Plain values read from Rust - no CSS vars of their own.
        floating_window: _,
        phone_field: _,
        tags_field: _,
        text_field: _,
        textarea: _,
        number_field: _,
        chrono_field: _,
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
    declarations.extend(font_size.to_css_declarations(SizeCss::FONT_SIZE, ""));
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
    declarations.extend(transition.to_css_declarations());
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
    declarations.extend(chrono_picker.to_css_declarations());
    declarations.extend(combobox.to_css_declarations());
    declarations.extend(slider.to_css_declarations());
    declarations.extend(list.to_css_declarations());
    declarations.extend(data_list.to_css_declarations());
    declarations.extend(table.to_css_declarations());
    declarations.extend(timeline.to_css_declarations());
    declarations.extend(tree.to_css_declarations());
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
    let ends = Ends {
        surface: *surface,
        ink: *ink,
    };

    push_named_color_declarations(&mut declarations, ends);
    // The muted ramp's text role, so a re-themed `muted` carries it.
    declarations.push(CssDeclaration::new(
        NamedColorCss::TEXT_DIMMED.name(),
        ColorValue::Text(Color::Muted, ColorShade::DEFAULT).value(),
    ));
    // A `Paper` that is not a hex (a `var()`, a gradient) cannot be measured.
    let cards: Vec<HexColor> = HexColor::parse(paper.background).into_iter().collect();
    for (color, base) in [
        (Color::Primary, *primary),
        (Color::Secondary, *secondary),
        (Color::Error, *error),
        (Color::Warning, *warning),
        (Color::Info, *info),
        (Color::Success, *success),
        (Color::Neutral, *neutral),
        (Color::Muted, *muted),
    ] {
        push_color_declarations(&mut declarations, color, base, ends, &cards);
    }
    declarations.extend(super::gradient_theme_declarations(theme));
    rebase_text_on_derived_surfaces(&mut declarations, ends);
    declarations
}

/// Code and `Kbd` text sits on `muted` steps that follow the page, the token hues don't:
/// each text var that falls short takes the smallest readable mix (todo 396).
fn rebase_text_on_derived_surfaces(declarations: &mut [CssDeclaration], ends: Ends) {
    let block = CODE_BLOCK_BACKGROUND.name().to_string();
    // Inline `Code`'s `muted.2` background, which `sx` resolves as a fill.
    let inline = ColorValue::Fill(Color::Muted, ColorShade::S2).var_name();
    let code = [block.clone(), inline];
    let kbd = [KBD_BACKGROUND.name().to_string()];
    let hover = [CODE_BLOCK_COPY_HOVER_BACKGROUND.name().to_string()];

    let texts: [(CssVar, &[String], f32); 15] = [
        (ANCHOR_CODE_COLOR, &code[1..], TEXT_CONTRAST),
        (CODE_TOK_KEYWORD, &code, TEXT_CONTRAST),
        (CODE_TOK_STRING, &code, TEXT_CONTRAST),
        (CODE_TOK_COMMENT, &code, TEXT_CONTRAST),
        (CODE_TOK_NUMBER, &code, TEXT_CONTRAST),
        (CODE_TOK_CONSTANT, &code, TEXT_CONTRAST),
        (CODE_TOK_FUNCTION, &code, TEXT_CONTRAST),
        (CODE_TOK_TYPE, &code, TEXT_CONTRAST),
        (CODE_TOK_TAG, &code, TEXT_CONTRAST),
        (CODE_TOK_ATTRIBUTE, &code, TEXT_CONTRAST),
        (CODE_TOK_HEADING, &code, TEXT_CONTRAST),
        (CODE_BLOCK_MUTED_TEXT, &code[..1], TEXT_CONTRAST),
        // 241's approved 4.27:1; the 3:1 floor let palettes drift below it (899).
        (CODE_BLOCK_LINE_NUMBER, &code[..1], 4.27),
        (KBD_COLOR, &kbd, TEXT_CONTRAST),
        // The copy button's icon, a graphic: 1.4.11.
        (CODE_BLOCK_COPY_HOVER_TEXT, &hover, 3.0),
    ];
    for (text, surfaces, floor) in texts {
        rebase_text(declarations, text.name(), surfaces, floor, ends);
    }
}

/// Leaves `name` alone when it or a surface can't be measured (a keyword, a translucent `rgba()`).
fn rebase_text(
    declarations: &mut [CssDeclaration],
    name: &str,
    surfaces: &[String],
    floor: f32,
    ends: Ends,
) {
    let resolve = |name: &str| {
        let mut value = name.to_string();
        for _ in 0..declarations.len() {
            let declared = declarations.iter().find(|d| d.property() == value)?.value();
            match CssVar::parse_value(declared) {
                Some(next) => value = next.to_string(),
                None => return HexColor::parse(declared),
            }
        }
        None
    };
    let Some(text) = resolve(name) else { return };
    let Some(surfaces) = surfaces
        .iter()
        .map(|surface| resolve(surface))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let readable = text.readable_on(&surfaces, floor, ends);
    // Unchanged, so a var that already reads stays a var.
    if readable == text {
        return;
    }
    if let Some(declaration) = declarations.iter_mut().find(|d| d.property() == name) {
        *declaration = CssDeclaration::new(name, readable.to_string());
    }
}

// Literals, not a themed scale: `@media` can't read custom properties.
fn push_breakpoint_declarations(declarations: &mut Vec<CssDeclaration>) {
    for size in super::Size::ALL {
        declarations.push(SizeCss::BREAKPOINT.declare(size, size.breakpoint_value()));
    }
}

fn push_named_color_declarations(declarations: &mut Vec<CssDeclaration>, ends: Ends) {
    declarations.push(CssDeclaration::new(
        NamedColorCss::INK.name(),
        ends.ink.to_string(),
    ));
    declarations.push(CssDeclaration::new(
        NamedColorCss::SURFACE.name(),
        ends.surface.to_string(),
    ));
}

fn theme_ends(theme: &Theme) -> Ends {
    Ends {
        surface: theme.surface,
        ink: theme.ink,
    }
}

/// The var a black or white `foreground` is spelled as: on a dark theme white is `--lsx-ink`.
fn foreground_var(foreground: HexColor, ends: Ends) -> String {
    if ends.foreground_is_ink(foreground) {
        NamedColorCss::INK.value()
    } else {
        NamedColorCss::SURFACE.value()
    }
}

/// The brand ramp, plus `text-N` and `fill-N` re-based on [`HexColor::text_base`] and
/// [`HexColor::fill_base`], and `contrast-N` paired with `fill-N` (todos 69, 314).
/// Re-based, not snapped per step: snapping merged `fill-6` and `fill-7` and lost the hover.
fn push_color_declarations(
    declarations: &mut Vec<CssDeclaration>,
    color: Color,
    base: HexColor,
    ends: Ends,
    cards: &[HexColor],
) {
    let ramp = color.shade_ramp();
    let text_base = base.text_base(ramp, ends, cards);
    let fill_base = base.fill_base(ramp, ends);

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Shade(color, shade).var_name(),
            base.shade(shade, ramp, ends).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Text(color, shade).var_name(),
            text_base.shade(shade, ramp, ends).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Fill(color, shade).var_name(),
            fill_base.shade(shade, ramp, ends).to_string(),
        ));
    }

    for shade in SHADES {
        declarations.push(CssDeclaration::new(
            ColorValue::Contrast(color, shade).var_name(),
            foreground_var(
                fill_base.shade(shade, ramp, ends).readable_contrast(ends),
                ends,
            ),
        ));
    }

    push_on_tint_declarations(declarations, color, base, text_base, fill_base, ends);
}

/// An unfilled control's label on its hover and selected tints, mixed until it reads on
/// both. Declared only where the resting label fails; `on_tint_color` falls back to it.
fn push_on_tint_declarations(
    declarations: &mut Vec<CssDeclaration>,
    color: Color,
    base: HexColor,
    text_base: HexColor,
    fill_base: HexColor,
    ends: Ends,
) {
    let ramp = color.shade_ramp();
    // The neutral ramps label in the brand shade, as `ColorValue::as_text` says.
    let label_base = match ramp {
        ShadeRamp::Chromatic => text_base,
        ShadeRamp::Neutral => base,
    };
    let tints = [
        fill_base.shade(HOVER_TINT_SHADE, ramp, ends),
        fill_base.shade(SELECTED_TINT_SHADE, ramp, ends),
    ];

    for shade in SHADES {
        let label = label_base.shade(shade, ramp, ends);
        let on_tint = label.readable_on(&tints, TEXT_CONTRAST, ends);
        if on_tint == label {
            continue;
        }
        if let Some(name) = ColorValue::Shade(color, shade).on_tint_name() {
            declarations.push(CssDeclaration::new(name, on_tint.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tokens::TEXT_CONTRAST;

    /// Every palette name, so no colour slips past the checks below.
    pub(super) const PALETTE: &[&str] = &[
        "primary",
        "secondary",
        "error",
        "warning",
        "info",
        "success",
        "neutral",
        "muted",
    ];

    /// Reads the var out of the rendered `:root`, so tests assert on what ships.
    pub(super) fn root_var(css: &str, name: &str) -> String {
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

    /// Todo 239: the brand stays `blue.6`; text and fill move by the smallest readable
    /// mix, `#1D78C8` at 4.59:1, not a whole step to `primary.8`.
    #[test]
    fn primary_keeps_its_brand_shade_and_moves_only_in_its_two_roles() {
        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();
        let white = HexColor::new(0xFF_FF_FF);

        assert_eq!(root_var(&css, "--lsx-primary-6"), "#228BE6");
        assert_eq!(root_var(&css, "--lsx-primary-text-6"), "#1D78C8");
        assert_eq!(root_var(&css, "--lsx-primary-fill-6"), "#1D78C8");

        let text = HexColor::parse(&root_var(&css, "--lsx-primary-text-6")).expect("a hex");
        assert!(
            text.contrast_ratio(white) >= TEXT_CONTRAST,
            "primary as text measured {:.2}:1 on paper",
            text.contrast_ratio(white)
        );
    }

    /// Snapping steps would merge `fill-6` and `fill-7`, losing every filled hover.
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

    /// No palette colour may ship a fill nobody can label.
    #[test]
    fn every_default_fill_carries_its_own_foreground() {
        // Against the painted foreground: a dark theme's ink is not pure white.
        for theme in [&Theme::DEFAULT, &Theme::DARK] {
            let css = Stylesheet::from(theme).as_str().to_string();

            for name in PALETTE {
                let fill = HexColor::parse(&root_var(&css, &format!("--lsx-{name}-fill-6")))
                    .expect("a hex");
                let foreground = match root_var(&css, &format!("--lsx-{name}-contrast-6")).as_str()
                {
                    "var(--lsx-ink)" => theme.ink,
                    "var(--lsx-surface)" => theme.surface,
                    other => panic!("unexpected contrast value {other}"),
                };

                let ratio = fill.contrast_ratio(foreground);
                assert!(
                    ratio >= TEXT_CONTRAST,
                    "{name} fill {fill} labelled at {ratio:.2}:1"
                );
            }
        }
    }

    /// Yellow and green reach no 4.5:1 on white within the ramp. Recorded: the only fix
    /// is a different base colour, the Maintainer's call.
    #[test]
    fn the_text_role_is_the_best_the_ramp_can_do_and_two_colors_fall_short() {
        assert_eq!(
            text_roles_falling_short(&Theme::DEFAULT),
            ["warning", "success"]
        );
    }

    /// On a dark page yellow and green have room to walk.
    #[test]
    fn every_text_role_reads_on_the_dark_theme_s_page() {
        assert_eq!(text_roles_falling_short(&Theme::DARK), Vec::<&str>::new());
    }

    /// Every shipped set: ink must read on page and card. Short text roles and pale
    /// `muted.6` (todo 394) are recorded, not asserted: ported palettes are others' designs.
    ///
    /// Every set keeps Libero's amber `warning`; the long entries are light halves keeping
    /// dark-page accents (Nord's own guidance, and the derived Vague, Osmium, kettek16).
    #[test]
    fn every_shipped_set_reads_on_its_own_page() {
        let mut short = Vec::new();
        let mut quiet = Vec::new();

        for set in ThemeSet::CATALOGUE {
            for theme in [Some(set.light_theme()), set.dark_theme()]
                .into_iter()
                .flatten()
            {
                let paper = HexColor::parse(theme.paper.background).expect("a hex");
                let scheme = theme.surface.color_scheme();

                for (what, against) in [("page", theme.surface), ("card", paper)] {
                    let ratio = theme.ink.contrast_ratio(against);
                    assert!(
                        ratio >= TEXT_CONTRAST,
                        "{} {scheme}: ink on its own {what} is {ratio:.2}:1",
                        set.name()
                    );
                }

                let falling = text_roles_falling_short(theme);
                if !falling.is_empty() {
                    short.push(format!("{} {scheme}: {}", set.name(), falling.join(" ")));
                }

                let ratio = muted_on_its_page(theme);
                if ratio < TEXT_CONTRAST {
                    // Floored, so a hair short never prints as a pass.
                    let ratio = (ratio * 100.0).floor() / 100.0;
                    quiet.push(format!("{} {scheme}: {ratio:.2}:1", set.name()));
                }
            }
        }

        assert_eq!(
            short,
            [
                "Libero light: warning success",
                "Ayu light: warning info success",
                "Ayu Mirage light: warning info success",
                "Catppuccin light: warning",
                "Dracula light: warning",
                "Ef Night light: warning",
                "Everforest light: warning",
                "Flexoki light: warning",
                "GitHub light: warning",
                "Gruvbox light: warning",
                "Gruvbox Classic light: warning",
                "Gruvbox Soft light: warning",
                "Kanagawa light: warning",
                "Kanagawa Dragon light: warning",
                "kettek16 light: primary warning info",
                "Nord light: primary secondary warning info success",
                "One light: warning",
                "Osmium light: primary secondary warning info success",
                "Rosé Pine light: warning",
                "shadcn/ui light: warning success",
                "Vague light: secondary warning info success",
            ],
            "the shipped palettes' text roles moved"
        );
        assert_eq!(
            quiet,
            [
                "Libero light: 3.32:1",
                "Ayu light: 3.05:1",
                "Ayu dark: 3.52:1",
                "Ayu Mirage light: 3.05:1",
                "Ayu Mirage dark: 3.58:1",
                "Catppuccin light: 4.36:1",
                "Dracula dark: 3.02:1",
                "Ef Night light: 4.49:1",
                "Everforest light: 3.08:1",
                "Everforest dark: 4.24:1",
                "Gruvbox Classic light: 4.28:1",
                "Kanagawa light: 2.93:1",
                "Kanagawa dark: 3.33:1",
                "Kanagawa Dragon light: 2.93:1",
                "One dark: 3.72:1",
                "Vague light: 2.98:1",
                "Vague dark: 3.02:1",
            ],
            "the shipped palettes' muted.6 moved"
        );
    }

    /// shadcn's own `--border`, `--input` and `--ring` sit at 1.3:1 to 2.5:1 on white, so
    /// they are not taken: functional edges are `muted.6` and the ring is ink and surface.
    #[test]
    fn the_shadcn_pair_reads_and_keeps_its_edges_and_ring_visible() {
        use crate::theme::{SHADCN_DARK, SHADCN_LIGHT};

        for theme in [&SHADCN_LIGHT, &SHADCN_DARK] {
            let css = Stylesheet::from(theme).as_str().to_string();
            let paper = HexColor::parse(theme.paper.background).expect("a hex");
            let var = |name: &str| HexColor::parse(&root_var(&css, name)).expect("a hex");

            for (what, against) in [("page", theme.surface), ("card", paper)] {
                for (name, floor) in [
                    ("--lsx-ink", TEXT_CONTRAST),
                    ("--lsx-muted-text-6", TEXT_CONTRAST),
                    ("--lsx-muted-6", 3.0),
                ] {
                    let ratio = var(name).contrast_ratio(against);
                    assert!(
                        ratio >= floor,
                        "{name} on the {} {what} is {ratio:.2}:1",
                        theme.surface.color_scheme()
                    );
                }
            }

            assert!(theme.ink.contrast_ratio(theme.surface) >= 3.0, "focus ring");
        }
    }

    /// `muted.6` on `theme`'s surface; never re-based, so it stays as pale as drawn.
    fn muted_on_its_page(theme: &Theme) -> f32 {
        let css = Stylesheet::from(theme).as_str().to_string();
        HexColor::parse(&root_var(&css, "--lsx-muted-6"))
            .expect("a hex")
            .contrast_ratio(theme.surface)
    }

    /// Palette colours whose text role misses [`TEXT_CONTRAST`] on `theme`'s surface.
    fn text_roles_falling_short(theme: &Theme) -> Vec<&'static str> {
        let css = Stylesheet::from(theme).as_str().to_string();
        PALETTE
            .iter()
            .filter(|name| {
                let text = HexColor::parse(&root_var(&css, &format!("--lsx-{name}-text-6")))
                    .expect("a hex");
                text.contrast_ratio(theme.surface) < TEXT_CONTRAST
            })
            .copied()
            .collect()
    }

    /// Todo 240: dimmed text is the muted text role, `#70777E` at 4.54:1: quiet but passing.
    #[test]
    fn dimmed_text_is_the_grey_ramps_text_role() {
        let css = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();
        let white = HexColor::new(0xFF_FF_FF);

        assert_eq!(
            root_var(&css, NamedColorCss::TEXT_DIMMED.name()),
            "var(--lsx-muted-text-6)"
        );
        assert_eq!(root_var(&css, "--lsx-muted-text-6"), "#70777E");

        let dimmed = HexColor::parse(&root_var(&css, "--lsx-muted-text-6")).expect("a hex");
        assert!(dimmed.contrast_ratio(white) >= TEXT_CONTRAST);
    }

    /// The neutral ramp is distance from the page: `muted.1` (every hover) sits on the
    /// surface, `muted.9` against the ink. Mixed towards white, it lit up dark pages.
    #[test]
    fn the_neutral_ramp_is_mixed_between_the_theme_s_own_ends() {
        let mut theme = Theme::DEFAULT;
        theme.surface = HexColor::new(0x1A_1B_1E);
        theme.ink = HexColor::new(0xE9_EC_EF);

        let dark = Stylesheet::from(&theme).as_str().to_string();
        let light = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();

        let step = |css: &str, shade: u8| {
            HexColor::parse(&root_var(css, &format!("--lsx-muted-{shade}"))).expect("a hex")
        };

        // Both ends, both schemes: S1 sits on the page, S9 against the ink.
        for (css, ends) in [
            (&dark, theme_ends(&theme)),
            (&light, theme_ends(&Theme::DEFAULT)),
        ] {
            assert!(
                step(css, 1).contrast_ratio(ends.surface) < 1.2,
                "muted.1 does not sit on its own surface"
            );
            assert!(
                step(css, 9).contrast_ratio(ends.surface) > 8.0,
                "muted.9 is not up against the ink"
            );
        }

        // The base is shade 6 in both, untouched by the mixing.
        assert_eq!(step(&dark, 6), Theme::DEFAULT.muted);
        assert_eq!(step(&light, 6), Theme::DEFAULT.muted);

        // The defect this closes: the light theme's `muted.1` is a bright patch here.
        assert!(
            step(&light, 1).contrast_ratio(theme.surface) > 10.0,
            "the light theme's hover background was not a bright patch here"
        );
    }

    /// Todo 69(b): roles derive against the theme's own surface, so a dark one walks
    /// its ramps the other way.
    #[test]
    fn a_dark_surface_derives_its_roles_the_other_way() {
        let mut theme = Theme::DEFAULT;
        theme.surface = HexColor::new(0x1A_1B_1E);
        theme.ink = HexColor::new(0xE9_EC_EF);

        let dark = Stylesheet::from(&theme).as_str().to_string();
        let light = Stylesheet::from(&Theme::DEFAULT).as_str().to_string();

        for name in PALETTE {
            let text =
                HexColor::parse(&root_var(&dark, &format!("--lsx-{name}-text-6"))).expect("a hex");
            let base =
                HexColor::parse(&root_var(&dark, &format!("--lsx-{name}-6"))).expect("a hex");

            assert!(
                text.relative_luminance() >= base.relative_luminance(),
                "{name} walked its text ramp darker on a dark surface"
            );
        }

        // The light theme's answer is unreadable here: the bug this closes.
        let here = HexColor::parse(&root_var(&dark, "--lsx-primary-text-6")).expect("a hex");
        let there = HexColor::parse(&root_var(&light, "--lsx-primary-text-6")).expect("a hex");
        assert!(here.contrast_ratio(theme.surface) >= TEXT_CONTRAST);
        assert!(there.contrast_ratio(theme.surface) < TEXT_CONTRAST);

        // Both label a fill in the page colour: same role, opposite end of the greyscale.
        assert_eq!(
            root_var(&dark, "--lsx-primary-contrast-6"),
            "var(--lsx-surface)"
        );
        assert_eq!(
            root_var(&light, "--lsx-primary-contrast-6"),
            "var(--lsx-surface)"
        );
        let dark_fill = HexColor::parse(&root_var(&dark, "--lsx-primary-fill-6")).expect("a hex");
        let light_fill = HexColor::parse(&root_var(&light, "--lsx-primary-fill-6")).expect("a hex");
        assert!(
            dark_fill.relative_luminance() > light_fill.relative_luminance(),
            "the dark theme's fill did not walk away from its own page"
        );
        // 1.4.11 asks 3:1 of a control against what is behind it.
        assert!(dark_fill.contrast_ratio(theme.surface) >= 3.0);

        assert!(dark.contains("background-color:var(--lsx-surface);"));
        assert!(dark.contains("color:var(--lsx-ink);"));
    }

    /// A bare `primary.6` is not CSS; Timeline once shipped one. Declare `ColorValue::value()`.
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
                // The resolved `var(--lsx-primary-6)` never matches.
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
            ("--lsx-font-size-xs", "0.75rem"),
            ("--lsx-font-size-md", "1rem"),
            ("--lsx-font-size-xxl", "1.375rem"),
            (
                "--lsx-glass-background",
                "color-mix(in srgb, var(--lsx-paper-background) 80%, transparent)",
            ),
            ("--lsx-glass-blur", "blur(12px)"),
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
            ("--lsx-paper-contrast", "var(--lsx-ink)"),
            ("--lsx-paper-border-color", "var(--lsx-muted-3)"),
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
            ("--lsx-flex-row-wrap", "wrap"),
        ]);
    }

    /// Todo 368: the two-tone ring owes 3:1 between its own tones, not against the surface.
    #[test]
    fn the_focus_ring_carries_its_own_contrast() {
        assert_declares(&[
            // The theme's two page ends, so a scheme redefines the ring with them.
            ("--lsx-focus-ring-color", "var(--lsx-ink)"),
            ("--lsx-focus-ring-halo", "var(--lsx-surface)"),
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
        // Each tone is a var naming another var; follow the one hop.
        let tone = |name| {
            let named = root_var(&css, name);
            let named = named
                .strip_prefix("var(")
                .and_then(|rest| rest.strip_suffix(')'))
                .expect("a tone names a theme var");
            HexColor::parse(&root_var(&css, named)).expect("a hex")
        };
        let (stripe, halo) = (
            tone("--lsx-focus-ring-color"),
            tone("--lsx-focus-ring-halo"),
        );

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
            // Text reads the global scale.
            ("--lsx-text-font-size-xs", "var(--lsx-font-size-xs)"),
            ("--lsx-text-font-size-md", "var(--lsx-font-size-md)"),
            ("--lsx-text-font-size-xxl", "var(--lsx-font-size-xxl)"),
            ("--lsx-text-font-weight-xs", "400"),
            ("--lsx-text-line-height-md", "1.5"),
        ]);
    }

    #[test]
    fn theme_css_declares_the_palette_and_its_contrasts() {
        assert_declares(&[
            ("--lsx-ink", "#000000"),
            ("--lsx-surface", "#FFFFFF"),
            ("--lsx-secondary-6", "#7950F2"),
            ("--lsx-error-6", "#FA5252"),
            ("--lsx-warning-6", "#FAB005"),
            ("--lsx-info-6", "#15AABF"),
            ("--lsx-success-6", "#40C057"),
            ("--lsx-muted-6", "#868E96"),
            ("--lsx-primary-contrast-1", "var(--lsx-ink)"),
            ("--lsx-primary-contrast-6", "var(--lsx-surface)"),
            ("--lsx-muted-contrast-6", "var(--lsx-ink)"),
        ]);
    }

    /// Locks both ramps so a tweak to one can't silently reshape the other.
    #[test]
    fn theme_css_palette_ramps_are_pinned() {
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
            ("--lsx-muted-1", "#F2F3F4"),
            ("--lsx-muted-3", "#E0E2E4"),
            ("--lsx-muted-6", "#868E96"),
            ("--lsx-muted-9", "#212325"),
        ] {
            assert!(css.contains(&format!("{var}:{hex};")), "{var} != {hex}");
        }
    }

    #[test]
    fn theme_css_sets_body_and_global_box_sizing_defaults() {
        let theme = &Theme::DEFAULT;
        let css = Stylesheet::from(theme);
        let css = css.as_str();

        assert!(css.contains("html{box-sizing:border-box;color-scheme:light;"));
        assert!(css.contains("-webkit-font-smoothing:antialiased;"));
        assert!(css.contains("-moz-osx-font-smoothing:grayscale;"));
        assert!(css.contains("*, *::before, *::after{box-sizing:inherit;}"));

        assert!(css.contains("body{margin:0;"));
        assert!(css.contains("background-color:var(--lsx-surface);"));
        // The default paper has high luminance, so its foreground is the ink.
        assert!(css.contains("color:var(--lsx-ink);"));
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

        assert!(layer.contains("html{box-sizing:border-box;color-scheme:light;"));
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

        assert!(css.contains("html{box-sizing:border-box;color-scheme:light;}"));
        assert!(!css.contains("font-smoothing"));
    }
}
