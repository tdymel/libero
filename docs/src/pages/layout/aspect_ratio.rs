use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{AspectRatio, Code, Flex, Text},
    sx::sx,
};

/// The child is the fixture - `ratio` is the only prop - so the code block
/// prints it verbatim.
const CHILD: &str = r#"Flex {
    sx: sx().background("primary").color("surface"),
    align: "center",
    justify: "center",
    "The child fills the box"
}"#;

/// A ratio only shows against a known width; the box has no size of its own.
// snippet: in AspectRatio { .. }
const SX: &str = r#"sx: sx().width("240px")"#;

#[component]
pub fn AspectRatioPage() -> Element {
    rsx! {
        DocPage {
            title: "AspectRatio",
            source: "libero/src/components/layout/aspect_ratio.rs",
            markdown: "/md/aspect_ratio.md",
            properties: vec![props("AspectRatio", vec![
                prop("ratio", "f32")
                    .default("1.0")
                    .doc("Width-to-height ratio, e.g. `16.0 / 9.0`."),
                prop("children", "Element").doc("The child to crop, filling the box."),
            ])],
            lead: rsx! {
                Text {
                    "Enforces a width-to-height ratio on its child, cropping it to fill the "
                    "box. "
                    Code { source: "ratio" }
                    " is a plain "
                    Code { source: "f32" }
                    " - write it as the division it reads as - and defaults to the theme's "
                    Code { source: "1.0" }
                    "."
                }
            },
            Demo {
                component: "AspectRatio",
                children_text: "",
                children_code: CHILD.to_string(),
                fixed: vec![SX.to_string()],
                controls: vec![
                    // The options are the rsx expressions themselves, so
                    // the code block prints an unquoted `f32`.
                    Control::slider(
                        "ratio",
                        ["3.0 / 4.0", "1.0", "4.0 / 3.0", "16.0 / 9.0", "21.0 / 9.0"],
                    )
                    .labels(["3 / 4", "1 / 1", "4 / 3", "16 / 9", "21 / 9"])
                    .default("1.0")
                    .code(|control, values| {
                        let value = values.str("ratio");
                        match value == control.default {
                            true => vec![],
                            false => vec![format!("ratio: {value}")],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    AspectRatio {
                        ratio: match values.str("ratio").as_str() {
                            "3.0 / 4.0" => 3.0 / 4.0,
                            "4.0 / 3.0" => 4.0 / 3.0,
                            "16.0 / 9.0" => 16.0 / 9.0,
                            "21.0 / 9.0" => 21.0 / 9.0,
                            _ => 1.0,
                        },
                        sx: sx().width("240px"),
                        Flex {
                            sx: sx().background("primary").color("surface"),
                            align: "center",
                            justify: "center",
                            "The child fills the box"
                        }
                    }
                },
            }
        }
    }
}
