//! `ThemeSwitcher`'s rendered contract: the glyph and the name say where a
//! press goes, the name comes from the localization, and `themes` adds a
//! picker beside the toggle.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::ThemeSwitcher,
    hooks::use_color_scheme,
    localization::{Localization, ThemeSwitcherLabels},
    theme::{ColorSchemeSetting, ThemeSet},
};

/// The three glyphs, told apart by their whole bodies.
const SYSTEM: &str = pictogram_icons_lucide::contrast::outlined.body;
const SUN: &str = pictogram_icons_lucide::sun::outlined.body;
const MOON: &str = pictogram_icons_lucide::moon::outlined.body;

thread_local! {
    static PRESSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Presses the button `PRESSES` times, after the first render, the way the
/// toggle's own press does: `cycle` with the system entry, `toggle` without.
#[component]
fn Press(with_system: bool) -> Element {
    let scheme = use_color_scheme();
    use_effect(move || {
        for _ in 0..PRESSES.get() {
            if with_system {
                scheme.cycle();
            } else {
                scheme.toggle();
            }
        }
    });
    rsx! {}
}

fn pressed(times: usize, with_system: bool) -> String {
    fn plain() -> Element {
        rsx! { LiberoProvider { ThemeSwitcher {} Press { with_system: false } } }
    }
    fn full() -> Element {
        rsx! { LiberoProvider { ThemeSwitcher { with_system: true } Press { with_system: true } } }
    }
    PRESSES.set(times);
    body(&render(if with_system { full } else { plain }))
}

fn assert_steps(with_system: bool, steps: &[(&str, &str)]) {
    for (presses, (glyph, name)) in steps.iter().enumerate() {
        let html = pressed(presses, with_system);
        let button = attributes_of(&html, "button");
        assert_eq!(button["aria-label"], *name, "after {presses}: {html}");
        assert!(html.contains(glyph), "after {presses}: {html}");
        assert!(button["data-state"].contains("outlined"), "{html}");
    }
}

/// The cycle with `with_system`, on a platform that reads light: following
/// it, then dark (the scheme it is not showing), then light pinned, then
/// following it again. The glyph and the name both say where a press goes.
#[test]
fn it_steps_from_the_system_to_the_other_scheme_and_back() {
    assert_steps(
        true,
        &[
            (MOON, "Switch to the dark theme"),
            (SUN, "Switch to the light theme"),
            (SYSTEM, "Follow the system theme"),
            (MOON, "Switch to the dark theme"),
        ],
    );
}

/// By default the system is only the starting state: the toggle flips
/// between the two schemes and never offers to follow the system.
#[test]
fn without_the_system_entry_it_flips_between_the_two_schemes() {
    assert_steps(
        false,
        &[
            (MOON, "Switch to the dark theme"),
            (SUN, "Switch to the light theme"),
            (MOON, "Switch to the dark theme"),
            (SUN, "Switch to the light theme"),
        ],
    );
}

/// Without `themes` it is one button, not a group of one.
#[test]
fn without_themes_it_is_the_toggle_alone() {
    let html = pressed(0, false);
    assert_eq!(html.matches("<button").count(), 1, "{html}");
    assert!(!html.contains(r#"role="group""#), "{html}");
}

/// With `themes` it is a named group of two buttons: the toggle, and a
/// chevron that announces the menu it opens.
#[test]
fn themes_add_a_picker_beside_the_toggle() {
    fn app() -> Element {
        rsx! { LiberoProvider { ThemeSwitcher { themes: ThemeSet::CATALOGUE } } }
    }

    let html = body(&render(app));
    let group = attributes_of(&html, "div");

    assert_eq!(group["role"], "group", "{html}");
    assert_eq!(group["aria-label"], "Theme", "{html}");
    assert_eq!(html.matches("<button").count(), 2, "{html}");
    assert!(html.contains(r#"aria-label="Choose a theme""#), "{html}");
    assert!(html.contains(r#"aria-haspopup="menu""#), "{html}");
    assert!(html.contains(r#"aria-expanded="false""#), "{html}");
}

static GERMAN: Localization = Localization {
    theme_switcher: ThemeSwitcherLabels {
        to_light: "Helles Design",
        to_dark: "Dunkles Design",
        ..ThemeSwitcherLabels::ENGLISH
    },
    ..Localization::ENGLISH
};

/// The two names are the localization's, so a translation is one struct.
#[test]
fn the_name_comes_from_the_localization() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &GERMAN,
                ThemeSwitcher {}
            }
        }
    }

    let html = body(&render(app));

    assert_eq!(
        attributes_of(&html, "button")["aria-label"],
        "Dunkles Design"
    );
}

/// `label` replaces the localization's names, and is handed the setting a
/// press moves to.
#[test]
fn a_label_callback_replaces_the_localization_s_names() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ThemeSwitcher {
                    label: move |next: ColorSchemeSetting| format!("to {next:?}"),
                }
            }
        }
    }

    let html = body(&render(app));

    assert_eq!(attributes_of(&html, "button")["aria-label"], "to Dark");
}
