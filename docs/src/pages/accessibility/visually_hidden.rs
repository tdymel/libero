use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Anchor, Text, VisuallyHidden};

/// The point is what the hidden text is read *after*, so the link it sits in
/// is part of the example - and the code block prints it. `focusable` is the
/// other use, a skip link, so it prints that instead.
fn wrap_link(values: &DemoValues, code: &str) -> String {
    if values.str("focusable") == "true" {
        return "VisuallyHidden {\n    focusable: true,\n    Anchor { to: \"#main\", \"Skip to content\" }\n}"
            .to_string();
    }
    format!(
        "Text {{\n    Anchor {{\n        to: \"https://example.com\",\n        \"Read more\"\n{}    }}\n}}",
        indent(&indent(code))
    )
}

#[component]
pub fn VisuallyHiddenPage() -> Element {
    rsx! {
        DocPage {
            title: "VisuallyHidden",
            source: "libero/src/components/accessibility/visually_hidden.rs",
            markdown: "/md/visually_hidden.md",
            properties: vec![props("VisuallyHidden", vec![
                prop("focusable", "bool")
                    .default("false")
                    .doc("Shows the content while focus is inside it, for a skip link."),
                prop("children", "Element").doc("The screen-reader-only content."),
            ])],
            lead: rsx! {
                Text {
                    "Content available to screen readers but removed from sighted layout - "
                    "e.g. extra context for a link that's ambiguous out of context. The preview "
                    "reads \"Read more\", a screen reader reads \"Read more about focus "
                    "management\". With focusable on it is a skip link: Tab into the preview "
                    "and it shows itself."
                }
            },
            Demo {
                component: "VisuallyHidden",
                children_text: " about focus management",
                controls: vec![Control::switch("focusable")],
                render: move |values: DemoValues| if values.str("focusable") == "true" {
                    rsx! {
                        VisuallyHidden { focusable: true,
                            Anchor { to: "#main", "Skip to content" }
                        }
                    }
                } else { rsx! {
                    Text {
                        Anchor {
                            to: "https://example.com",
                            "Read more"
                            VisuallyHidden { " about focus management" }
                        }
                    }
                } },
                wrap: Wrap(wrap_link),
            }
        }
    }
}
