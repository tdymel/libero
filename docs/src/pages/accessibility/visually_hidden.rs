use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Anchor, Code, Text, VisuallyHidden};

/// Prints the link the hidden text sits in, since the point is what it's read after.
/// `focusable` prints a skip link instead.
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
                prop("children", "Element")
                    .default("required")
                    .doc("The screen-reader-only content."),
            ])],
            accessibility: a11y()
                .handles(["With `focusable`, the content shows while focus is inside it."])
                .must([
                    "Place the text where it should be read, inside the link and not next to it.",
                    "Use it for text a screen reader user is missing, never to hide something sighted users need. To replace a control's whole name, use `aria_label` instead.",
                    "Keep the children to text unless `focusable` is set, or keyboard focus lands somewhere invisible.",
                    "A skip link shows at its place in the flow, so put it first on the page. Further down, put it in a positioned parent and set `position: absolute` through `sx`, or Tab never scrolls it into view.",
                ]),
            lead: rsx! {
                Text {
                    "Content for screen readers only, such as extra context for a link that "
                    "is vague on its own. The preview shows \"Read more\", and a screen reader "
                    "reads \"Read more about focus management\". With "
                    Code { source: "focusable" }
                    " on it is a skip link. Tab into the preview and it shows itself."
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
