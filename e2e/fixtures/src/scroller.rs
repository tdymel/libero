//! A `Scroller` of buttons, narrower than its content, left to right and under
//! `dir="rtl"`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Scroller};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/scroller", || rsx! { ScrollerPage {} }),
    ("/scroller/rtl", || rsx! { ScrollerPage { rtl: true } }),
];

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
