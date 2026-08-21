use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    sx::sx,
};

#[component]
pub fn ButtonPage() -> Element {
    rsx! {
        DocPage {
            title: "Button",
            lead: rsx! {
                Text {
                    "A clickable control, or a router-aware link when "
                    Code { source: "to" }
                    " is set."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Button",
                    children_text: "Save changes",
                    controls: vec![
                        Control::color(
                            "color",
                            ["primary", "secondary", "success", "error", "warning", "info"],
                        ),
                        Control::toggle("variant", ["outlined", "filled", "text"]),
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                        Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                        Control::switch("full_width"),
                        Control::switch("selected"),
                        Control::switch("disabled"),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Button {
                            color: values.str("color"),
                            variant: values.str("variant"),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            full_width: values.str("full_width") == "true",
                            // `Some(false)` is still a toggle button
                            // (`aria-pressed="false"`); unset is not.
                            selected: match values.str("selected").as_str() {
                                "true" => Some(true),
                                _ => None,
                            },
                            disabled: values.str("disabled") == "true",
                            "Save changes"
                        }
                    },
                }
            }
            DocSection {
                title: "Form type",
                Text {
                    sx: sx().color("grey.6"),
                    "A plain "
                    Code { source: "<button>" }
                    " submits an enclosing form. Ours defaults to "
                    Code { source: "type=\"button\"" }
                    " instead - set it yourself for a submit or reset button.",
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    Button { variant: "filled", r#type: "submit", "Submit" }
                    Button { variant: "outlined", r#type: "reset", "Reset" }
                }
            }
            DocSection {
                title: "As a link",
                Text {
                    sx: sx().color("grey.6"),
                    "Renders as a real anchor, or a router Link when to matches an internal route.",
                }
                Button {
                    variant: "outlined",
                    to: "https://dioxuslabs.com",
                    target: "_blank",
                    "Open Dioxus docs"
                }
            }
        }
    }
}
