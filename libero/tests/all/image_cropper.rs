use crate::common::{body, render, tags_with};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::ImageCropper};

/// Todo 3833: a blank `aria_label` names nothing, so the crop box and its controls keep the default name.
#[test]
fn a_blank_aria_label_falls_back_to_the_localized_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ImageCropper { src: "/photo.png", alt: "Photo", aria_label: "  " }
            }
        }
    }

    let html = body(&render(app));
    let named = tags_with(&html, r#"aria-label=""#);
    assert!(!named.is_empty(), "{html}");
    for tag in named {
        assert!(!tag["aria-label"].trim().is_empty(), "{tag:?}");
    }
    let boxed = tags_with(&html, r#"role="slider""#);
    assert!(
        boxed.iter().any(|tag| tag.contains_key("aria-label")),
        "{html}"
    );
}
