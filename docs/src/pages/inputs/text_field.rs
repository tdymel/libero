use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Text, TextField};

#[component]
pub fn TextFieldPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        DocPage {
            title: "TextField",
            source: "libero/src/components/inputs/text_field",
            markdown: "/md/text_field.md",
            properties: vec![
                props("TextField", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("value", "Option<String>")
                        .doc("The text in the field; strictly controlled. `None` is the empty field."),
                    prop("onchange", "EventHandler<String>")
                        .doc("Called per keystroke with the text the field should hold next."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the field."),
                    prop("label", "String")
                        .doc("The field's own caption, above the control. Names the field through a `for`/`id` pair."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A single-line text field. Strictly controlled: "
                    Code { source: "value" }
                    " is what it shows, and "
                    Code { source: "onchange" }
                    " fires per keystroke with the text it should hold next - so the caller "
                    "can rewrite or reject input instead of racing the DOM for it. There is "
                    "no uncontrolled mode."
                }
            },
            Demo {
                component: "TextField",
                children_text: "",
                fixed: vec![
                    "value: value()".to_string(),
                    "onchange: move |next| value.set(next)".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    // The caption is a whole element, not a style: unset,
                    // there is no `<label>` above the field at all.
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Name\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Ada Lovelace\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    TextField {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Name".to_string()),
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "Ada Lovelace".to_string()),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        value: value(),
                        onchange: move |next| value.set(next),
                    }
                },
            }
        }
    }
}
