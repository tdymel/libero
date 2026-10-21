//! `Share`'s rendered contract: the icon-only name comes from the
//! localization or `aria_label`, `children` make a labelled button, and the
//! status is mounted empty before any share.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Share,
    localization::{Localization, ShareLabels},
};

static GERMAN: Localization = Localization {
    share: ShareLabels::GERMAN,
    ..Localization::ENGLISH
};

#[test]
fn the_name_comes_from_the_localization() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &GERMAN, Share { url: "https://libero-ui.dev" } }
        }
    }

    let html = body(&render(app));

    assert_eq!(attributes_of(&html, "button")["aria-label"], "Teilen");
}

/// `aria_label` wins, and the status waits, empty, beside the button.
#[test]
fn aria_label_names_it_and_the_status_is_mounted_empty() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Share { url: "https://libero-ui.dev", aria_label: "Share this page" } }
        }
    }

    let html = body(&render(app));

    assert_eq!(
        attributes_of(&html, "button")["aria-label"],
        "Share this page"
    );
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(!html.contains("Copied"), "{html}");
}

/// Children make a labelled button: named by its text, with no `aria-label`.
#[test]
fn children_render_a_labelled_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Share { url: "https://libero-ui.dev", "Share article" } }
        }
    }

    let html = body(&render(app));

    let button = attributes_of(&html, "button");
    assert!(!button.contains_key("aria-label"), "{html}");
    assert!(html.contains("Share article"), "{html}");
    assert!(html.contains("<svg"), "the share glyph: {html}");
    assert!(html.contains(r#"role="status""#), "{html}");
}

/// The label's id joins the caller's `aria-describedby`, in both modes.
#[test]
fn the_label_joins_a_callers_aria_describedby() {
    fn icon() -> Element {
        rsx! {
            LiberoProvider {
                Share { url: "x", label: "This page", aria_describedby: "mine" }
            }
        }
    }
    fn labelled() -> Element {
        rsx! {
            LiberoProvider {
                Share { url: "x", label: "This page", aria_describedby: "mine", "Share" }
            }
        }
    }

    for app in [icon, labelled] {
        let html = body(&render(app));
        assert_eq!(html.matches("aria-describedby").count(), 1, "{html}");
        let described = &attributes_of(&html, "button")["aria-describedby"];
        let ids: Vec<&str> = described.split(' ').collect();
        assert_eq!(ids.len(), 2, "{html}");
        assert!(ids.contains(&"mine"), "{html}");
    }
}

#[test]
fn disabled_renders_a_disabled_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Share { url: "x", disabled: true } }
        }
    }

    let html = body(&render(app));

    assert_eq!(
        attributes_of(&html, "button")["aria-disabled"],
        "true",
        "{html}"
    );
}
