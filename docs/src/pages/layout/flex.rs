use crate::components::{Control, Demo, DemoValues, DocPage, or_unset, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Text},
    sx::sx,
};

/// The three children the demo lays out - a subtree, so the code block prints
/// them verbatim rather than as a quoted child.
const CHILDREN: &str = r#"Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
Box { sx: sx().padding("8px 16px").background("primary.1"), "Three" }"#;

/// `align` and `justify` distribute *spare* space, so the flex needs a box
/// bigger than its children - its own, not a wrapper's, or the children only
/// ever fill it.
// snippet: in Flex { .. }
const BOX_SX: &str =
    r#"sx: sx().width("400px").height("200px").padding("8px").background("muted.1")"#;

fn controls() -> Vec<Control> {
    vec![
        Control::toggle("direction", ["column", "row"]),
        Control::slider("gap", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
        Control::select(
            "align",
            ["auto", "flex-start", "center", "flex-end", "stretch"],
        ),
        Control::select(
            "justify",
            [
                "auto",
                "flex-start",
                "center",
                "flex-end",
                "space-between",
                "space-around",
            ],
        ),
        Control::select("wrap", ["auto", "wrap", "nowrap"]),
    ]
}

#[component]
pub fn FlexPage() -> Element {
    rsx! {
        DocPage {
            title: "Flex",
            source: "libero/src/components/layout/flex.rs",
            markdown: "/md/flex.md",
            properties: vec![props("Flex", vec![
                prop("direction", "FlexDirection")
                    .default("column")
                    .doc("Lays the children out in a row or a column."),
                prop("align", "ThemeAwareValue")
                    .default("stretch in a column, center in a row")
                    .doc("Cross-axis alignment."),
                prop("justify", "ThemeAwareValue")
                    .default("flex-start")
                    .doc("Main-axis alignment."),
                prop("gap", "Size")
                    .default("md")
                    .doc("Space between children."),
                prop("wrap", "FlexWrap")
                    .default("nowrap in a column, wrap in a row")
                    .doc("Whether children wrap onto new lines. Also takes a `bool`."),
                prop("children", "Element").doc("The flex's children."),
            ])],
            lead: rsx! {
                Text {
                    "A flexbox container with theme-aware direction, gap, alignment and "
                    "wrapping. A row and a column each have their own theme defaults, so "
                    "a row wraps and centers its children without naming a value."
                }
            },
            Demo {
                component: "Flex",
                children_text: "",
                children_code: CHILDREN,
                fixed: vec![BOX_SX.to_string()],
                controls: controls(),
                render: move |values: DemoValues| rsx! {
                    Flex {
                        sx: sx()
                            .width("400px")
                            .height("200px")
                            .padding("8px")
                            .background("muted.1"),
                        direction: values.str("direction"),
                        gap: values.str("gap"),
                        align: or_unset(values.str("align")),
                        justify: or_unset(values.str("justify")),
                        wrap: or_unset(values.str("wrap")),
                        Box {
                            sx: sx().padding("8px 16px").background("primary.1"),
                            "One"
                        }
                        Box {
                            sx: sx().padding("8px 16px").background("primary.1"),
                            "Two"
                        }
                        Box {
                            sx: sx().padding("8px 16px").background("primary.1"),
                            "Three"
                        }
                    }
                },
            }
        }
    }
}
