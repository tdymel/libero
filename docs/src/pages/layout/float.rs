use crate::components::{
    Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Float, Text},
    sx::sx,
    use_theme,
};

const CHILD: &str = r#"Box { sx: sx().padding("4px 8px").background("primary").color("primary-contrast"), "Badge" }"#;

/// A float positions against the nearest `position: relative` ancestor, so
/// the preview has to be one - and the code block has to say so.
fn wrap_anchor(_: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().position(\"relative\").width(\"160px\").height(\"120px\").background(\"primary.1\"),\n{}}}",
        indent(code)
    )
}

#[component]
pub fn FloatPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Float",
            source: "libero/src/components/layout/float.rs",
            markdown: "/md/float.md",
            properties: vec![props("Float", vec![
                prop("placement", "Placement")
                    .default(theme.float.placement.as_str())
                    .doc("The corner or edge to anchor to, such as `\"top-start\"`."),
                prop("offset_x", "ThemeAwareValue")
                    .default(theme.float.offset_x)
                    .doc("Shift to the right, a spacing step or a CSS length. A negative step like `\"-md\"` shifts left."),
                prop("offset_y", "ThemeAwareValue")
                    .default(theme.float.offset_y)
                    .doc("Shift down, a spacing step or a CSS length. A negative step shifts up."),
                prop("fixed", "bool")
                    .default("false")
                    .doc("Anchors to the viewport instead of the parent, so it stays put while the page scrolls. An ancestor with a `transform`, `filter`, `contain` or `container-type` still captures it."),
                prop("z_index", "ThemeAwareValue")
                    .default("200")
                    .doc("Stacking order."),
                prop("children", "Element").doc("The anchored content."),
            ])],
            accessibility: a11y()
                .handles(["`Float` adds no roles, and `start` and `end` follow the text direction."])
                .must([
                    "Write the float next to what it marks: screen readers and Tab follow the code, not where it shows.",
                    "Keep it off text and controls at 320px wide and at 200% text size: it takes no space, so nothing moves out of its way.",
                ]),
            lead: rsx! {
                Text {
                    "Anchors its child to a corner or edge of the nearest positioned "
                    "ancestor, like a badge on an avatar. The parent sets "
                    Code { source: "position: relative" }
                    " itself. The offsets follow the page, not the placement, so on a "
                    Code { source: "top-end" }
                    " badge a positive "
                    Code { source: "offset_x" }
                    " and a negative "
                    Code { source: "offset_y" }
                    " hang it off the corner."
                }
            },
            Demo {
                component: "Float",
                children_text: "",
                children_code: CHILD.to_string(),
                controls: vec![
                    Control::select(
                        "placement",
                        [
                            "top-start",
                            "top-center",
                            "top-end",
                            "center-start",
                            "center-center",
                            "center-end",
                            "bottom-start",
                            "bottom-center",
                            "bottom-end",
                        ],
                    )
                    .labels([
                        "Top start",
                        "Top center",
                        "Top end",
                        "Center start",
                        "Center",
                        "Center end",
                        "Bottom start",
                        "Bottom center",
                        "Bottom end",
                    ])
                    .default(theme.float.placement.as_str()),
                    Control::slider(
                        "offset_x",
                        ["-lg", "-md", "-sm", "-xs", "auto", "xs", "sm", "md", "lg"],
                    )
                    .default("auto"),
                    Control::slider(
                        "offset_y",
                        ["-lg", "-md", "-sm", "-xs", "auto", "xs", "sm", "md", "lg"],
                    )
                    .default("auto"),
                    // Leaves the frame for the window's own corner.
                    Control::switch("fixed"),
                ],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx()
                            .position("relative")
                            .width("160px")
                            .height("120px")
                            .background("primary.1"),
                        Float {
                            placement: values.str("placement"),
                            offset_x: or_unset(values.str("offset_x")),
                            offset_y: or_unset(values.str("offset_y")),
                            fixed: values.str("fixed") == "true",
                            Box {
                                sx: sx().padding("4px 8px").background("primary").color("primary-contrast"),
                                "Badge"
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_anchor),
            }
        }
    }
}
