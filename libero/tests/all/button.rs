//! `Button`'s `loading` state: the label stays, the loader is silent, and the
//! button stays focusable.

use crate::common::{attributes_of, body, classes_of, has_rule_for, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Button, theme::Color};

/// Busy, not disabled: native `disabled` would drop focus mid-wait, so the
/// button says `aria-busy` + `aria-disabled` and stays in the tab order. The
/// label is still in the tree - it is the accessible name - and the loader
/// beside it is `aria-hidden`, since the button already has one.
#[test]
fn loading_keeps_the_label_and_marks_the_button_busy() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { loading: true, "Save changes" } }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["aria-busy"], "true", "{attributes:?}");
    assert_eq!(attributes["aria-disabled"], "true", "{attributes:?}");
    assert!(!attributes.contains_key("disabled"), "{attributes:?}");
    assert!(
        attributes["data-state"].contains("loading"),
        "{attributes:?}"
    );
    assert!(html.contains("<span>Save changes</span>"), "{html}");
    assert!(html.contains(r#"aria-hidden="true""#), "{html}");
}

/// Off, the button is exactly what it was: no wrapper, no loader, no ARIA.
#[test]
fn not_loading_renders_the_children_bare() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { "Save changes" } }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "button");

    assert!(!attributes.contains_key("aria-busy"), "{attributes:?}");
    assert!(!attributes.contains_key("aria-disabled"), "{attributes:?}");
    assert!(!html.contains("<span"), "{html}");
}

/// The tint is the lightest shade and its label the *contrast* of that shade,
/// not a darker tone of the hue - which our ramp cannot make legible.
#[test]
fn a_tonal_button_labels_its_container_with_that_shade_s_contrast() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { variant: "tonal", color: Color::Warning, "Tint" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["data-state"], "tonal size-md radius-md");
    assert!(attributes["style"].contains("--lsx-button-container:var(--lsx-warning-fill-1);"));
    assert!(
        attributes["style"].contains("--lsx-button-on-container:var(--lsx-warning-contrast-1);")
    );
}

/// Elevated sits on the surface and is separated by its shadow alone - so it
/// paints no container, and the shadow comes from the shared scale.
#[test]
fn an_elevated_button_keeps_the_surface_and_reads_the_elevation_scale() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { variant: "elevated", color: Color::Error, "Lift" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["data-state"], "elevated size-md radius-md");
    assert!(!attributes["style"].contains("--lsx-button-container:"));
    assert!(html.contains("box-shadow:var(--lsx-shadow-xs)"));
    assert!(html.contains("--lsx-shadow-xs:"));
}

#[test]
fn button_renders_its_class_state_and_variables() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { color: Color::Error, "Save" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["type"], "button");
    assert_eq!(attributes["data-state"], "filled size-md radius-md");
    assert!(attributes["style"].contains("--lsx-button-color:var(--lsx-error-text-6);"));
    assert!(body(&html).contains(">Save<"));

    for class in classes_of(&html, "button") {
        assert!(has_rule_for(&html, &class), "{class} has no CSS rule");
    }
}

