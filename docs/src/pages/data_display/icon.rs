use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, or_unset};
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
            lead: rsx! {
                Text {
                    "Wraps an svg child in a sized, colored badge. "
                    Code { source: "color" }
                    " sets the container's CSS color, which any child svg using "
                    Code { source: "currentColor" }
                    " for its fill/stroke then inherits."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Icon",
                    children_text: "",
                    children_code: CHILDREN,
                    controls: vec![
                        Control::toggle("variant", ["filled", "outlined", "transparent"]),
                        // A bare `primary` is what an unset `color` resolves
                        // to, so that swatch prints nothing.
                        Control::color(
                            "color",
                            ["primary", "secondary", "success", "error", "warning", "info"],
                        ),
                        Control::slider("size", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                        Control::slider("radius", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                        Control::toggle("component", ["span", "div"]),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Icon {
                            variant: values.str("variant"),
                            color: match values.str("color").as_str() {
                                "primary" => Input::None,
                                color => Input::from(color),
                            },
                            size: or_unset(values.str("size")),
                            radius: or_unset(values.str("radius")),
                            component: values.str("component"),
                            CheckmarkIcon {}
                        }
                    },
                }
            }
        }
    }
}
