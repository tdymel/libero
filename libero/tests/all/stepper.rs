//! `Stepper`'s rendered contract: the ordered list and `aria-current`, which
//! steps are derived as what, which are buttons, where the content goes in
//! each orientation, and the rail geometry it shares with `Timeline`.

use std::collections::BTreeMap;

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{OptionLabel, Options, StepState, Stepper},
};

#[derive(Clone, Debug, PartialEq, Options)]
enum Stage {
    Account,
    #[option(label = "Shipping address")]
    Shipping,
    Review,
}

fn content(stage: Stage) -> Element {
    match stage {
        Stage::Account => rsx! { "account body" },
        Stage::Shipping => rsx! { "shipping body" },
        Stage::Review => rsx! { "review body" },
    }
}

/// Every `<tag …>` open tag in the markup, in order.
fn all(html: &str, tag: &str) -> Vec<BTreeMap<String, String>> {
    let body = body(html);
    body.match_indices(&format!("<{tag}"))
        .filter(|(at, _)| {
            // `<li` must not match `<link`, nor `<span` a `<spanner`.
            matches!(body.as_bytes()[at + tag.len() + 1], b' ' | b'>')
        })
        .map(|(at, _)| attributes_of(&body[at..], tag))
        .collect()
}

fn states(html: &str) -> Vec<String> {
    all(html, "li")
        .into_iter()
        .map(|li| li.get("data-state").cloned().unwrap_or_default())
        .collect()
}

fn headers(html: &str) -> Vec<BTreeMap<String, String>> {
    let body = body(html);
    body.match_indices("data-step-header")
        .map(|(at, _)| {
            let open = body[..at].rfind('<').unwrap();
            let tag = &body[open + 1..][..body[open + 1..].find(' ').unwrap()];
            let mut attributes = attributes_of(&body[open..], tag);
            attributes.insert("tag".to_string(), tag.to_string());
            attributes
        })
        .collect()
}

fn middle_app() -> Element {
    rsx! {
        LiberoProvider {
            Stepper::<Stage> { id: "checkout", active: Some(Stage::Shipping), content: content }
        }
    }
}

#[test]
fn it_is_an_ordered_list_with_an_explicit_role() {
    let html = render(middle_app);
    let ol = attributes_of(&body(&html), "ol");
    assert_eq!(ol["role"], "list", "{html}");
    assert_eq!(all(&html, "li").len(), 3, "{html}");
}

#[test]
fn position_derives_completed_active_and_pending() {
    let html = render(middle_app);
    let states = states(&html);
    assert!(states[0].split(' ').any(|s| s == "completed"), "{states:?}");
    assert!(states[1].split(' ').any(|s| s == "active"), "{states:?}");
    assert!(states[2].split(' ').any(|s| s == "pending"), "{states:?}");
    // Horizontally a step carries the connector *before* it, so only the one
    // after a completed step is accented.
    assert!(!states[0].contains("line-active"), "{states:?}");
    assert!(states[1].contains("line-active"), "{states:?}");
    assert!(!states[2].contains("line-active"), "{states:?}");
}

#[test]
fn only_the_current_step_is_aria_current() {
    let html = render(middle_app);
    let headers = headers(&html);
    let current: Vec<_> = headers
        .iter()
        .map(|header| header.get("aria-current").map(String::as_str))
        .collect();
    assert_eq!(current, [None, Some("step"), None], "{html}");
    for (index, header) in headers.iter().enumerate() {
        assert_eq!(header["id"], format!("checkout-step-{index}"));
    }
}

#[test]
fn without_onstepclick_nothing_is_a_control() {
    let html = render(middle_app);
    assert!(!body(&html).contains("<button"), "{html}");
    for header in headers(&html) {
        // Focusable for the focus return only, never a tab stop.
        assert_eq!(header["tabindex"], "-1");
    }
}

#[test]
fn only_reached_steps_are_buttons_by_default() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> { active: Some(Stage::Shipping), onstepclick: |_| {} }
            }
        }
    }
    let html = render(app);
    let tags: Vec<_> = headers(&html)
        .into_iter()
        .map(|h| h["tag"].clone())
        .collect();
    assert_eq!(tags, ["button", "button", "span"], "{html}");
    for button in all(&html, "button") {
        assert_eq!(button["type"], "button");
        assert!(!button.contains_key("tabindex"), "{html}");
    }
}

#[test]
fn allow_next_steps_makes_every_step_a_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> {
                    active: Some(Stage::Account),
                    onstepclick: |_| {},
                    allow_next_steps: true,
                }
            }
        }
    }
    let html = render(app);
    assert_eq!(all(&html, "button").len(), 3, "{html}");
}

#[test]
fn no_active_step_means_every_step_is_finished() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> { active: None, content: content }
            }
        }
    }
    let html = render(app);
    for state in states(&html) {
        assert!(state.split(' ').any(|s| s == "completed"), "{html}");
    }
    assert!(!body(&html).contains("aria-current"), "{html}");
    assert!(!body(&html).contains("role=\"region\""), "{html}");
    // Completion is said in words, not only by the check.
    assert_eq!(body(&html).matches(", Completed").count(), 3, "{html}");
}

