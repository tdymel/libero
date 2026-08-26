use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, or_unset, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Float, Text},
    sx::sx,
    use_theme,
};

const CHILD: &str = r#"Box { sx: sx().padding("4px 8px").background("primary"), "Badge" }"#;

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
                    .default("center-center")
                    .doc("Anchor corner/edge, e.g. `\"top-start\"`."),
                prop("offset_x", "ThemeAwareValue")
                    .default("0px")
                    .doc("Shift along the horizontal axis - a size token from the spacing scale (`\"md\"`, or `\"-md\"` for the other direction), or any CSS length."),
                prop("offset_y", "ThemeAwareValue")
                    .default("0px")
                    .doc("Shift along the vertical axis."),
                prop("z_index", "ThemeAwareValue")
                    .default("200")
                    .doc("Stacking order."),
                prop("children", "Element").doc("The anchored content."),
            ])],
            lead: rsx! {
                Text { "Anchors its child to a corner/edge of the nearest `position: relative` ancestor - e.g. a badge on an avatar. The parent must set `position: relative` itself. `offset_x`/`offset_y` take a size token from the spacing scale, or any CSS length, and shift it right/down along the page axes - negate the token (`\"-md\"`) to shift left/up instead." }
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
                            Box { sx: sx().padding("4px 8px").background("primary"), "Badge" }
                        }
                    }
                },
                wrap: Wrap(wrap_anchor),
            }
        }
    }
}
