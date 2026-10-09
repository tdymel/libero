//! `CircularProgress`'s rendered contract: the `progressbar` ARIA set on the root,
//! the arc as SVG attributes, and a label that is drawn but not the name.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::CircularProgress};

#[test]
fn the_root_carries_the_role_the_raw_values_and_the_name() {
    fn app() -> Element {
        rsx! { LiberoProvider { CircularProgress { aria_label: "Upload", value: 3.0, max: 10.0 } } }
    }

    let html = body(&render(app));
    let root = attributes_of(&html, "span");

    assert_eq!(root["role"], "progressbar", "{html}");
    assert_eq!(root["aria-label"], "Upload", "{html}");
    assert_eq!(root["aria-valuemin"], "0", "{html}");
    assert_eq!(root["aria-valuemax"], "10", "{html}");
    assert_eq!(root["aria-valuenow"], "3", "{html}");
    assert_eq!(root["aria-valuetext"], "30%", "{html}");
}

/// Native bakes an inline svg from its attributes, so the arc's length is one.
#[test]
fn the_arc_is_drawn_by_svg_attributes() {
    fn app() -> Element {
        rsx! { LiberoProvider { CircularProgress { aria_label: "Upload", value: 25.0 } } }
    }

    let html = body(&render(app));
    let svg = attributes_of(&html, "svg");
    let circle = attributes_of(&html, "circle");

    assert_eq!(svg["aria-hidden"], "true", "{html}");
    assert_eq!(svg["data-slot"], "arc", "{html}");
    assert_eq!(circle["stroke"], "currentColor", "{html}");
    assert_eq!(circle["stroke-dasharray"], "282.74 282.74", "{html}");
    // Three quarters of the ring left undrawn.
    assert_eq!(circle["stroke-dashoffset"], "212.06", "{html}");
}

#[test]
fn indeterminate_drops_valuenow_and_the_percentage() {
    fn app() -> Element {
        rsx! { LiberoProvider { CircularProgress { aria_label: "Connecting", value: None } } }
    }

    let html = body(&render(app));
    let root = attributes_of(&html, "span");

    assert!(!root.contains_key("aria-valuenow"), "{html}");
    assert!(!root.contains_key("aria-valuetext"), "{html}");
    assert!(
        root["data-state"]
            .split(' ')
            .any(|token| token == "indeterminate"),
        "{html}"
    );
}

/// `progressbar` takes its name from the author only, and the label is hidden: the
/// value is already read.
#[test]
fn the_label_is_drawn_but_not_the_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                CircularProgress { aria_label: "Upload", value: 3.0, max: 8.0, aria_valuetext: "3 of 8 files", "3/8" }
            }
        }
    }

    let html = body(&render(app));
    let root = attributes_of(&html, "span");

    assert_eq!(root["aria-label"], "Upload", "{html}");
    assert_eq!(root["aria-valuetext"], "3 of 8 files", "{html}");
    let label = html
        .split('<')
        .find(|tag| tag.contains(r#"data-slot="label""#))
        .expect("a label");
    assert!(label.contains(r#"aria-hidden="true""#), "{html}");
    assert!(html.contains("3/8"), "{html}");
}

/// Yellow gets its ink edge as a second, wider ring under the arc (todo 2033).
#[test]
fn a_warning_arc_sits_on_an_ink_edge() {
    fn app() -> Element {
        rsx! { LiberoProvider { CircularProgress { aria_label: "Quota", value: 80.0, color: "warning" } } }
    }

    let html = body(&render(app));
    assert_eq!(html.matches("<circle").count(), 2, "{html}");
}
