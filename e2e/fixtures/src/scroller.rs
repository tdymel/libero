//! A `Scroller` of buttons, narrower than its content.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Scroller};

use crate::Routes;

pub const ROUTES: Routes = &[("/scroller", || rsx! { ScrollerPage {} })];

#[component]
fn ScrollerPage() -> Element {
    rsx! {
        button { id: "before", "Before" }
        div { style: "width: 300px",
            Scroller { id: "strip", aria_label: "Tags",
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
