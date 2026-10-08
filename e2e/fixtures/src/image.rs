//! `Image`: a described picture, a decorative one, the fallback, an empty
//! fallback and the rounded zoom button.

use dioxus::prelude::*;
use libero::components::{Avatar, CropRect, Image, ImageCropper};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/image", || rsx! { ImagePage {} }),
    ("/image/missed-error", || rsx! { MissedErrorPage {} }),
];

/// A PNG that decodes to nothing: `complete` is true as soon as it mounts.
const UNDECODABLE: &str = "data:image/png;base64,bm90IGFuIGltYWdl";

/// Todo 2529: the tests swallow the `error` event, as a server-rendered page misses it before
/// hydration, so only the check at mount can show the fallback.
#[component]
fn MissedErrorPage() -> Element {
    let mut shown = use_signal(|| false);
    let mut errors = use_signal(|| 0);

    rsx! {
        // The count restarts per mount: the first one may report its failure before it unmounts.
        button {
            id: "mount",
            onclick: move |_| {
                shown.toggle();
                errors.set(0);
            },
            "Mount"
        }
        if shown() {
            div { width: "64px", height: "64px",
                Image {
                    id: "missed-image",
                    src: UNDECODABLE,
                    fallback_src: FALLBACK.to_string(),
                    alt: "A brown square",
                }
            }
            Avatar { id: "missed-avatar", name: "Ada Lovelace", initials: "AL", src: UNDECODABLE }
            ImageCropper {
                id: "missed-cropper",
                src: UNDECODABLE,
                alt: "A broken picture",
                onchange: |_: CropRect| {},
                onerror: move |()| errors += 1,
            }
            p { id: "error", "{errors}" }
        }
    }
}

const PICTURE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 4 4'%3E%3Crect width='4' height='4' fill='%23369'/%3E%3C/svg%3E";
const FALLBACK: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 4 4'%3E%3Crect width='4' height='4' fill='%23963'/%3E%3C/svg%3E";
const MISSING: &str = "/does-not-exist.png";

#[component]
fn ImagePage() -> Element {
    rsx! {
        div { display: "flex", gap: "16px", flex_wrap: "wrap",
            div { width: "64px", height: "64px",
                Image { id: "described", src: PICTURE, alt: "A blue square" }
            }
            div { width: "64px", height: "64px",
                Image { id: "decorative", src: PICTURE, decorative: true }
            }
            div { width: "64px", height: "64px",
                Image {
                    id: "fallback",
                    src: MISSING,
                    fallback_src: FALLBACK.to_string(),
                    alt: "A brown square",
                }
            }
            div { width: "64px", height: "64px",
                Image {
                    id: "empty-fallback",
                    src: MISSING,
                    fallback_src: String::new(),
                    alt: "A missing square",
                }
            }
            div { width: "64px", height: "64px",
                Image {
                    id: "zoom",
                    src: PICTURE,
                    alt: "A blue square",
                    zoomable: true,
                    loading: "lazy",
                    radius: "md",
                }
            }
        }
    }
}
