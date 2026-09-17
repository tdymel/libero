use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Code, Table, Text, column},
    sx::sx,
};

const APIS: [(&str, &str); 6] = [
    ("timer()", "Run a callback after a delay or on an interval."),
    (
        "keyboard()",
        "Hear key presses anywhere in the document, for shortcuts.",
    ),
    ("scroll()", "Hear anything scrolling, not only the page."),
    (
        "document()",
        "Read the focused element and the viewport size.",
    ),
    ("clock()", "Today's date in the user's time zone."),
    (
        "use_element()",
        "Focus, scroll and measure one element you mount.",
    ),
];

#[component]
pub fn PlatformPage() -> Element {
    rsx! {
        DocPage {
            title: "Platform",
            markdown: "/md/platform.md",
            lead: rsx! {
                Text {
                    "Everything that reaches past dioxus to the machine goes through "
                    Code { source: "libero::platform" }
                    ". Components use it, and so can you."
                }
            },

            DocSection {
                title: "The APIs",
                Table {
                    aria_label: "Platform APIs",
                    data: APIS.to_vec(),
                    columns: vec![
                        column("API")
                            .value(|row: &(&str, &str)| row.0)
                            .render(|row: &(&str, &str)| rsx! { Code { source: row.0, sx: sx().white_space("nowrap") } }),
                        column("What it's for").value(|row: &(&str, &str)| row.1),
                    ],
                }
            }

            DocSection {
                title: "Using them",
                Text {
                    "Each accessor returns an "
                    Code { source: "Option" }
                    ". "
                    Code { source: "None" }
                    " means the running renderer can't do it, so branch once and let the "
                    "feature be absent there instead of unwrapping."
                }
                Text {
                    "A callback API hands back a subscription, and dropping it stops the "
                    "callback. Keep it in a signal for as long as the component lives. The "
                    "callback runs outside every scope, so write what it learns into a signal."
                }
                Text {
                    "Element reads such as "
                    Code { source: "dimensions()" }
                    " are futures. Start one in an event handler and await it in a "
                    Code { source: "spawn" }
                    ". A renderer that can't serve one answers "
                    Code { source: "PlatformError::Unsupported" }
                    "."
                }
            }
        }
    }
}