/// `<button>`'s HTML default is `submit`; ours is `button`, but only as a
/// fallback - a caller asking for a submit button has to get one.
#[test]
fn a_button_defaults_to_type_button_and_yields_to_the_caller() {
    fn plain() -> Element {
        rsx! { LiberoProvider { Button { "Save" } } }
    }
    fn submit() -> Element {
        rsx! { LiberoProvider { Button { r#type: "submit", "Save" } } }
    }

    assert_eq!(
        attributes_of(&render(plain), "button")
            .get("type")
            .map(String::as_str),
        Some("button")
    );
    assert_eq!(
        attributes_of(&render(submit), "button")
            .get("type")
            .map(String::as_str),
        Some("submit")
    );
}

/// `Elevated` paints the surface colour through a var, which `sx` cannot
/// infer a focus contrast from, so the arm publishes its own - and one with a
/// declared referent, since a set var that resolves to nothing erases the
/// ring rather than recolouring it.
#[test]
fn the_elevated_variant_reads_the_surface_and_publishes_its_focus_contrast() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { variant: "elevated", "Save" } }
        }
    }

    let html = render(app);
    let class = classes_of(&body(&html), "button")
        .into_iter()
        .next()
        .expect("a framework class");

    let selector = format!(r#".{class}[data-state~="elevated"]{{"#);
    let start = html.find(&selector).expect("the elevated arm's rule");
    let rule = &html[start..start + html[start..].find('}').expect("a closed rule")];
    assert!(
        rule.contains(
            "background:var(--lsx-paper-background);--lsx-focus-contrast:var(--lsx-paper-contrast);"
        ),
        "{rule}"
    );
    assert!(
        html.contains("--lsx-paper-contrast:"),
        "no referent declared"
    );
}

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[route("/settings")]
    Settings {},
}

#[component]
fn Settings() -> Element {
    rsx! {}
}

/// `to` takes a typed route directly, the way `Anchor`'s does. No router is
/// mounted, so it renders as a plain `href`.
#[test]
fn a_button_takes_a_route_as_its_target() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { to: Route::Settings {}, "Settings" } }
        }
    }

    let attributes = attributes_of(&body(&render(app)), "a");

    assert_eq!(attributes["href"], "/settings", "{attributes:?}");
}

/// The explicit conversion and a bare path still compile.
#[test]
fn a_button_still_takes_a_navigation_target_or_a_path() {
    fn target() -> Element {
        rsx! {
            LiberoProvider { Button { to: NavigationTarget::from(Route::Settings {}), "Settings" } }
        }
    }
    fn path() -> Element {
        rsx! {
            LiberoProvider { Button { to: "/settings", "Settings" } }
        }
    }

    for app in [target, path] {
        assert_eq!(attributes_of(&body(&render(app)), "a")["href"], "/settings");
    }
}

static OUTLINED: libero::theme::Theme = libero::theme::Theme {
    button: libero::theme::ButtonDefaults {
        variant: libero::theme::Variant::Outlined,
        ..libero::theme::Theme::DEFAULT.button
    },
    action_icon: libero::theme::ActionIconDefaults {
        variant: libero::theme::Variant::Outlined,
        ..libero::theme::Theme::DEFAULT.action_icon
    },
    chip: libero::theme::ChipDefaults {
        variant: libero::theme::Variant::Outlined,
        ..libero::theme::Theme::DEFAULT.chip
    },
    badge: libero::theme::BadgeDefaults {
        variant: libero::theme::Variant::Outlined,
        ..libero::theme::Theme::DEFAULT.badge
    },
    icon: libero::theme::IconDefaults {
        variant: libero::theme::Variant::Outlined,
        ..libero::theme::Theme::DEFAULT.icon
    },
    ..libero::theme::Theme::DEFAULT
};

/// A project sets the variant once in the theme, and every chrome component
/// with no `variant` of its own takes it. An `ActionIcon` draws chrome only
/// once it has a `color` or a `variant`, so it gets a `color`.
#[test]
fn an_unset_variant_follows_the_theme() {
    use libero::components::{ActionIcon, Badge, Chip, Icon};

    fn app() -> Element {
        rsx! {
            LiberoProvider { themes: &OUTLINED,
                Button { "Save" }
                ActionIcon { aria_label: "Close", color: Color::Primary, "x" }
                Chip { "Tag" }
                Badge { "New" }
                Icon { "i" }
            }
        }
    }

    let html = body(&render(app));
    let outlined = html
        .split("data-state=\"")
        .skip(1)
        .filter(|rest| {
            rest.split('"')
                .next()
                .is_some_and(|state| state.split(' ').any(|token| token == "outlined"))
        })
        .count();
    assert_eq!(outlined, 5, "{html}");
}
