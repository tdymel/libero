//! `Burger`'s rendered contract: which of the three ARIA facts appear when,
//! where the accessible name comes from, and what a caller can still override.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Burger,
    theme::{BurgerDefaults, BurgerLabels, Theme},
};

/// A translated theme, to prove the two words are the theme's and not the
/// component's. `&'static` because `LiberoProvider` takes one.
static GERMAN: Theme = Theme {
    burger: BurgerDefaults {
        labels: BurgerLabels {
            open: "Menü öffnen",
            close: "Menü schließen",
        },
        ..Theme::DEFAULT.burger
    },
    ..Theme::DEFAULT
};

/// The root is a real `<button type="button">` - `ActionIcon`'s, so the ripple
/// and the disabled handling are already right - with one empty `<span>` for
/// the glyph. The bars are that span's pseudo-elements, so there is nothing
/// else in the markup.
#[test]
fn it_renders_a_button_around_one_empty_span() {
    fn app() -> Element {
        rsx! { LiberoProvider { Burger {} } }
    }

    let html = body(&render(app));

    assert_eq!(html.matches("<button").count(), 1, "{html}");
    assert_eq!(html.matches("<span").count(), 1, "{html}");
    assert!(html.contains("></span></button>"), "{html}");
    assert_eq!(
        attributes_of(&html, "button")
            .get("type")
            .map(String::as_str),
        Some("button"),
        "{html}"
    );
}

/// `None` leaves it a plain button, so a `Burger` can open a modal - which is
/// not expanded by its trigger - without claiming otherwise. `Some(false)` is
/// still a disclosure. `Button::selected`'s rule.
#[test]
fn aria_expanded_appears_only_when_opened_is_some() {
    fn unset() -> Element {
        rsx! { LiberoProvider { Burger {} } }
    }
    fn closed() -> Element {
        rsx! { LiberoProvider { Burger { opened: false, "aria-controls": "nav" } } }
    }
    fn open() -> Element {
        rsx! { LiberoProvider { Burger { opened: true, "aria-controls": "nav" } } }
    }

    assert!(
        !attributes_of(&body(&render(unset)), "button").contains_key("aria-expanded"),
        "an unset `opened` must emit no aria-expanded at all"
    );
    assert_eq!(
        attributes_of(&body(&render(closed)), "button")
            .get("aria-expanded")
            .map(String::as_str),
        Some("false")
    );
    assert_eq!(
        attributes_of(&body(&render(open)), "button")
            .get("aria-expanded")
            .map(String::as_str),
        Some("true")
    );
}

/// The name is the theme's, not the caller's, so every burger in a project
/// announces itself the same way - and a translation replaces two strings.
#[test]
fn the_accessible_name_comes_from_the_theme_and_follows_opened() {
    fn english_closed() -> Element {
        rsx! { LiberoProvider { Burger { opened: false, "aria-controls": "nav" } } }
    }
    fn english_open() -> Element {
        rsx! { LiberoProvider { Burger { opened: true, "aria-controls": "nav" } } }
    }
    fn german_open() -> Element {
        rsx! {
            LiberoProvider { theme: &GERMAN,
                Burger { opened: true, "aria-controls": "nav" }
            }
        }
    }

    let name = |app: fn() -> Element| {
        attributes_of(&body(&render(app)), "button")
            .get("aria-label")
            .cloned()
            .expect("an aria-label")
    };

    assert_eq!(name(english_closed), "Open navigation");
    assert_eq!(name(english_open), "Close navigation");
    assert_eq!(name(german_open), "Menü schließen");
}

