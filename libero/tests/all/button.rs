//! `Button`'s `loading` state: the label stays, the loader is silent, and the
//! button stays focusable.

use crate::common::{
    attributes_of, body, classes_of, has_rule_for, nth_attributes, render, rules_for,
};

use std::collections::BTreeSet;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Button,
    theme::{Color, Size, SizeCss},
};

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

/// Todo 869: disabled but focusable - `aria-disabled` rather than `disabled`,
/// with the disabled look.
#[test]
fn a_focusable_disabled_button_stays_in_the_tab_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { disabled: true, focusable_when_disabled: true, "Next" } }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "button");

    assert!(!attributes.contains_key("disabled"), "{attributes:?}");
    assert_eq!(attributes["aria-disabled"], "true", "{attributes:?}");
    assert!(!attributes.contains_key("aria-busy"), "{attributes:?}");
    assert!(
        attributes["data-state"]
            .split(' ')
            .any(|token| token == "disabled"),
        "{attributes:?}"
    );
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

/// Todo 3993: at the start the loader takes the icon's slot and the label is no longer
/// dimmed under an overlay, so it is not wrapped.
#[test]
fn a_start_loader_replaces_the_icon_and_leaves_the_label_bare() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button {
                    loading: true,
                    loading_position: "start",
                    icon: rsx! { svg { id: "glyph" } },
                    "Save"
                }
            }
        }
    }

    let html = body(&render(app));
    let state = &attributes_of(&html, "button")["data-state"];

    assert!(tokens(state).contains("loading"), "{state}");
    assert!(!tokens(state).contains("loading-overlay"), "{state}");
    assert!(!html.contains(r#"id="glyph""#), "the icon stayed: {html}");
    assert!(html.contains(r#"<span data-slot="button-icon">"#), "{html}");
    assert!(html.contains("Save</button>"), "{html}");
    assert!(!html.contains("<span>Save"), "{html}");
}

/// Todo 3993: at the end it takes the trailing icon's.
#[test]
fn an_end_loader_replaces_the_trailing_icon() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button {
                    loading: true,
                    loading_position: "end",
                    trailing_icon: rsx! { svg { id: "glyph" } },
                    "Next"
                }
            }
        }
    }

    let html = body(&render(app));

    assert!(!html.contains(r#"id="glyph""#), "the icon stayed: {html}");
    assert!(
        html.contains(r#"Next<span data-slot="button-trailing-icon">"#),
        "{html}"
    );
    assert_eq!(attributes_of(&html, "button")["aria-busy"], "true");
}

/// Todo 3993: the custom loader stands in for the spinner, in the overlay by default
/// and in the slot at the start or end.
#[test]
fn a_custom_loader_replaces_the_spinner_in_every_position() {
    fn centered() -> Element {
        rsx! {
            LiberoProvider {
                Button { loading: true, loader: rsx! { i { id: "mine" } }, "Save" }
            }
        }
    }
    fn at_end() -> Element {
        rsx! {
            LiberoProvider {
                Button {
                    loading: true,
                    loading_position: "end",
                    loader: rsx! { i { id: "mine" } },
                    "Save"
                }
            }
        }
    }

    let html = body(&render(centered));
    assert!(
        html.contains(r#"<span>Save</span><span><i id="mine"></i></span></button>"#),
        "{html}"
    );
    assert!(tokens(&attributes_of(&html, "button")["data-state"]).contains("loading-overlay"));

    let html = body(&render(at_end));
    assert!(
        html.contains(
            r#"Save<span data-slot="button-trailing-icon"><i id="mine"></i></span></button>"#
        ),
        "{html}"
    );
}

/// Todo 3993: a position without `loading` changes nothing.
#[test]
fn a_loading_position_alone_renders_the_children_bare() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { loading_position: "start", "Save" } }
        }
    }

    let html = body(&render(app));

    assert!(html.contains(">Save</button>"), "{html}");
    assert!(!attributes_of(&html, "button").contains_key("aria-busy"));
}

