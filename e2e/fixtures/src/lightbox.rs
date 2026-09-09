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

/// The six pictures, each URL marked with `gallery`, so a second gallery of
/// the same files is fetched under URLs of its own and the test can tell
/// which gallery a request came from. The thumbnails get URLs of their own
/// too: the strip loads every one of them, and would hide what the stage's
/// lazy loading fetched.
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

/// Two triggers into one viewer: the first picture, and the last - the far
/// end a gallery swap scrolls back from. `#swap-gallery` sits under the
/// modal, so only a script can press it: it is the second `open_with` while
/// the viewer is open (todo 323), which no control inside the viewer makes.
#[component]
fn LightboxPage() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        // Named, or every open logs the missing-name warning and the console
        // pass fails. The theme's own name, so the fixture reads as a caller
        // doing it right rather than renaming the component.
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
