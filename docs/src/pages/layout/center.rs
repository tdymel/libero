use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Center, Code, Text},
    sx::sx,
};

/// The child is the fixture - `inline` is the only prop - so the code block
/// prints it verbatim.
const CHILD: &str = r#"Box { sx: sx().padding("8px 16px").background("primary").color("primary-contrast"), "Centered" }"#;

/// No width of its own: filling the parent - or not - is what `inline`
/// decides, so the wrapper below is what supplies the width.
// snippet: in Center { .. }
const SX: &str = r#"sx: sx().height("120px").background("primary.1")"#;

/// A printed wrapper gives `Center` a width: the shrink-wrapping preview makes both modes alike.
fn wrap_parent(_: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().width(\"260px\").background(\"muted.2\"),\n{}}}",
        indent(code)
    )
}

#[component]
pub fn CenterPage() -> Element {
    rsx! {
        DocPage {
            title: "Center",
            source: "libero/src/components/layout/center.rs",
            markdown: "/md/center.md",
            properties: vec![props("Center", vec![
                prop("inline", "bool")
                    .default("false")
                    .doc("Shrinks to the child instead of filling the parent's width."),
                prop("children", "Element").doc("The centered content."),
            ])],
            lead: rsx! {
                Text {
                    "Centers its child horizontally and vertically. It fills the parent's "
                    "width, the outer band here, and has no height of its own. With "
                    Code { source: "inline" }
                    " it shrinks to its child."
                }
            },
            Demo {
                component: "Center",
                children_text: "",
                children_code: CHILD.to_string(),
                fixed: vec![SX.to_string()],
                controls: vec![Control::switch("inline")],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx().width("260px").background("muted.2"),
                        Center {
                            inline: (values.str("inline") == "true").then_some(true),
                            sx: sx().height("120px").background("primary.1"),
                            Box {
                                sx: sx()
                                    .padding("8px 16px")
                                    .background("primary")
                                    .color("primary-contrast"),
                                "Centered"
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_parent),
            }
        }
    }
}
