use dioxus::prelude::*;
use libero::{
    components::{Alert, Anchor, Text},
    sx::sx,
};

use crate::GITHUB;

/// Where the project stands, said plainly.
#[component]
pub fn Status() -> Element {
    rsx! {
        Alert { title: "Before 1.0",
            Text {
                "Libero has not reached 1.0, and its APIs can still change between releases. "
                "Progress and open issues are on "
                // The page's link colour falls short on the tint; the alert's text reads.
                Anchor {
                    to: GITHUB,
                    target: "_blank",
                    underline: "always",
                    sx: sx().color("inherit"),
                    "GitHub"
                }
                "."
            }
        }
    }
}
