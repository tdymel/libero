//! `ButtonGroup`'s rendered contract: a named `group` whose defaults reach the
//! buttons inside, each button's own prop winning, and seams in the CSS.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Button, ButtonGroup, DirectionToggle},
};

fn states(html: &str, from: &str) -> String {
    let at = html
        .find(from)
        .unwrap_or_else(|| panic!("no {from} in {html}"));
    attributes_of(&html[at..], "button")
        .remove("data-state")
        .unwrap_or_default()
}

#[test]
fn the_group_is_a_named_horizontal_group() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { "aria-label": "Alignment",
                    Button { "Left" }
                }
            }
        }
    }
    let html = body(&render(app));
    let group = attributes_of(&html, "div");
    assert_eq!(group["role"], "group");
    assert_eq!(group["aria-label"], "Alignment");
    assert!(group["data-state"].contains("horizontal"), "{group:?}");
}

#[test]
fn the_defaults_reach_each_button_and_its_own_prop_wins() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { variant: "filled", size: "lg", radius: "xl", disabled: true,
                    Button { "One" }
                    Button { variant: "outlined", size: "xs", disabled: false, "Two" }
                    ActionIcon { aria_label: "Three", "3" }
                }
            }
        }
    }
    let html = body(&render(app));
    let has = |states: &str, state: &str| states.split(' ').any(|s| s == state);

    let one = states(&html, "<button");
    for state in ["filled", "size-lg", "radius-xl", "disabled"] {
        assert!(has(&one, state), "{state}: {one}");
    }
    let two = states(&html[html.find("One").unwrap()..], "<button");
    assert!(has(&two, "outlined") && has(&two, "size-xs"), "{two}");
    assert!(!has(&two, "disabled"), "{two}");

    let icon = attributes_of(&html[html.find("Two").unwrap()..], "button");
    assert!(has(&icon["data-state"], "filled"), "{icon:?}");
    assert!(icon.contains_key("disabled"), "{icon:?}");
}

#[test]
fn a_forwarding_toggle_takes_the_group_variant() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { variant: "tonal", DirectionToggle {} }
            }
        }
    }
    let html = body(&render(app));
    let toggle = attributes_of(&html, "button");
    assert!(toggle["data-state"].contains("tonal"), "{toggle:?}");
}

#[test]
fn outside_a_group_nothing_changes() {
    fn app() -> Element {
        rsx! { LiberoProvider { ActionIcon { aria_label: "Bare", "x" } } }
    }
    let html = body(&render(app));
    let icon = attributes_of(&html, "button");
    assert!(
        !icon
            .get("data-state")
            .is_some_and(|s| s.contains("outlined")),
        "chromeless without a variant: {icon:?}"
    );
}

#[test]
fn the_css_joins_the_seams_on_logical_sides() {
    fn app() -> Element {
        rsx! { LiberoProvider { ButtonGroup { Button { "A" } Button { "B" } } } }
    }
    let html = render(app);
    for rule in [
        ":not(:first-child):is(button, a)",
        "margin-inline-start:-1px",
        "border-start-start-radius:0",
        "border-end-end-radius:0",
        ":not([data-state~=\"outlined\"])",
    ] {
        assert!(html.contains(rule), "{rule}");
    }
}
