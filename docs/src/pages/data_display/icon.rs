use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Icon, Input, Text};

use crate::icons::CheckmarkIcon;

/// The svg the badge wraps - a subtree, so the code block prints it verbatim.
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
                        .doc("Chrome around the svg, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. A badge is not interactive, so it takes no hover response."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Sets the container's CSS color, which a currentColor svg then inherits. A theme color also tints the background under variant filled."),
                    prop("size", "ThemeAwareValue")
                        .default("md")
                        .doc("Badge width and height."),
                    prop("radius", "ThemeAwareValue")
                        .default("sm")
                        .doc("Corner radius of the badge."),
                    prop("children", "Element").doc("The svg to badge."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Wraps an svg child in a sized, colored badge. "
                    Code { source: "color" }
                    " sets the container's CSS color, which any child svg using "
                    Code { source: "currentColor" }
                    " for its fill/stroke then inherits."
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
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::toggle("component", ["span", "div"]),
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
        }
    }
}
