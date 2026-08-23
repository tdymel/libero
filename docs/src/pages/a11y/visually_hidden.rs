use crate::components::{Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Anchor, Text, VisuallyHidden};

/// The point is what the hidden text is read *after*, so the link it sits in
/// is part of the example - and the code block prints it.
fn wrap_link(_: &DemoValues, code: &str) -> String {
    format!(
        "Text {{\n    Anchor {{\n        to: \"https://example.com\",\n        \"Read more\"\n{}    }}\n}}",
        indent(&indent(code))
    )
}

#[component]
pub fn VisuallyHiddenPage() -> Element {
    rsx! {
        DocPage {
            title: "Visually Hidden",
            properties: vec![props("VisuallyHidden", vec![
                prop("children", "Element").doc("The screen-reader-only content."),
            ])],
            lead: rsx! {
                Text {
                    "Content available to screen readers but removed from sighted layout - "
                    "e.g. extra context for a link that's ambiguous out of context. It takes "
                    "no props, so the demo below has nothing to vary: the preview reads "
                    "\"Read more\", a screen reader reads \"Read more about focus management\"."
                }
            },
            Demo {
                component: "VisuallyHidden",
                children_text: " about focus management",
                controls: vec![],
                render: move |_: DemoValues| rsx! {
                    Text {
                        Anchor {
                            to: "https://example.com",
                            "Read more"
                            VisuallyHidden { " about focus management" }
                        }
                    }
                },
                wrap: Wrap(wrap_link),
            }
        }
    }
}
