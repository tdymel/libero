//! `CopyButton`'s rendered contract: the name comes from the localization or
//! `aria_label`, and the status is mounted empty before any copy.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::CopyButton,
    localization::{CopyButtonLabels, Localization},
};

static GERMAN: Localization = Localization {
    copy_button: CopyButtonLabels::GERMAN,
    ..Localization::ENGLISH
};

#[test]
fn the_name_comes_from_the_localization() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &GERMAN, CopyButton { value: "x" } }
        }
    }

    let html = body(&render(app));

    assert_eq!(attributes_of(&html, "button")["aria-label"], "Kopieren");
}

/// `aria_label` wins, and the status waits, empty, beside the button.
#[test]
fn aria_label_names_it_and_the_status_is_mounted_empty() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { CopyButton { value: "x", aria_label: "Copy link" } }
        }
    }

    let html = body(&render(app));

    assert_eq!(attributes_of(&html, "button")["aria-label"], "Copy link");
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(!html.contains("Copied"), "{html}");
}
