use super::palette::theme_ends;
use crate::{
    css::Stylesheet,
    theme::{HexColor, NamedColorCss, RIPPLE_KEYFRAMES, Theme, ThemeSet},
    tokens::TEXT_CONTRAST,
};

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
            let fill =
                HexColor::parse(&root_var(&css, &format!("--lsx-{name}-fill-6"))).expect("a hex");
            let foreground = match root_var(&css, &format!("--lsx-{name}-contrast-6")).as_str() {
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

/// Every shipped set: ink must read on page and card, `muted.6` reach 3:1. Short text
/// roles and `muted.6` under 4.5:1 (todo 394) are recorded: ported palettes are others' designs.
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
            // Todo 1308: borders and handles are non-text, so 3:1 is a floor (WCAG 1.4.11).
            assert!(
                ratio >= 3.0,
                "{} {scheme}: muted.6 on its own page is {ratio:.2}:1",
                set.name()
            );
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
            "Kanagawa light: 3.00:1",
            "Kanagawa dark: 3.33:1",
            "Kanagawa Dragon light: 3.00:1",
            "One dark: 3.72:1",
            "Vague light: 3.02:1",
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
            let text =
                HexColor::parse(&root_var(&css, &format!("--lsx-{name}-text-6"))).expect("a hex");
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
        let base = HexColor::parse(&root_var(&dark, &format!("--lsx-{name}-6"))).expect("a hex");

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
