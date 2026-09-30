//! A `Scroller` of buttons, narrower than its content, left to right and under
//! `dir="rtl"`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Scroller};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/scroller", || rsx! { ScrollerPage {} }),
    ("/scroller/rtl", || rsx! { ScrollerPage { rtl: true } }),
    ("/scroller/plain", || rsx! { PlainPage {} }),
];

/// Plain text, nothing focusable: `#wide` overflows, `#narrow` fits.
#[component]
fn PlainPage() -> Element {
    rsx! {
        button { id: "before", "Before" }
        div { style: "width: 300px",
            Scroller { id: "wide", aria_label: "Wide",
                Flex { direction: "row", gap: "sm", wrap: false,
                    for i in 0..12 {
                        span { key: "{i}", style: "white-space: nowrap", "Word {i}" }
                    }
                }
            }
            Scroller { id: "narrow", aria_label: "Narrow",
                Flex { direction: "row", gap: "sm", wrap: false,
                    span { "One" }
                    span { "Two" }
                }
            }
        }
        button { id: "after", "After" }
    }
}

#[component]
fn ScrollerPage(#[props(default)] rtl: bool) -> Element {
    rsx! {
        div { dir: if rtl { "rtl" } else { "ltr" },
            button { id: "before", "Before" }
            div { style: "width: 300px",
                Scroller { id: "strip", aria_label: "Tags", draggable: rtl,
                    Flex { direction: "row", gap: "sm",
                        for i in 0..12 {
                            Button { key: "{i}", id: "tag-{i}", size: "xs", variant: "outlined", "Tag {i}" }
                        }
                    }
                }
            }
            button { id: "after", "After" }
        }
    }
}
