//! `ImageCropper` over a 400 x 200 picture: free and controlled, then square
//! and uncontrolled. `#crop` reads the box in whole percent, `#pixels` in px.
//! `/refit` swaps the picture and the aspect; `/tiny` starts at `min_size`;
//! `/broken` has a `src` that fails, `#error` counts its `onerror` calls.
//! `/start` is free and unset, as the docs demo starts (1343).

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
    ("/image-cropper/refit", || rsx! { RefitCropperPage {} }),
    ("/image-cropper/tiny", || rsx! { TinyCropperPage {} }),
    ("/image-cropper/broken", || rsx! { BrokenCropperPage {} }),
    ("/image-cropper/start", || rsx! { StartCropperPage {} }),
];

const PICTURE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='200'%3E%3Crect width='400' height='200' fill='%23339af0'/%3E%3Ccircle cx='200' cy='100' r='60' fill='%23ffd43b'/%3E%3C/svg%3E";

const TALL: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='400'%3E%3Crect width='200' height='400' fill='%2340c057'/%3E%3C/svg%3E";

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
                id: "cropper",
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

#[component]
fn RefitCropperPage() -> Element {
    let mut tall = use_signal(|| false);
    let mut wide = use_signal(|| false);
    let mut crop = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            Button { id: "swap", onclick: move |_| tall.toggle(), "Swap picture" }
            Button { id: "widen", onclick: move |_| wide.toggle(), "Widen" }
            ImageCropper {
                src: if tall() { TALL } else { PICTURE },
                alt: "A picture",
                aspect: if wide() { 2.0 } else { 1.0 },
                onchange: move |next| crop.set(reading(next)),
            }
            p { id: "crop", "{crop}" }
        }
    }
}

#[component]
fn TinyCropperPage() -> Element {
    let mut crop = use_signal(|| CropRect {
        x: 0.4,
        y: 0.4,
        width: 0.05,
        height: 0.05,
    });

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                src: PICTURE,
                alt: "A sun on blue",
                min_size: 0.05,
                value: crop(),
                onchange: move |next| crop.set(next),
            }
            p { id: "crop", "{reading(crop())}" }
        }
    }
}

#[component]
fn BrokenCropperPage() -> Element {
    let mut errors = use_signal(|| 0);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                src: "data:image/png;base64,bm90IGFuIGltYWdl",
                alt: "A broken picture",
                onchange: |_: CropRect| {},
                onerror: move |()| errors += 1,
            }
            p { id: "error", "{errors}" }
        }
    }
}

#[component]
fn StartCropperPage() -> Element {
    let mut crop = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                src: PICTURE,
                alt: "A sun on blue",
                onchange: move |next| crop.set(reading(next)),
            }
            p { id: "crop", "{crop}" }
        }
    }
}
