use crate::components::{
    Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Float, Input, Text},
    sx::sx,
    use_theme,
};

/// Clears a sticky `Header` that publishes its height; `0px` without one.
const BELOW_HEADER: &str = "var(--lsx-header-height, 0px)";

/// A fixed top float goes under the page's sticky header unless pushed below it.
fn header_offset(values: &DemoValues) -> Option<&'static str> {
    let clears = values.str("fixed") == "true"
        && values.str("placement").starts_with("top")
        && values.str("offset_y") == "auto";
    clears.then_some(BELOW_HEADER)
}

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
                    .doc("Anchors to the viewport instead of the parent, so it stays put while the page scrolls. An ancestor with a `transform`, `filter`, `contain` or `container-type` still captures it. It stays below overlays and modals."),
                prop("z_index", "ThemeAwareValue")
                    .default("1")
                    .doc("Stacking order. The default sits below a sticky `Header`; raise it to go over one."),
                prop("children", "Element").doc("The anchored content."),
            ])],
            accessibility: a11y()
                .handles(["`Float` adds no roles, and `start` and `end` follow the text direction."])
                .must([
                    "Write the float next to what it marks: screen readers and Tab follow the code, not where it shows.",
                    "Keep it off text and controls at 320px wide and at 200% text size: it takes no space, so nothing moves out of its way.",
                    "Keep a `fixed` float off the control that has focus, or the focus ring is hidden behind it (WCAG 2.4.11).",
                    "Put a notification stack in a status region (`role=\"status\"`), or its messages are not announced (WCAG 4.1.3).",
                ])
                .example("A \"New\" badge floated over a card's corner, written right after the card's title in the code: a screen reader reads \"Spring sale, New\", in that order."),
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
                    " hang it off the corner in left-to-right text. Under "
                    Code { source: "dir=rtl" }
                    " that corner is at the left edge, so use a negative "
                    Code { source: "offset_x" }
                    ". For a button floating over the page, use "
                    Code { source: "FloatingActionButton" }
                    "."
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
                    .default("auto")
                    .code(|_, values| match (header_offset(values), values.str("offset_y")) {
                        (Some(offset), _) => vec![format!("offset_y: {offset:?}")],
                        (None, value) if value == "auto" => vec![],
                        (None, value) => vec![format!("offset_y: {value:?}")],
                    }),
                    // Leaves the frame for the window's own corner, below the docs header.
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
                            offset_y: match header_offset(&values) {
                                Some(offset) => Input::from(offset.to_string()),
                                None => or_unset(values.str("offset_y")),
                            },
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
