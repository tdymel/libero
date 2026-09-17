use dioxus::prelude::*;
use libero::components::{Alert, Anchor, Text};

use crate::GITHUB;

/// Where the project stands, said plainly.
#[component]
pub fn Status() -> Element {
    rsx! {
        Alert { title: "Before 1.0",
            Text {
                "Libero has not reached 1.0, and its APIs can still change between releases. "
                "Progress and open issues are on "
                Anchor { to: GITHUB, target: "_blank", "GitHub" }
                "."
            }
        }
    }
}
