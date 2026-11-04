//! `ImageCropper` over a 400 x 200 picture: free and controlled, then square
//! and uncontrolled. `#crop` reads the box in whole percent, `#pixels` in px.
//! `/refit` swaps the picture and the aspect; `/tiny` starts at `min_size`;
//! `/broken` has a `src` that fails, `#error` counts its `onerror` calls.
//! `/start` is free and unset, as the docs demo starts (1343). `/pan` is
//! square, unset and in pan mode (1368). `/bare` is free and pan with
//! `controls: false` (2112). `/overhang` has a 0.25 overhang on white and a cropper
//! with `ImageCropperControls::None` (free_ball). `/helpers` runs `platform::image`'s
//! helpers on two 2 x 1 pictures.

use dioxus::prelude::*;
use dioxus::html::FileData;
use libero::{
    components::{Button, CropRect, CropShape, Flex, ImageCropper, ImageCropperControls},
    platform::image::{Backdrop, crop_image, file_from_bytes, has_transparency, on_square},
};

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
    ("/image-cropper/pan", || rsx! { PanCropperPage {} }),
    ("/image-cropper/bare", || rsx! { BareCropperPage {} }),
    ("/image-cropper/overhang", || rsx! { OverhangCropperPage {} }),
    ("/image-cropper/helpers", || rsx! { HelpersPage {} }),
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

/// `#crop` reads what `#disabled` emits; `#disabled-pan` has the zoom slider.
#[component]
fn DisabledCropperPage() -> Element {
    let mut crop = use_signal(String::new);
    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                id: "disabled",
                src: PICTURE,
                alt: "A sun on blue",
                disabled: true,
                onchange: move |next| crop.set(reading(next)),
            }
            p { id: "crop", "{crop}" }
            ImageCropper {
                id: "disabled-pan",
                src: PICTURE,
                alt: "A sun on blue, to pan",
                aspect: 1.0,
                pan: true,
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

#[component]
fn PanCropperPage() -> Element {
    let mut crop = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                id: "cropper",
                src: PICTURE,
                alt: "A sun on blue",
                aspect: 1.0,
                pan: true,
                onchange: move |next| crop.set(reading(next)),
            }
            p { id: "crop", "{crop}" }
        }
    }
}

#[component]
fn BareCropperPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                id: "bare",
                src: PICTURE,
                alt: "A sun on blue",
                controls: false,
                onchange: |_: CropRect| {},
            }
            ImageCropper {
                id: "bare-pan",
                src: PICTURE,
                alt: "A sun on blue, to pan",
                aspect: 1.0,
                pan: true,
                controls: false,
                onchange: |_: CropRect| {},
            }
        }
    }
}

#[component]
fn OverhangCropperPage() -> Element {
    let mut crop = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            ImageCropper {
                id: "over",
                src: PICTURE,
                alt: "A sun on blue",
                overhang: 0.25,
                backdrop: Backdrop::rgb(255, 255, 255),
                onchange: move |next| crop.set(reading(next)),
            }
            p { id: "crop", "{crop}" }
            ImageCropper {
                id: "none",
                src: PICTURE,
                alt: "A sun on blue, without buttons",
                controls: ImageCropperControls::None,
                onchange: |_: CropRect| {},
            }
        }
    }
}

/// A 2 x 1 RGBA PNG whose left pixel is see-through, and a 2 x 1 RGB one.
const SEE_THROUGH: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 1, 8, 6, 0, 0,
    0, 244, 34, 127, 138, 0, 0, 0, 15, 73, 68, 65, 84, 120, 156, 99, 248, 207, 0, 2, 255, 255, 3, 0,
    10, 254, 2, 254, 95, 45, 130, 53, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];
const OPAQUE: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 1, 8, 2, 0, 0,
    0, 123, 64, 232, 221, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 207, 0, 4, 255, 1, 7, 0, 1,
    255, 226, 35, 158, 89, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

/// A PNG's size from its IHDR, big-endian from byte 16.
async fn size(file: Option<FileData>) -> String {
    let Some(file) = file else {
        return "none".to_string();
    };
    let bytes = file.read_bytes().await.unwrap_or_default();
    let at = |from: usize| {
        bytes
            .get(from..from + 4)
            .map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    };
    format!("{}x{}", at(16), at(20))
}

#[component]
fn HelpersPage() -> Element {
    let mut out = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            Button {
                id: "run",
                onclick: move |_| {
                    spawn(async move {
                        let see = file_from_bytes("see.png", "image/png", SEE_THROUGH.to_vec());
                        let flat = file_from_bytes("flat.png", "image/png", OPAQUE.to_vec());
                        let (Some(see), Some(flat)) = (see, flat) else {
                            out.set("no files".to_string());
                            return;
                        };
                        let rect = CropRect { x: -0.5, y: 0.0, width: 2.0, height: 1.0 };
                        out.set(format!(
                            "{:?} {:?} {} {}",
                            has_transparency(&see).await,
                            has_transparency(&flat).await,
                            size(on_square(&flat, 0.5, Backdrop::default()).await).await,
                            size(crop_image(&flat, rect, None, Backdrop::rgb(255, 255, 255)).await).await,
                        ));
                    });
                },
                "Run"
            }
            p { id: "helpers", "{out}" }
        }
    }
}
