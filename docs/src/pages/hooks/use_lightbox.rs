use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Code, CodeBlock, Flex, Image, Text},
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
    sx::sx,
};

const GALLERY: &str = r#"#[derive(Clone, PartialEq)]
struct Photo {
    src: String,
    alt: String,
}

#[component]
fn Gallery(photos: Vec<Photo>) -> Element {
    let lightbox = use_lightbox(LightboxOptions::default());
    let items: Vec<LightboxItem> = photos.iter().map(|p| LightboxItem::new(&p.src, &p.alt)).collect();

    rsx! {
        Flex { direction: "row", gap: "xs",
            for (index, photo) in photos.iter().enumerate() {
                Box {
                    component: "button",
                    r#type: "button",
                    aria_label: "Open {photo.alt}",
                    sx: sx().width("96px").height("96px").padding("0").border_width("0"),
                    onclick: {
                        let items = items.clone();
                        move |_| { lightbox.open_with((items.clone(), index)); }
                    },
                    Image { src: "{photo.src}", alt: "", fit: "cover" }
                }
            }
        }
    }
}"#;

static PICTURES: [(Asset, &str); 3] = [
    (asset!("/assets/gallery/1.svg"), "Breakfast"),
    (asset!("/assets/gallery/3.svg"), "Camera"),
    (asset!("/assets/gallery/4.svg"), "Coffee"),
];

#[derive(Clone, PartialEq)]
struct Photo {
    src: String,
    alt: String,
}

/// `GALLERY`, rendered.
#[component]
fn Gallery(photos: Vec<Photo>) -> Element {
    let lightbox = use_lightbox(LightboxOptions::default());
    let items: Vec<LightboxItem> = photos
        .iter()
        .map(|p| LightboxItem::new(&p.src, &p.alt))
        .collect();

    rsx! {
        Flex { direction: "row", gap: "xs",
            for (index, photo) in photos.iter().enumerate() {
                Box {
                    component: "button",
                    r#type: "button",
                    aria_label: "Open {photo.alt}",
                    sx: sx().width("96px").height("96px").padding("0").border_width("0"),
                    onclick: {
                        let items = items.clone();
                        move |_| {
                            lightbox.open_with((items.clone(), index));
                        }
                    },
                    Image { src: "{photo.src}", alt: "", fit: "cover" }
                }
            }
        }
    }
}

#[component]
pub fn UseLightboxPage() -> Element {
    let photos: Vec<Photo> = PICTURES
        .iter()
        .map(|(src, alt)| Photo {
            src: src.to_string(),
            alt: alt.to_string(),
        })
        .collect();

    rsx! {
        DocPage {
            title: "use_lightbox",
            source: "libero/src/components/overlay/use_lightbox.rs",
            markdown: "/md/use_lightbox.md",
            lead: rsx! {
                Text {
                    Code { source: "use_lightbox(options) -> ModalHandle<LightboxOpening>" }
                    " registers a picture viewer over the page. Open it with the pictures "
                    "and the index to start on, or with one picture. "
                    Anchor { to: Route::LightboxPage {}, "Lightbox" }
                    " covers zoom, thumbnails, captions and swiping."
                }
            },

            DocSection {
                title: "Usage",
                Gallery { photos }
                CodeBlock { source: GALLERY, language: "rust" }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "Each thumbnail is a button named for its picture, and the image inside "
                    "has an empty "
                    Code { source: "alt" }
                    ", so the name is read once. The viewer is a modal, so focus returns to "
                    "the thumbnail when it closes."
                }
            }
        }
    }
}
