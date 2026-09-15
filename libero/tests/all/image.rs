use crate::common::{attributes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Image};

#[test]
fn image_renders_its_source_and_alt_text() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Image { src: "/logo.png", alt: "The logo" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "img");

    assert_eq!(attributes["src"], "/logo.png");
    assert_eq!(attributes["alt"], "The logo");
}

/// Todo 602: `decorative` hides the picture; no `alt` renders none, so a
/// checker still flags it rather than a silent `alt=""`.
#[test]
fn decorative_is_alt_empty_and_a_missing_alt_renders_none() {
    fn decorative() -> Element {
        rsx! { LiberoProvider { Image { src: "/logo.png", decorative: true } } }
    }
    fn missing() -> Element {
        rsx! { LiberoProvider { Image { src: "/logo.png" } } }
    }

    let img = attributes_of(&render(decorative), "img");
    assert_eq!(img["alt"], "");
    assert_eq!(img["role"], "presentation");

    let img = attributes_of(&render(missing), "img");
    assert!(!img.contains_key("alt"), "{img:?}");
    assert!(!img.contains_key("role"), "{img:?}");
}

/// A zoomable image opens a modal `Lightbox`, so its button is a dialog
/// opener, not a toggle.
#[test]
fn a_zoomable_image_is_a_dialog_opener() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Image { src: "/logo.png", alt: "The logo", zoomable: true, class: "mine" }
            }
        }
    }

    let html = render(app);
    let button = attributes_of(&html, "button");

    assert_eq!(button["type"], "button");
    assert_eq!(button["aria-haspopup"], "dialog");
    assert_eq!(button["aria-label"], "Zoom in: The logo");
    assert!(!button.contains_key("aria-pressed"), "{button:?}");
    assert!(button["class"].contains("mine"), "{button:?}");

    let img = attributes_of(&html, "img");
    assert_eq!(img["src"], "/logo.png");
    assert_eq!(img["alt"], "");
    assert!(!img["class"].contains("mine"), "{img:?}");
}
