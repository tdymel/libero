use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoContext, LiberoProvider,
    components::Text,
    theme::{HexColor, Theme, ThemeSet},
};

/// Two themes that differ in one colour and nothing else. Deliberately not a
/// dark theme: this measures the switching mechanism, not a palette.
static LIGHT: Theme = Theme {
    primary: HexColor::new(0x22_8B_E6),
    ..Theme::DEFAULT
};
static DARK: Theme = Theme {
    primary: HexColor::new(0xE6_8B_22),
    ..Theme::DEFAULT
};

/// A theme of the caller's own, and not the library's default by value:
/// `LiberoProvider` pairs `Theme::DEFAULT` with `Theme::DARK`, and it decides
/// that by comparing the theme it was handed.
static SOLO: Theme = Theme {
    primary: HexColor::new(0x22_E6_8B),
    ..Theme::DEFAULT
};

fn set() -> ThemeSet {
    ThemeSet::new().light(&LIGHT).dark(&DARK)
}

/// The pair is in the sheet up front: the light theme at `:root`, the dark one
/// behind the media query for the system case, and one attribute block each so
/// an explicit choice wins. That is what makes a switch cost no rebuild.
#[test]
fn a_pair_emits_both_themes_and_the_attribute_blocks() {
    fn app() -> Element {
        rsx! { LiberoProvider { themes: set(), Text { "hello" } } }
    }

    let html = render(app);

    assert!(html.contains(":root{"));
    assert!(html.contains("@media (prefers-color-scheme: dark){:root:not([data-lsx-theme]){"));
    assert!(html.contains(r#":root[data-lsx-theme="light"]{"#));
    assert!(html.contains(r#":root[data-lsx-theme="dark"]{"#));
    assert!(html.contains("color-scheme:light dark;"));

    // Both palettes ship, or there would be nothing for the attribute to
    // select between.
    assert!(html.contains("--lsx-primary-6:#228BE6;"));
    assert!(html.contains("--lsx-primary-6:#E68B22;"));
}

/// The `theme:` prop is sugar for a one-theme set, so every call site that
/// predates `ThemeSet` keeps its old sheet exactly - nothing to switch to
/// means none of the blocks above are emitted.
#[test]
fn one_theme_still_emits_one_root_block() {
    fn app() -> Element {
        rsx! { LiberoProvider { theme: &SOLO, Text { "hello" } } }
    }

    let html = render(app);

    assert!(!html.contains("prefers-color-scheme: dark"));
    // The selector, not the name: the provider's restore script names the
    // attribute too, and it is emitted whatever the set holds.
    assert!(!html.contains("[data-lsx-theme"));
    assert!(html.contains("color-scheme:light;"));
    assert_eq!(html.matches("--lsx-primary-6:").count(), 1);
}

/// An app that names no theme at all still gets the library's own pair, so
/// `prefers-color-scheme` switches it with no JS and no theme authoring.
#[test]
fn the_default_provider_ships_the_library_s_pair() {
    fn app() -> Element {
        rsx! { LiberoProvider { Text { "hello" } } }
    }

    let html = render(app);

    assert!(html.contains("@media (prefers-color-scheme: dark){:root:not([data-lsx-theme]){"));
    assert!(html.contains("color-scheme:light dark;"));
    // The dark theme's own page, which the light one never emits.
    assert!(html.contains("--lsx-surface:#1A1B1E;"));
}

/// `use_theme()` is reactive: a component that read the old theme re-renders
/// with the new one. And it is only a re-render - the `use_hook` below runs
/// once per mount, so a second line would mean the switch tore the subtree
/// down and rebuilt it.
#[test]
fn a_switch_re_renders_the_readers_without_re_mounting_them() {
    #[component]
    fn Reader() -> Element {
        let theme = libero::hooks::use_theme();
        let mounts = use_hook(|| {
            static COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
        });

        rsx! { Text { "primary={theme.primary} mounts={mounts}" } }
    }

    #[component]
    fn Switcher() -> Element {
        let context = use_context::<LiberoContext>();
        use_effect(move || context.set_active_theme(ThemeSet::DARK));

        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                themes: set(),
                Reader {}
                Switcher {}
            }
        }
    }

    let html = body(&render(app));

    assert!(
        html.contains(&format!("primary={}", DARK.primary)),
        "the reader kept the old theme: {html}"
    );
    assert!(html.contains("mounts=1"), "the reader re-mounted: {html}");
}

/// A name the set does not have changes nothing, rather than blanking every
/// colour the app draws with.
///
/// "Nothing" is asserted as **the pair's own sheet, untouched** - both
/// palettes and all four blocks. Asserting only that the light palette is
/// still there would be a check that cannot fail: the pair is emitted up
/// front, so `--lsx-primary-6:#228BE6` is in the sheet whatever the switch
/// does. Measured: with `set_active_theme` mutated to rebuild the sheet on an
/// unknown name, the weaker assertion stayed green and this one goes red.
#[test]
fn an_unknown_theme_name_is_ignored() {
    #[component]
    fn Switcher() -> Element {
        let context = use_context::<LiberoContext>();
        use_effect(move || context.set_active_theme("sepia"));

        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { themes: set(), Switcher {} }
        }
    }

    let switched = render(app);

    fn untouched() -> Element {
        rsx! {
            LiberoProvider { themes: set(), Text { "hello" } }
        }
    }

    // The sheet a set with no switch at all emits. Byte-for-byte the same one,
    // or the unknown name changed something.
    let sheet = |html: &str| {
        let start = html.find(":root{").expect("a root block");
        let end = html[start..].find("</style>").expect("the sheet closes") + start;
        html[start..end].to_string()
    };

    assert_eq!(sheet(&switched), sheet(&render(untouched)));
    assert!(switched.contains(r#":root[data-lsx-theme="dark"]{"#));
    assert!(switched.contains("--lsx-primary-6:#228BE6;"));
    assert!(switched.contains("--lsx-primary-6:#E68B22;"));
}

/// A theme beyond the pair is not in the emitted sheet, so selecting it
/// rebuilds one - the accepted cost of keeping the sheet from growing with
/// every theme an app owns.
#[test]
fn a_named_theme_beyond_the_pair_rebuilds_the_sheet() {
    static SEPIA: Theme = Theme {
        primary: HexColor::new(0x70_4214),
        ..Theme::DEFAULT
    };

    #[component]
    fn Switcher() -> Element {
        let context = use_context::<LiberoContext>();
        use_effect(move || context.set_active_theme("sepia"));

        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                themes: set().named("sepia", &SEPIA),
                Switcher {}
            }
        }
    }

    let html = render(app);

    assert!(html.contains("--lsx-primary-6:#704214;"));
    // The rebuilt sheet is one theme's, so the pair's blocks are gone with
    // it. The selector, not the name - the restore script names it too.
    assert!(!html.contains("[data-lsx-theme"));
}
