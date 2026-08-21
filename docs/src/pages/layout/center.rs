use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent};
use dioxus::prelude::*;
use libero::{
    components::{Box, Center, Code, Text},
    sx::sx,
};

/// The child is the fixture - `inline` is the only prop - so the code block
/// prints it verbatim.
const CHILD: &str = r#"Box { sx: sx().padding("8px 16px").background("primary"), "Centered" }"#;

/// No width of its own: filling the parent - or not - is what `inline`
/// decides, so the wrapper below is what supplies the width.
const SX: &str = r#"sx: sx().height("120px").background("primary.1")"#;

/// The preview pane centers and shrink-wraps whatever it holds, so a bare
/// `Center` would have no width to fill and both modes would look alike. The
/// wrapper gives it one, and the code block prints it.
fn wrap_parent(_: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().width(\"260px\").background(\"grey.2\"),\n{}}}",
        indent(code)
    )
}

#[component]
pub fn CenterPage() -> Element {
    rsx! {
        DocPage {
            title: "Center",
            lead: rsx! {
                Text {
                    "Centers its child both horizontally and vertically. "
                    Code { source: "inline" }
                    " switches it from "
                    Code { source: "flex" }
                    " to "
                    Code { source: "inline-flex" }
                    ", so it shrinks to its child instead of filling the parent's width - "
                    "the grey band is the parent."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Center",
                    children_text: "",
                    children_code: CHILD.to_string(),
                    fixed: vec![SX.to_string()],
                    controls: vec![Control::switch("inline")],
                    render: move |values: DemoValues| rsx! {
                        Box {
                            sx: sx().width("260px").background("grey.2"),
                            Center {
                                inline: (values.str("inline") == "true").then_some(true),
                                sx: sx().height("120px").background("primary.1"),
                                Box {
                                    sx: sx().padding("8px 16px").background("primary"),
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
}
