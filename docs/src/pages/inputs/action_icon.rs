use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, or_unset};
use dioxus::prelude::*;
use libero::components::{ActionIcon, Code, Input, Text};

use crate::icons::CheckmarkIcon;

/// The svg the button wraps - a subtree, so the code block prints it verbatim.
const CHILDREN: &str = "CheckmarkIcon {}";

/// `"none"` is unset, and unset is what makes the button inherit its
/// surroundings rather than draw a badge.
fn or_plain<T>(value: String) -> Input<T>
where
    Input<T>: From<String>,
{
    match value.as_str() {
        "none" => Input::None,
        _ => Input::from(value),
    }
}

#[component]
pub fn ActionIconPage() -> Element {
    rsx! {
        DocPage {
            title: "ActionIcon",
            lead: rsx! {
                Text {
                    Code { source: "Icon" }
                    "'s sizing, color, and variant system, rendered as a real "
                    Code { source: "<button>" }
                    " with click handling and required a11y - for icon-only actions like a "
                    "copy, close, or delete button. "
                    Code { source: "aria_label" }
                    " is required, not optional: an icon-only button has no visible text for "
                    "a screen reader to announce."
                }
                Text {
                    "With neither "
                    Code { source: "variant" }
                    " nor "
                    Code { source: "color" }
                    " set - the demo's "
                    Code { source: "none" }
                    " variant - it contributes no background or color of its own and inherits "
                    "the surrounding text color, rather than defaulting to a filled badge the "
                    "way "
                    Code { source: "Icon" }
                    " does. That is how "
                    Code { source: "Code" }
                    "'s own copy button is built."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "ActionIcon",
                    children_text: "",
                    children_code: CHILDREN,
                    fixed: vec!["aria_label: \"Confirm\"".to_string()],
                    controls: vec![
                        Control::toggle("variant", ["none", "filled", "outlined", "transparent"])
                            .labels(["None", "Filled", "Outlined", "Transparent"]),
                        // A bare `primary` is what an unset `color` resolves
                        // to, so that swatch prints nothing.
                        Control::color(
                            "color",
                            ["primary", "secondary", "success", "error", "warning", "info"],
                        ),
                        Control::slider("size", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                        Control::slider("radius", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                        Control::switch("disabled"),
                    ],
                    render: move |values: DemoValues| rsx! {
                        ActionIcon {
                            variant: or_plain(values.str("variant")),
                            color: match values.str("color").as_str() {
                                "primary" => Input::None,
                                color => Input::from(color),
                            },
                            size: or_unset(values.str("size")),
                            radius: or_unset(values.str("radius")),
                            disabled: values.str("disabled") == "true",
                            aria_label: "Confirm",
                            CheckmarkIcon {}
                        }
                    },
                }
            }
            DocSection {
                title: "As a link",
                Text {
                    "Renders as a real anchor, or a router Link when to matches an internal route.",
                }
                ActionIcon {
                    variant: "outlined",
                    color: "primary",
                    to: "https://dioxuslabs.com",
                    target: "_blank",
                    aria_label: "Open Dioxus docs",
                    CheckmarkIcon {}
                }
            }
        }
    }
}
