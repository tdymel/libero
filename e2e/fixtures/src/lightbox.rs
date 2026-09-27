//! `Lightbox`, for the overlay archetype.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, LightboxPart, Parts},
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/lightbox", || rsx! { LightboxPage {} }),
    ("/lightbox-parts", || rsx! { LightboxPartsPage {} }),
    ("/lightbox-tall", || rsx! { TallLightboxPage {} }),
];

/// A caption long enough to outgrow the viewport: the strip below must stay reachable (todo 1307).
#[component]
fn TallLightboxPage() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        aria_label: Some("Gallery".into()),
        ..LightboxOptions::default()
    });
    let caption = "A caption that runs on and on, as a long description does. ".repeat(40);

    rsx! {
        Button {
            id: "open-lightbox",
            variant: "outlined",
            onclick: move |_| {
                let items = gallery("tall")
                    .into_iter()
                    .map(|item| item.caption(caption.clone()))
                    .collect::<Vec<_>>();
                lightbox.open_with(items);
            },
            "Open the gallery"
        }
    }
}

/// `LightboxOptions::sx` styles the dialog, `parts` the caption and the thumbnails.
#[component]
fn LightboxPartsPage() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        aria_label: Some("Gallery".into()),
        sx: sx().letter_spacing("2px").into(),
        parts: Parts::new()
            .part(LightboxPart::Caption, sx().font_style("italic"))
            .part(LightboxPart::Thumbnail, sx().opacity("0.5"))
            .into(),
        ..LightboxOptions::default()
    });

    rsx! {
        Button {
            id: "open-lightbox",
            variant: "outlined",
            onclick: move |_| {
                lightbox.open_with(gallery("parts"));
            },
            "Open the gallery"
        }
    }
}

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
