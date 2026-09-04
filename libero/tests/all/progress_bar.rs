//! `ProgressBar`'s rendered contract: the `progressbar` ARIA set on the root,
//! what indeterminate drops, and which `aria-valuetext` wins.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::ProgressBar};

/// The role and all four values sit on the root - the element that always has
/// size - and the fill inside it is decorative. The name reaches the root as a
/// plain global attribute, so there is no `aria_label` prop.
#[test]
fn the_root_carries_the_role_the_raw_values_and_the_name() {
    fn app() -> Element {
        rsx! { LiberoProvider { ProgressBar { aria_label: "Upload", value: 3.0, max: 10.0 } } }
    }

    let html = body(&render(app));
    let root = attributes_of(&html, "div");

    assert_eq!(root["role"], "progressbar", "{html}");
    assert_eq!(root["aria-label"], "Upload", "{html}");
    assert_eq!(root["aria-valuemin"], "0", "{html}");
    assert_eq!(root["aria-valuemax"], "10", "{html}");
    assert_eq!(root["aria-valuenow"], "3", "{html}");
    assert_eq!(root["aria-valuetext"], "30%", "{html}");
    assert!(
        root["data-state"]
            .split(' ')
            .any(|token| token == "determinate"),
        "{html}"
    );
    // The fill: one empty, hidden child. (The provider's portal root follows.)
    assert!(
        html.contains(r#"aria-hidden="true""#) && html.contains("></div></div>"),
        "{html}"
    );
}

/// `None` is how ARIA spells "busy, amount unknown": no `aria-valuenow`, and
/// no percentage to announce either.
#[test]
fn indeterminate_drops_valuenow_and_the_percentage() {
    fn app() -> Element {
        rsx! { LiberoProvider { ProgressBar { aria_label: "Connecting", value: None } } }
    }

    let html = body(&render(app));
    let root = attributes_of(&html, "div");

    assert!(!root.contains_key("aria-valuenow"), "{html}");
    assert!(!root.contains_key("aria-valuetext"), "{html}");
    assert_eq!(root["aria-valuemin"], "0", "{html}");
    assert!(
        root["data-state"]
            .split(' ')
            .any(|token| token == "indeterminate"),
        "{html}"
    );
    assert!(!html.contains("--lsx-progress-bar-fill"), "{html}");
}

#[test]
fn the_aria_valuetext_prop_replaces_the_percentage() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ProgressBar {
                    aria_label: "Downloading update",
                    value: 4.2,
                    max: 12.0,
                    aria_valuetext: "4.2 MB of 12 MB",
                }
            }
        }
    }

    let html = body(&render(app));
    assert_eq!(
        attributes_of(&html, "div")["aria-valuetext"],
        "4.2 MB of 12 MB",
        "{html}"
    );
}

/// The percentage is only a default, so a caller's own attribute beats it -
/// the reason it goes through `attr_default` and not `attr`.
#[test]
fn a_spread_aria_valuetext_beats_the_percentage() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ProgressBar { aria_label: "Track", value: 40.0, "aria-valuetext": "1:34 of 4:02" }
            }
        }
    }

    let html = body(&render(app));
    assert_eq!(
        attributes_of(&html, "div")["aria-valuetext"],
        "1:34 of 4:02",
        "{html}"
    );
    assert_eq!(html.matches("aria-valuetext").count(), 1, "{html}");
}
