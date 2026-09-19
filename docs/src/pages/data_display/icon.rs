use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Icon, Input, Text};

use crate::icons::CheckmarkIcon;

/// The svg the box wraps. A subtree, so the code block prints it verbatim.
const CHILDREN: &str = "CheckmarkIcon {}";

#[component]
pub fn IconPage() -> Element {
    rsx! {
        DocPage {
            title: "Icon",
            source: "libero/src/components/data_display/icon.rs",
            markdown: "/md/icon.md",
            properties: vec![
                props("Icon", vec![
                    prop("component", "HtmlTag").default("span").doc("Element to render as."),
                    prop("variant", "Variant")
                        .default("filled")
                        .doc("The look, shared with `Button`. An icon is not interactive, so it has no hover state."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The CSS color, which an svg drawn in `currentColor` inherits. Under `filled` a theme color also tints the background."),
                    prop("size", "ThemeAwareValue")
                        .default("md")
                        .doc("Width and height."),
                    prop("radius", "ThemeAwareValue")
                        .default("sm")
                        .doc("Corner radius."),
                    prop("children", "Element").default("required").doc("The svg."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Wraps an svg in a sized, colored box. An svg drawn in "
                    Code { source: "currentColor" }
                    " takes the box's "
                    Code { source: "color" }
                    ". With "
                    Code { source: "standard" }
                    " the svg fills the box. Every other variant insets it, clear of the "
                    "box's edges."
                }
            },
            Demo {
                component: "Icon",
                children_text: "",
                children_code: CHILDREN,
                controls: vec![
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color("color"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    // `standard` draws no box, so there is no corner to round.
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm")
                        .hidden_when(|values| values.str("variant") == "standard"),
                    Control::toggle("component", ["span", "div"]).labels(["Span", "Div"]),
                ],
                render: move |values: DemoValues| rsx! {
                    Icon {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        component: values.str("component"),
                        CheckmarkIcon {}
                    }
                },
            }
            DocSection { title: "Accessibility",
                Text {
                    "An icon is hidden from screen readers. One that means something needs "
                    Code { source: "aria_label" }
                    ", which makes it "
                    Code { source: "role=\"img\"" }
                    ". A "
                    Code { source: "<title>" }
                    " inside the svg does not name it, since it is hidden with the rest. "
                    "For a clickable icon, use "
                    Code { source: "ActionIcon" }
                    "."
                }
            }
        }
    }
}
