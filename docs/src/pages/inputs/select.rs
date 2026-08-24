use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
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
            properties: vec![
                props("Select", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("value", "String").doc("The selected option's value; strictly controlled."),
                    prop("onchange", "EventHandler<String>")
                        .doc("Called with the newly picked option's value."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the select."),
                    prop("label", "String")
                        .doc("Wraps the select in a `<label>` with this text; unset renders no label element at all."),
                    prop("label_sx", "Sx").doc("Styles the label alone - the rest of `sx` lands on the wrapper."),
                    prop("children", "Element").doc("The `Option` elements to list."),
                ]),
                props("Option", vec![
                    prop("value", "String").doc("The option's value, reported to `onchange`."),
                    prop("children", "Element").doc("The option's visible label."),
                ])
                .extends("option"),
            ],
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
