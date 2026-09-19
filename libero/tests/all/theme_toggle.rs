//! `ThemeToggle`'s rendered contract: the glyph and the name say where a
//! press goes, the name comes from the localization, and `themes` adds a
//! picker beside the toggle.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::ThemeToggle,
    hooks::use_color_scheme,
    localization::{Localization, ThemeToggleLabels},
    theme::{ColorSchemeSetting, ThemeSet},
};

/// The three glyphs, told apart by a path only each one draws.
const SYSTEM: &str = "M12 3a9 9";
const SUN: &str = "<circle cx=\"12\" cy=\"12\" r=\"4\"";
const MOON: &str = "M20 14.5A8.5";

thread_local! {
    static PRESSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Presses the button `PRESSES` times, after the first render, the way a
/// reader would.
#[component]
fn Press() -> Element {
    let scheme = use_color_scheme();
    use_effect(move || {
        for _ in 0..PRESSES.get() {
            scheme.cycle();
        }
    });
    rsx! {}
}

fn pressed(times: usize) -> String {
    fn app() -> Element {
        rsx! { LiberoProvider { ThemeToggle {} Press {} } }
    }
    PRESSES.set(times);
    body(&render(app))
}

/// The cycle, with a platform that reads light: following it, then dark
/// (the scheme it is not showing), then light pinned, then following it
/// again. The glyph and the name both say where a press goes.
#[test]
fn it_steps_from_the_system_to_the_other_scheme_and_back() {
    let steps = [
        (MOON, "Switch to the dark theme"),
        (SUN, "Switch to the light theme"),
        (SYSTEM, "Follow the system theme"),
        (MOON, "Switch to the dark theme"),
    ];

    for (presses, (glyph, name)) in steps.into_iter().enumerate() {
        let html = pressed(presses);
        let button = attributes_of(&html, "button");
        assert_eq!(button["aria-label"], name, "after {presses}: {html}");
        assert!(html.contains(glyph), "after {presses}: {html}");
        assert!(button["data-state"].contains("outlined"), "{html}");
    }
}

/// Without `themes` it is one button, not a group of one.
#[test]
fn without_themes_it_is_the_toggle_alone() {
    let html = pressed(0);
    assert_eq!(html.matches("<button").count(), 1, "{html}");
    assert!(!html.contains(r#"role="group""#), "{html}");
}

/// With `themes` it is a named group of two buttons: the toggle, and a
/// chevron that announces the menu it opens.
#[test]
fn themes_add_a_picker_beside_the_toggle() {
    fn app() -> Element {
        rsx! { LiberoProvider { ThemeToggle { themes: ThemeSet::CATALOGUE } } }
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
    theme_toggle: ThemeToggleLabels {
        to_light: "Helles Design",
        to_dark: "Dunkles Design",
        ..ThemeToggleLabels::ENGLISH
    },
    ..Localization::ENGLISH
};

/// The two names are the localization's, so a translation is one struct.
#[test]
fn the_name_comes_from_the_localization() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &GERMAN,
                ThemeToggle {}
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
                ThemeToggle {
                    label: move |next: ColorSchemeSetting| format!("to {next:?}"),
                }
            }
        }
    }

    let html = body(&render(app));

    assert_eq!(attributes_of(&html, "button")["aria-label"], "to Dark");
}
