//! `AspectRatio`.

use dioxus::prelude::*;
use libero::components::AspectRatio;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/aspect-ratio-link", || rsx! { LinkPage {} }),
    ("/aspect-ratio-children", || rsx! { ChildrenPage {} }),
];

/// Todos 2532, 2535: a replaced child takes no `::after`, a positioned one must stay put on focus.
#[component]
fn ChildrenPage() -> Element {
    rsx! {
        div { style: "width: 320px; display: flex; flex-direction: column; gap: 16px",
            // First in the tab order: Tab walks into the video's own controls.
            AspectRatio { id: "pinned-ratio", ratio: 16.0 / 9.0,
                a {
                    id: "pinned",
                    href: "#here",
                    style: "position: absolute; top: 8px; left: 8px; width: 50%; height: 50%",
                    "Pinned"
                }
            }
            AspectRatio { id: "video-ratio", ratio: 16.0 / 9.0,
                video { id: "video", controls: true, tabindex: "0", style: "background: #369" }
            }
        }
    }
}

const PICTURE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 4 3'%3E%3Crect width='4' height='3' fill='%23369'/%3E%3C/svg%3E";

/// Todo 2480: a link holding a picture, which paints over the link's inset ring.
#[component]
fn LinkPage() -> Element {
    rsx! {
        div { style: "width: 320px",
            AspectRatio { id: "ratio", ratio: 4.0 / 3.0,
                a { id: "link", href: "#here",
                    img { src: PICTURE, alt: "A blue field", style: "display: block; width: 100%; height: 100%" }
                }
            }
        }
    }
}
