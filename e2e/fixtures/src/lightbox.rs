//! `Lightbox`, for the overlay archetype.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex},
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/lightbox", || rsx! { LightboxPage {} })];

static GALLERY: [Asset; 6] = [
    asset!("/assets/gallery/1.svg"),
    asset!("/assets/gallery/2.svg"),
    asset!("/assets/gallery/3.svg"),
    asset!("/assets/gallery/4.svg"),
    asset!("/assets/gallery/5.svg"),
    asset!("/assets/gallery/6.svg"),
];

/// URLs tagged with the gallery name, thumbnails separately, so the test can tell which
/// gallery and which part (strip or lazy stage) fetched a request.
fn gallery(name: &str) -> Vec<LightboxItem> {
    GALLERY
        .iter()
        .enumerate()
        .map(|(i, src)| {
            LightboxItem::new(format!("{src}?{name}"), format!("Picture {}", i + 1))
                .thumbnail(format!("{src}?{name}-thumbnail"))
                .caption(format!("Picture {} of the {name} gallery", i + 1))
        })
        .collect()
}

/// Triggers for the first and last picture. `#swap-gallery` sits under the modal, so only a
/// script presses it: a second `open_with` while open (todo 323).
#[component]
fn LightboxPage() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        // Unnamed, every open logs the missing-name warning and the console pass fails.
        aria_label: Some("Gallery".into()),
        ..LightboxOptions::default()
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "open-lightbox",
                variant: "outlined",
                onclick: move |_| {
                    lightbox.open_with(gallery("first"));
                },
                "Open the gallery"
            }
            Button {
                id: "open-lightbox-last",
                variant: "outlined",
                onclick: move |_| {
                    lightbox.open_with((gallery("first"), 5));
                },
                "Open the last picture"
            }
            Button {
                id: "swap-gallery",
                variant: "text",
                onclick: move |_| {
                    lightbox.open_with((gallery("second"), 2));
                },
                "Swap the gallery"
            }
        }
    }
}
