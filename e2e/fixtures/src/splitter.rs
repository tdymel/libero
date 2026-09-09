//! `Splitter`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Splitter, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/splitter", || rsx! { SplitterPage {} })];

/// A `Splitter` between two buttons, so focus has somewhere to be before a
/// drag and a Shift+Tab never sits at the document edge.
#[component]
fn SplitterPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            div { style: "height: 160px",
                Splitter {
                    initial_size: 50.0,
                    aria_label: "Resize panes",
                    panel_a: rsx! { Text { "Pane A" } },
                    panel_b: rsx! { Text { "Pane B" } },
                }
            }
            Button { id: "after", "After" }
        }
    }
}