/// Todo 3998: the prop `value` is also the `<button>`'s own attribute, so it still reaches the DOM.
#[test]
fn a_buttons_value_is_forwarded_to_the_native_attribute() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { value: "bold", "Bold" }
                Button { "Plain" }
            }
        }
    }

    let html = body(&render(app));

    assert_eq!(nth_attributes(&html, "button", 0)["value"], "bold");
    assert!(!nth_attributes(&html, "button", 1).contains_key("value"));
    assert!(
        !nth_attributes(&html, "button", 0).contains_key("aria-pressed"),
        "a button outside a selecting group is not a toggle"
    );
}

/// Todo 662: the children are the button's own flex items, never wrapped, so
/// a caller's icon, gap and `margin-left: auto` keep working. Never wider than
/// its container; a long label is cut, with no ellipsis.
#[test]
fn the_children_stay_the_buttons_flex_items() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { svg { id: "glyph" } "Save changes" } }
        }
    }

    let html = render(app);
    assert!(
        body(&html).contains(r#"<svg id="glyph"></svg>Save changes</button>"#),
        "{html}"
    );
    assert!(!html.contains("text-overflow"), "{html}");
    let rules = rules_for(&html, &attributes_of(&body(&html), "button"));
    assert!(rules.contains("max-width:100%;min-width:0;"), "{rules}");
}

/// Todo 4719: under a finger a button is 44px each way, drawn by its own box since the
/// ripple's `overflow: hidden` would clip a `::before`.
#[test]
fn a_button_grows_to_a_finger_on_a_coarse_pointer() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { size: "xs", "Save" } }
        }
    }

    let html = render(app);
    let media = html
        .find("@media (pointer: coarse)")
        .unwrap_or_else(|| panic!("no coarse query: {html}"));
    assert!(
        html[media..].contains("min-height:44px;min-width:44px;"),
        "{html}"
    );
}

/// Todo 662: `icon` gets a slot of its own before the children, kept whole and
/// a spacing step from them; while loading it hides with the children.
#[test]
fn the_icon_sits_in_its_own_slot_before_the_children() {
    fn idle() -> Element {
        rsx! {
            LiberoProvider { Button { icon: rsx! { svg { id: "glyph" } }, "Save" } }
        }
    }
    fn busy() -> Element {
        rsx! {
            LiberoProvider { Button { loading: true, icon: rsx! { svg { id: "glyph" } }, "Save" } }
        }
    }

    let html = render(idle);
    assert!(
        body(&html).contains(
            r#"<span data-slot="button-icon"><svg id="glyph"></svg></span>Save</button>"#
        ),
        "{html}"
    );
    let class = classes_of(&body(&html), "button");
    let class = class.first().expect("a framework class");
    assert!(
        html.contains(&format!(
            ".{class} > span > [data-slot='button-icon']{{display:inline-flex;"
        )),
        "{html}"
    );
    let html = body(&render(busy));
    assert!(
        html.contains(
            r#"<span><span data-slot="button-icon"><svg id="glyph"></svg></span>Save</span>"#
        ),
        "{html}"
    );
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

    assert_eq!(
        tokens(&attributes["data-state"]),
        tokens("tonal size-md radius-md")
    );
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

    assert_eq!(
        tokens(&attributes["data-state"]),
        tokens("elevated size-md radius-md")
    );
    assert!(!attributes["style"].contains("--lsx-button-container:"));
    let rules = rules_for(&html, &attributes);
    assert!(rules.contains("box-shadow:var(--lsx-shadow-xs)"), "{rules}");
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
    assert_eq!(
        tokens(&attributes["data-state"]),
        tokens("filled size-md radius-md")
    );
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

/// Todo 4466: a custom `radius` rides the override variable; one that leaves its declaration is refused.
#[test]
fn a_radius_that_leaves_its_declaration_is_refused() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { id: "safe", radius: "4px", "Safe" }
                Button { id: "hostile", radius: "0;position:fixed;background:url(https://evil.example/x)", "Hostile" }
            }
        }
    }

    let html = body(&render(app));
    let name = SizeCss::RADIUS.override_var();

    assert!(
        nth_attributes(&html, "button", 0)["style"].ends_with(&format!("{}:4px;", name.name())),
    );
    let hostile = nth_attributes(&html, "button", 1);
    assert!(
        hostile
            .get("style")
            .is_none_or(|style| !style.contains(name.name())),
        "{hostile:?}"
    );
    assert!(!html.contains("position:fixed"), "{html}");
}

