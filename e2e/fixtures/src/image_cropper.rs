//! `ImageCropper` over a 400 x 200 picture: free and controlled, then square
//! and uncontrolled. `#crop` reads the box in whole percent, `#pixels` in px.

use dioxus::prelude::*;
use libero::components::{Button, CropRect, CropShape, Flex, ImageCropper};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/image-cropper", || rsx! { ImageCropperPage {} }),
    ("/image-cropper/square", || rsx! { SquareCropperPage {} }),
    (
        "/image-cropper/disabled",
        || rsx! { DisabledCropperPage {} },
    ),
];

const PICTURE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='200'%3E%3Crect width='400' height='200' fill='%23339af0'/%3E%3Ccircle cx='200' cy='100' r='60' fill='%23ffd43b'/%3E%3C/svg%3E";

fn reading(crop: CropRect) -> String {
    let percent = |fraction: f64| (fraction * 100.0).round();
    format!(
        "{},{},{},{}",
        percent(crop.x),
        percent(crop.y),
        percent(crop.width),
        percent(crop.height)
    )
}

#[component]
fn ImageCropperPage() -> Element {
    let mut crop = use_signal(|| CropRect {
        x: 0.25,
        y: 0.25,
        width: 0.5,
        height: 0.5,
    });

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            Button { id: "before", "Before" }
            ImageCropper {
                src: PICTURE,
                alt: "A sun on blue",
                value: crop(),
                onchange: move |next| crop.set(next),
            }
            p { id: "crop", "{reading(crop())}" }
        }
    }
}

#[component]
fn DisabledCropperPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                src: PICTURE,
                alt: "A sun on blue",
                disabled: true,
                onchange: |_: CropRect| {},
            }
        }
    }
}

#[component]
fn SquareCropperPage() -> Element {
    let mut pixels = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                src: PICTURE,
                alt: "A sun on blue",
                aspect: 1.0,
                shape: CropShape::Circle,
                onchange: move |next: CropRect| {
                    let rect = next.to_pixels(400, 200);
                    pixels.set(format!("{}x{}", rect.width, rect.height));
                },
            }
            p { id: "pixels", "{pixels}" }
        }
    }
}
