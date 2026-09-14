//! `Image`: a described picture, a decorative one, the fallback and the zoom
//! button.

use dioxus::prelude::*;
use libero::components::Image;

use crate::Routes;

pub const ROUTES: Routes = &[("/image", || rsx! { ImagePage {} })];

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
                Image { id: "decorative", src: PICTURE }
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
                    id: "zoom",
                    src: PICTURE,
                    alt: "A blue square",
                    zoomable: true,
                    "loading": "lazy",
                }
            }
        }
    }
}