/// A `data-state` as a set: the order the tokens are assembled in is not the contract.
fn tokens(state: &str) -> BTreeSet<&str> {
    state.split_whitespace().collect()
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

/// An optional target is a link when set and a plain `<button>` when `None`.
#[test]
fn an_optional_target_is_a_link_or_a_plain_button() {
    fn route() -> Element {
        rsx! {
            LiberoProvider { Button { to: Some(Route::Settings {}), "Settings" } }
        }
    }
    fn path() -> Element {
        rsx! {
            LiberoProvider { Button { to: Some("/settings"), "Settings" } }
        }
    }
    fn owned() -> Element {
        rsx! {
            LiberoProvider { Button { to: Some("/settings".to_string()), "Settings" } }
        }
    }
    fn no_route() -> Element {
        rsx! {
            LiberoProvider { Button { to: None::<Route>, "Settings" } }
        }
    }
    fn no_path() -> Element {
        rsx! {
            LiberoProvider { Button { to: None::<&str>, "Settings" } }
        }
    }

    for app in [route, path, owned] {
        assert_eq!(attributes_of(&body(&render(app)), "a")["href"], "/settings");
    }
    for app in [no_route, no_path] {
        let html = body(&render(app));
        assert!(!html.contains("<a"), "{html}");
        assert!(html.contains("<button"), "{html}");
    }
}

/// `justify` reaches the base rule through a per-button variable.
#[test]
fn gap_is_set_as_a_spacing_variable_on_the_button() {
    fn sized() -> Element {
        rsx! {
            LiberoProvider { Button { gap: "lg", "Menu" } }
        }
    }
    fn css() -> Element {
        rsx! {
            LiberoProvider { Button { gap: "0", "Menu" } }
        }
    }

    let sized = attributes_of(&body(&render(sized)), "button");
    let css = attributes_of(&body(&render(css)), "button");

    assert!(
        sized["style"].contains(&format!(
            "--lsx-button-gap:{}",
            SizeCss::SPACING.value(Size::Lg)
        )),
        "{sized:?}"
    );
    assert!(css["style"].contains("--lsx-button-gap:0"), "{css:?}");
}

#[test]
fn justify_is_set_as_a_variable_on_the_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Button { justify: "space-between", "Menu" } }
        }
    }
    fn unset() -> Element {
        rsx! {
            LiberoProvider { Button { "Menu" } }
        }
    }

    let set = attributes_of(&body(&render(app)), "button");
    let plain = attributes_of(&body(&render(unset)), "button");

    assert!(
        set["style"].contains("--lsx-button-justify:space-between"),
        "{set:?}"
    );
    assert!(
        !plain
            .get("style")
            .is_some_and(|style| style.contains("--lsx-button-justify")),
        "{plain:?}"
    );
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {

    use crate::common::body;
    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{LiberoProvider, components::Button};

    fn button_app() -> Element {
        rsx! { LiberoProvider { Button { "Click" } } }
    }

    #[test]
    fn clicking_a_button_runs_its_ripple() {
        assert_ripple_alternates(button_app);
    }

    /// Todo 3994: `ripple: false` still clicks, but draws nothing.
    #[test]
    fn a_button_without_ripple_runs_no_ripple_and_still_clicks() {
        fn app() -> Element {
            let mut clicks = use_signal(|| 0u32);
            rsx! {
                LiberoProvider {
                    Button { ripple: false, onclick: move |_| clicks += 1, "{clicks}" }
                }
            }
        }

        let mut page = Page::mount(app);
        let button = page.rec.click.expect("registered no click listener");
        page.click(button);
        page.click(button);

        let html = dioxus_ssr::render(&page.dom);
        assert!(
            !data_state(&html).contains("ripple"),
            "{}",
            data_state(&html)
        );
        assert!(body(&html).contains(">2</button>"), "{html}");
    }
}
