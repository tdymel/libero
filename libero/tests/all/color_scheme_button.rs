//! `ColorSchemeButton`'s rendered contract: the glyph and the name follow the
//! scheme on screen, the name is the theme's, and a press flips the scheme.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::ColorSchemeButton,
    hooks::use_color_scheme,
    theme::{ColorScheme, ColorSchemeButtonDefaults, ColorSchemeButtonLabels, Theme, ThemeSet},
};

/// The moon's only path, and the sun's disc: which glyph is drawn.
const MOON: &str = "M20 14.5A8.5";
const SUN: &str = "<circle";

/// Flips the scheme once, after the first render, the way a press would.
#[component]
fn Flip() -> Element {
    let scheme = use_color_scheme();
    use_effect(move || scheme.toggle());
    rsx! {}
}

/// Light is showing, so the button offers dark: a moon, named for what the
/// press does, in the theme's outlined chrome.
#[test]
fn in_the_light_scheme_it_offers_the_dark_one() {
    fn app() -> Element {
        rsx! { LiberoProvider { ColorSchemeButton {} } }
    }

    let html = body(&render(app));
    let button = attributes_of(&html, "button");

    assert_eq!(button["aria-label"], "Switch to the dark theme", "{html}");
    assert!(button["data-state"].contains("outlined"), "{html}");
    assert!(html.contains(MOON) && !html.contains(SUN), "{html}");
}

/// After a flip the dark scheme is showing, and the button offers light.
#[test]
fn in_the_dark_scheme_it_offers_the_light_one() {
    fn app() -> Element {
        rsx! { LiberoProvider { ColorSchemeButton {} Flip {} } }
    }

    let html = body(&render(app));

    assert_eq!(
        attributes_of(&html, "button")["aria-label"],
        "Switch to the light theme",
        "{html}"
    );
    assert!(html.contains(SUN) && !html.contains(MOON), "{html}");
}

static GERMAN: Theme = Theme {
    color_scheme_button: ColorSchemeButtonDefaults {
        labels: ColorSchemeButtonLabels {
            to_light: "Helles Design",
            to_dark: "Dunkles Design",
        },
        ..ColorSchemeButtonDefaults::DEFAULT
    },
    ..Theme::DEFAULT
};

/// The two names are the theme's, so a translation is one struct.
#[test]
fn the_name_comes_from_the_theme() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                themes: ThemeSet::pair("German", &GERMAN, &Theme::DARK),
                ColorSchemeButton {}
            }
        }
    }

    let html = body(&render(app));

    assert_eq!(
        attributes_of(&html, "button")["aria-label"],
        "Dunkles Design"
    );
}

/// `label` replaces the theme's names, and is handed the scheme on screen.
#[test]
fn a_label_callback_replaces_the_theme_s_names() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorSchemeButton {
                    label: move |showing: ColorScheme| format!("showing {showing:?}"),
                }
            }
        }
    }

    let html = body(&render(app));

    assert_eq!(
        attributes_of(&html, "button")["aria-label"],
        "showing Light"
    );
}