/// The escape hatch from `&'static str`: it runs during render, so it can read
/// a locale that is only known at runtime. It beats the theme, both ways.
#[test]
fn the_label_callback_overrides_the_theme_on_both_states() {
    fn open() -> Element {
        rsx! {
            LiberoProvider { theme: &GERMAN,
                Burger {
                    opened: true,
                    "aria-controls": "nav",
                    label: move |opened: bool| match opened {
                        true => "fermer".to_string(),
                        false => "ouvrir".to_string(),
                    },
                }
            }
        }
    }
    fn closed() -> Element {
        rsx! {
            LiberoProvider {
                Burger {
                    opened: false,
                    "aria-controls": "nav",
                    label: move |opened: bool| match opened {
                        true => "fermer".to_string(),
                        false => "ouvrir".to_string(),
                    },
                }
            }
        }
    }

    let name = |app: fn() -> Element| {
        attributes_of(&body(&render(app)), "button")
            .get("aria-label")
            .cloned()
            .expect("an aria-label")
    };

    assert_eq!(name(open), "fermer");
    assert_eq!(name(closed), "ouvrir");
}

/// Not a declared prop: it rides `GlobalAttributes` through `base_props!` and
/// the caller spreads it. Nothing sets it internally, so the caller's is the
/// only one - see [[codebase/attribute-precedence]].
#[test]
fn aria_controls_is_the_callers_and_reaches_the_button() {
    fn app() -> Element {
        rsx! { LiberoProvider { Burger { opened: true, "aria-controls": "site-nav" } } }
    }

    let html = body(&render(app));
    assert_eq!(
        attributes_of(&html, "button")
            .get("aria-controls")
            .map(String::as_str),
        Some("site-nav")
    );
    assert_eq!(html.matches("aria-controls").count(), 1, "{html}");
}

/// The glyph, not the button, carries the state - the bars and their rotation
/// are keyed on it. A closed burger has no token at all.
#[test]
fn the_opened_state_lands_on_the_glyph() {
    fn open() -> Element {
        rsx! { LiberoProvider { Burger { opened: true, "aria-controls": "nav" } } }
    }
    fn closed() -> Element {
        rsx! { LiberoProvider { Burger {} } }
    }

    assert_eq!(
        attributes_of(&body(&render(open)), "span")
            .get("data-state")
            .map(String::as_str),
        Some("opened")
    );
    assert!(
        !attributes_of(&body(&render(closed)), "span").contains_key("data-state"),
        "a closed burger must not carry the token"
    );
}

/// Both vars are declared on every render, never conditionally: a `Variables`
/// set that shrinks leaves the dropped custom property behind on the element,
/// so an unset `color` would keep whatever the last one was.
#[test]
fn the_glyph_declares_both_of_its_vars_even_when_nothing_is_set() {
    fn bare() -> Element {
        rsx! { LiberoProvider { Burger {} } }
    }
    fn coloured() -> Element {
        rsx! { LiberoProvider { Burger { color: "primary", size: "lg" } } }
    }

    let style = |app: fn() -> Element| {
        attributes_of(&body(&render(app)), "span")
            .get("style")
            .cloned()
            .expect("the glyph's inline vars")
    };

    let bare = style(bare);
    assert!(bare.contains("--lsx-burger-color:currentColor"), "{bare}");
    assert!(
        bare.contains("--lsx-burger-size-override:var(--lsx-burger-size)"),
        "{bare}"
    );

    let coloured = style(coloured);
    assert!(
        coloured.contains("--lsx-burger-color:var(--lsx-primary-6)"),
        "{coloured}"
    );
    assert!(
        coloured.contains("--lsx-burger-size-override:var(--lsx-burger-size-lg)"),
        "{coloured}"
    );
}

/// The button is the glyph plus one spacing step.
#[test]
fn it_sizes_the_button_around_the_glyph() {
    fn app() -> Element {
        rsx! { LiberoProvider { Burger {} } }
    }

    let style = attributes_of(&body(&render(app)), "button")
        .get("style")
        .cloned()
        .expect("the button's inline vars");

    assert!(
        style.contains(
            "--lsx-action-icon-size-override:calc(var(--lsx-burger-size) + var(--lsx-spacing-xs))"
        ),
        "{style}"
    );
}

#[test]
fn disabled_reaches_the_underlying_button() {
    fn app() -> Element {
        rsx! { LiberoProvider { Burger { disabled: true } } }
    }

    let attributes = attributes_of(&body(&render(app)), "button");
    let html = body(&render(app));
    assert!(
        attributes.contains_key("disabled") || html.contains("disabled"),
        "{html}"
    );
}
