//! The card variant of `Checkbox`, `Radio` and `RadioGroup`: the field
//! wrapper drawn as a surface that is the hit area. What a click does is only
//! reachable in a browser; this pins the markup and the rules it relies on.

mod common;

use common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Checkbox, Options, Radio, RadioGroup},
};

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
}

#[test]
fn a_card_checkbox_marks_its_wrapper_and_keeps_its_input_wiring() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox {
                    variant: "card",
                    label: "Priority support",
                    description: "Answers within four hours.",
                    checked: true,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let wrapper = attributes_of(&body, "div");
    let input = attributes_of(&body, "input");
    let id = &input["id"];

    assert!(
        wrapper["data-state"]
            .split(' ')
            .any(|token| token == "card"),
        "{wrapper:?}"
    );
    // The card changes the surface, not the control or its a11y.
    assert_eq!(input["type"], "checkbox");
    assert_eq!(input["aria-describedby"], format!("{id}-description"));
    assert_eq!(attributes_of(&body, "label")["for"], *id);

    // The card is drawn from Paper's tokens, and rings itself.
    let class = &classes_of(&body, "div")[0];
    assert!(
        html.contains(&format!(
            r#".{class}[data-state~="card"]{{background:var(--lsx-paper-background);border:1px solid var(--lsx-paper-border-color);border-radius:var(--lsx-paper-radius);"#
        )),
        "{html}"
    );
    assert!(html.contains(&format!(
        r#".{class}[data-state~="card"]:has(input:focus-visible){{outline:"#
    )));
}

#[test]
fn the_control_drops_its_ring_inside_a_card() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox { variant: "card", label: "A", checked: false, onchange: move |_| {} }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let control = &classes_of(&body, "span")[0];
    assert!(
        html.contains(&format!(
            r#".{control}[data-state~="card"]:has(> input:focus-visible){{outline:none;}}"#
        )),
        "{html}"
    );
}

#[test]
fn a_plain_checkbox_is_not_a_card() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox { label: "A", checked: false, onchange: move |_| {} }
            }
        }
    }

    let body = body(&render(app));
    let wrapper = attributes_of(&body, "div");
    assert!(
        !wrapper["data-state"]
            .split(' ')
            .any(|token| token == "card"),
        "{wrapper:?}"
    );
}

#[test]
fn a_card_radio_marks_its_wrapper() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Radio { variant: "card", label: "Free", checked: false, onselect: move |_| {} }
            }
        }
    }

    let body = body(&render(app));
    let wrapper = attributes_of(&body, "div");
    assert!(
        wrapper["data-state"]
            .split(' ')
            .any(|token| token == "card"),
        "{wrapper:?}"
    );
}

/// Every option becomes a card and carries its own description; the group
/// itself stays a plain field.
#[test]
fn a_card_radio_group_describes_each_option() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                RadioGroup {
                    variant: "card",
                    label: "Plan",
                    value: Plan::Pro,
                    onchange: move |_| {},
                    option_description: move |plan: Plan| match plan {
                        Plan::Free => "Up to three projects.".to_string(),
                        Plan::Pro => String::new(),
                    },
                }
            }
        }
    }

    let body = body(&render(app));
    // Each option's wrapper, not the control span inside it.
    let cards = body.matches(r#"inline card"><span"#).count();
    assert_eq!(cards, 2, "{body}");
    assert!(!attributes_of(&body, "div")["data-state"].contains("card"));

    let first = attributes_of(&body, "input");
    let id = &first["id"];
    assert_eq!(first["aria-describedby"], format!("{id}-description"));
    assert!(body.contains("Up to three projects."));
    // An empty description renders no slot at all.
    assert_eq!(
        body.matches(r#"data-slot="description""#).count(),
        1,
        "{body}"
    );
}
