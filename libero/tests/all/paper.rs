use crate::common::{attributes_of, body, classes_of, has_rule_for, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Paper};

#[test]
fn a_paper_is_a_surface_with_no_semantics_of_its_own() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { radius: "lg", bordered: true, "sheet" }
            }
        }
    }

    let html = render(app);
    let paper = attributes_of(&body(&html), "div");

    assert_eq!(paper["data-state"], "radius-lg bordered");
    assert!(
        !paper.contains_key("role"),
        "a surface names nothing: {paper:?}"
    );
    for class in classes_of(&body(&html), "div") {
        assert!(
            has_rule_for(&html, &class),
            "{class} is referenced but never emitted"
        );
    }
    assert!(
        html.contains("background:var(--lsx-paper-background);"),
        "{html}"
    );
}

/// The themed default arrives as a plain declaration, not as a token - which
/// is what lets `Dialog` override it in its own base static.
#[test]
fn a_paper_that_names_no_step_carries_no_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { "sheet" }
            }
        }
    }

    let paper = attributes_of(&body(&render(app)), "div");

    assert!(!paper.contains_key("data-state"), "{paper:?}");
}

#[test]
fn a_paper_takes_the_interactive_look_only_once_it_can_be_pressed() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { id: "plain", "sheet" }
                Paper { id: "link", component: "a", href: "/orders/4021", "sheet" }
                Paper { id: "press", onclick: |_| {}, "sheet" }
                Paper { id: "button", component: "button", "sheet" }
                Paper { id: "bare-link", component: "a", "sheet" }
            }
        }
    }

    let html = render(app);
    let state = |tag: &str, id: &str| {
        let at = html.find(&format!("id=\"{id}\"")).expect(id);
        let start = html[..at].rfind(&format!("<{tag}")).expect(tag);
        attributes_of(&html[start..], tag)
            .get("data-state")
            .cloned()
    };

    assert_eq!(state("div", "plain"), None);
    assert_eq!(state("a", "link").as_deref(), Some("interactive"));
    assert_eq!(state("div", "press").as_deref(), Some("interactive"));
    assert_eq!(state("button", "button").as_deref(), Some("interactive"));
    assert_eq!(state("a", "bare-link"), None);
}

#[test]
fn the_interactive_look_tints_on_hover_and_press_and_rings_on_focus() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { component: "a", href: "/orders/4021", "sheet" }
            }
        }
    }

    let html = render(app);

    assert!(
        html.contains("[data-state~=\"interactive\"]:hover"),
        "{html}"
    );
    assert!(
        html.contains("[data-state~=\"interactive\"]:active"),
        "{html}"
    );
    assert!(
        html.contains("[data-state~=\"interactive\"]:focus-visible"),
        "{html}"
    );
}