#[test]
fn the_state_override_adds_an_error_without_moving_the_current_step() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> {
                    active: Some(Stage::Shipping),
                    state: |stage: Stage| (stage == Stage::Shipping).then_some(StepState::Error),
                }
            }
        }
    }
    let html = render(app);
    let states = states(&html);
    assert!(states[0].contains("completed"), "{states:?}");
    assert!(states[1].split(' ').any(|s| s == "error"), "{states:?}");
    assert!(!states[1].split(' ').any(|s| s == "active"), "{states:?}");
    assert!(states[2].contains("pending"), "{states:?}");
    assert_eq!(
        headers(&html)[1].get("aria-current").map(String::as_str),
        Some("step")
    );
    assert!(body(&html).contains(", Error"), "{html}");
}

#[test]
fn horizontal_content_is_one_region_for_the_current_step() {
    let html = render(middle_app);
    let body = body(&html);
    assert!(body.contains("shipping body"), "{html}");
    assert!(!body.contains("account body"), "{html}");
    assert!(!body.contains("review body"), "{html}");

    let at = body.find("role=\"region\"").expect("a region");
    let open = body[..at].rfind('<').unwrap();
    let region = attributes_of(&body[open..], "div");
    assert_eq!(region["id"], "checkout-content-1");
    assert_eq!(region["aria-labelledby"], "checkout-step-1");
}

#[test]
fn vertical_content_collapses_under_each_step() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> {
                    id: "checkout",
                    active: Some(Stage::Shipping),
                    orientation: "vertical",
                    label_position: "below",
                    content: content,
                }
            }
        }
    }
    let html = render(app);
    let body = body(&html);
    for index in 0..3 {
        let at = body
            .find(&format!("id=\"checkout-content-{index}\""))
            .unwrap_or_else(|| panic!("region {index} missing:\n{html}"));
        let open = body[..at].rfind('<').unwrap();
        let region = attributes_of(&body[open..], "div");
        assert_eq!(region["role"], "region");
        assert_eq!(region["aria-labelledby"], format!("checkout-step-{index}"));
    }
    // A closed step's content is not mounted.
    assert!(body.contains("shipping body"), "{html}");
    assert!(!body.contains("account body"), "{html}");
    assert!(!body.contains("review body"), "{html}");
    // Vertically a step carries the connector *after* it.
    let states = states(&html);
    assert!(states[0].contains("line-active"), "{states:?}");
    assert!(!states[1].contains("line-active"), "{states:?}");
    // The label position is ignored, and does not print a token.
    let root = attributes_of(&body, "div");
    assert!(!root["data-state"].contains("label-below"), "{root:?}");
}

#[test]
fn a_rich_label_is_drawn_hidden_and_named_in_text() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> {
                    active: Some(Stage::Account),
                    label: |stage: Stage| OptionLabel::rich(stage.label(), rsx! { b { "drawn" } }),
                }
            }
        }
    }
    let html = render(app);
    let body = body(&html);
    assert!(
        body.contains("<span aria-hidden=\"true\"><b>drawn</b>"),
        "{html}"
    );
    assert!(body.contains("Shipping address"), "{html}");
}

#[test]
fn a_description_prints_and_an_empty_one_does_not() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> {
                    active: Some(Stage::Account),
                    description: |stage: Stage| match stage {
                        Stage::Account => "Who you are".to_string(),
                        _ => String::new(),
                    },
                }
            }
        }
    }
    let html = render(app);
    assert_eq!(
        body(&html).matches("data-step-description").count(),
        1,
        "{html}"
    );
    assert!(body(&html).contains("Who you are"), "{html}");
}

/// The vertical rail goes through `Rail`, so it starts on the marker's
/// centreline and content clears the whole marker plus one gap.
#[test]
fn the_vertical_rail_is_the_shared_geometry() {
    let html = render(middle_app);
    let css: String = html.split_whitespace().collect();
    assert!(
        css.contains("left:calc(var(--lsx-stepper-marker)/2-var(--lsx-stepper-line-width)/2)"),
        "{html}"
    );
    assert!(
        css.contains("padding-left:calc(var(--lsx-stepper-marker)+var(--lsx-stepper-gap))"),
        "{html}"
    );
    assert!(css.contains(":not(:last-of-type)::before"), "{html}");
}

/// A per-instance colour sets the accent and its contrast twin on the root;
/// without one the root leaves the theme's `:root` value alone.
#[test]
fn a_colour_override_sets_the_accent_and_its_contrast() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> { active: Some(Stage::Account), color: "success" }
            }
        }
    }
    let root = attributes_of(&body(&render(app)), "div");
    assert!(
        root["style"].contains("--lsx-stepper-color:var(--lsx-success-text-6)"),
        "{root:?}"
    );
    assert!(
        root["style"].contains("--lsx-stepper-color-contrast:var(--lsx-success-contrast-6)"),
        "{root:?}"
    );

    let plain = attributes_of(&body(&render(middle_app)), "div");
    assert!(
        !plain
            .get("style")
            .is_some_and(|s| s.contains("--lsx-stepper-color")),
        "{plain:?}"
    );
}

/// Focus return finds a step by `{id}-step-{n}` and its content by
/// `{id}-content-{n}`, and a caller's id need not be a CSS identifier. The
/// lookup selects by attribute (todo 248), which only works while the step
/// headers carry exactly that id.
#[test]
fn a_caller_id_that_is_no_css_identifier_still_names_the_steps() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Stepper::<Stage> {
                    id: "1-faq",
                    active: Some(Stage::Shipping),
                    onstepclick: |_| {},
                    content: content,
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("id=\"1-faq-step-0\""), "{body}");
    assert!(body.contains("id=\"1-faq-content-1\""), "{body}");
}
