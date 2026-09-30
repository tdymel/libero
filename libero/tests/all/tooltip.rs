use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, Tooltip},
    sx::sx,
    theme::Size,
};

/// Closed, the bubble is not rendered: only the label, hidden, under the id
/// the trigger's `aria-describedby` names.
#[test]
fn a_closed_tooltip_keeps_its_label_as_a_hidden_description() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip {
                    label: rsx! { "Copy" },
                    label_id: "copy-tip",
                    open_delay: 300,
                    gap: Size::Sm,
                    Button { "C" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let at = body.find("id=\"copy-tip\"").expect("the label");
    let description = attributes_of(&body[body[..at].rfind('<').unwrap()..], "span");

    assert_eq!(description["id"], "copy-tip");
    assert_eq!(description["role"], "tooltip");
    assert!(description.contains_key("hidden"), "{description:?}");
    assert!(body.contains(">Copy<"));
    assert_eq!(
        body.matches("copy-tip").count(),
        1,
        "no bubble while closed"
    );
    // The trigger stays a real button inside the wrapper.
    assert!(body.contains(">C<"));
}

/// Open, the bubble is a fixed, portaled box, and the hidden label gives its
/// id up to it. Its width cap is in the class, where a caller's `sx` beats it.
#[test]
fn an_open_tooltip_portals_its_bubble() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip {
                    label: rsx! { "Copy" },
                    label_id: "copy-tip",
                    open: true,
                    sx: sx().max_width("12rem"),
                    Button { "C" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert_eq!(body.matches("id=\"copy-tip\"").count(), 1, "{body}");
    let at = body.find("id=\"copy-tip\"").unwrap();
    let bubble = attributes_of(&body[body[..at].rfind('<').unwrap()..], "span");

    assert_eq!(bubble["role"], "tooltip");
    assert!(!bubble.contains_key("hidden"), "{bubble:?}");
    // No close counts down on a bubble nobody left.
    assert!(!bubble.contains_key("data-closing"), "{bubble:?}");
    assert!(bubble["data-state"].contains("size-sm"), "{bubble:?}");
    assert!(bubble["style"].contains("position:fixed;"), "{bubble:?}");
    assert!(!bubble["style"].contains("max-width"), "{bubble:?}");
    assert!(html.contains("max-width:min(20rem,"));
    assert!(html.contains("max-width:12rem"));
}

/// Without a `label_id` nothing points at the label, so a closed tooltip
/// renders none of it.
#[test]
fn a_closed_tooltip_without_an_id_renders_no_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip { label: rsx! { "Copy" }, Button { "C" } }
            }
        }
    }

    let html = render(app);
    assert!(!body(&html).contains("Copy"));
}

#[test]
fn a_disabled_tooltip_renders_its_trigger_bare() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip { label: rsx! { "Copy" }, label_id: "copy-tip", disabled: true, Button { "C" } }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert!(body.contains(">C<"));
    assert!(!body.contains("copy-tip"));
}
