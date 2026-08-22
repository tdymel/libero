use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Option as SelectOption, Select, Text};

/// The options are the fixture - the props are on the select itself - so the
/// code block prints them verbatim.
const CHILDREN: &str = r#"SelectOption { value: "xs", "Extra small" }
SelectOption { value: "sm", "Small" }
SelectOption { value: "md", "Medium" }
SelectOption { value: "lg", "Large" }
SelectOption { value: "xl", "Extra large" }"#;

#[component]
pub fn SelectPage() -> Element {
    let mut value = use_signal(|| "sm".to_string());

    rsx! {
        DocPage {
            title: "Select",
            lead: rsx! {
                Text {
                    "A styled native select, wrapped in its own "
                    Code { source: "label" }
                    " element when "
                    Code { source: "label" }
                    " is set. Strictly controlled: "
                    Code { source: "value" }
                    " drives it, "
                    Code { source: "onchange" }
                    " reports what the user picked."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Select",
                    children_text: "",
                    children_code: CHILDREN,
                    fixed: vec![
                        "value: value()".to_string(),
                        "onchange: move |v| value.set(v)".to_string(),
                    ],
                    controls: vec![
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                        Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("sm"),
                        // The label is a whole element, not a style: with it
                        // unset there is no `<span>` above the select at all.
                        Control::switch("label").default("true").code(|_, values| {
                            match values.str("label").as_str() {
                                "true" => vec!["label: \"Size\"".to_string()],
                                _ => vec![],
                            }
                        }),
                        Control::switch("disabled"),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Select {
                            size: values.str("size"),
                            radius: values.str("radius"),
                            label: (values.str("label") == "true").then(|| "Size".to_string()),
                            disabled: (values.str("disabled") == "true").then_some(true),
                            value: value(),
                            onchange: move |v| value.set(v),
                            SelectOption { value: "xs", "Extra small" }
                            SelectOption { value: "sm", "Small" }
                            SelectOption { value: "md", "Medium" }
                            SelectOption { value: "lg", "Large" }
                            SelectOption { value: "xl", "Extra large" }
                        }
                    },
                }
            }
        }
    }
}
