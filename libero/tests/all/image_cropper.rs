use crate::common::{body, render, tags_with};
use crate::dispatch::Page;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{CropRect, ImageCropper},
};

/// Todo 4383: a press a controlled caller clamps says the clamped box, not the one asked for.
#[test]
fn a_clamped_press_says_the_box_the_caller_kept() {
    fn app() -> Element {
        let mut crop = use_signal(|| CropRect {
            x: 0.1,
            y: 0.1,
            width: 0.4,
            height: 0.5,
        });
        rsx! {
            LiberoProvider {
                ImageCropper {
                    src: "/photo.png",
                    alt: "Photo",
                    value: crop(),
                    onchange: move |next: CropRect| crop.set(CropRect { x: next.x.min(0.12), ..next }),
                }
            }
        }
    }

    let mut page = Page::mount(app);
    let right = page.rec.element("click", "aria-label", "Move right");
    page.click(right);
    page.dom.process_events();
    page.render();
    let html = body(&dioxus_ssr::render(&page.dom));
    assert!(html.contains("40% by 50%, at 12%, 10%"), "{html}");
}

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
