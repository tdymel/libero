//! `Button`'s `loading` state: the label stays, the loader is silent, and the
//! button stays focusable.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Button};

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
