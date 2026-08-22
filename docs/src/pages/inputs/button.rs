use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Text},
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
                    " is set. A plain "
                    Code { source: "<button>" }
                    " submits an enclosing form; ours defaults to "
                    Code { source: "type=\"button\"" }
                    " instead, so a submit or reset button says so."
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
                        Control::toggle("variant", ["outlined", "filled", "text"])
                            .labels(["Outlined", "Filled", "Text"]),
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                        Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                        Control::switch("full_width"),
                        Control::switch("selected"),
                        Control::switch("disabled"),
                        // A `GlobalAttributes` pass-through rather than a
                        // prop, so it prints as the raw identifier.
                        Control::toggle("type", ["button", "submit", "reset"]).code(
                            |control, values| match values.str("type") {
                                value if value == control.default => vec![],
                                value => vec![format!("r#type: {value:?}")],
                            },
                        ),
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
                            r#type: values.str("type"),
                            "Save changes"
                        }
                    },
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
