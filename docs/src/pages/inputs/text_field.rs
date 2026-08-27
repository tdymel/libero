use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Icon, Text, TextField};

#[component]
pub fn TextFieldPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        DocPage {
            title: "TextField",
            source: "libero/src/components/form/text_field.rs",
            markdown: "/md/text_field.md",
            properties: vec![
                props("TextField", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("value", "Option<String>")
                        .doc("The text in the field. `None` leaves the `<input>` uncontrolled - it keeps its own text and needs no handler."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires per keystroke with the text the field should hold next. Native name, native timing."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("leading", "Element")
                        .doc("Inside the frame, before the control - a search icon, a currency prefix."),
                    prop("trailing", "Element")
                        .doc("Inside the frame, after the control - a clear button, a unit."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. Names the field through a `for`/`id` pair. Takes a string or an `Element`."),
                    prop("description", "Caption")
                        .doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control: formatting rules, constraints, counters."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required, adds `aria-required` and shows an asterisk in the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the field."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A single-line text field with the five slots every field shares: label, "
                    "description, the control, helper text, and a validation message. Pass "
                    Code { source: "value" }
                    " to control it and "
                    Code { source: "oninput" }
                    " to hear about keystrokes; omit "
                    Code { source: "value" }
                    " and the input keeps its own text. Whichever caption slots are filled "
                    "are named by "
                    Code { source: "aria-describedby" }
                    " automatically. "
                    Code { source: "leading" }
                    " and "
                    Code { source: "trailing" }
                    " put content inside the border, beside the control."
                }
            },
            Demo {
                component: "TextField",
                children_text: "",
                fixed: vec![
                    "value: value()".to_string(),
                    "oninput: move |next| value.set(next)".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    // Each caption is a whole element, not a style: unset,
                    // the slot is not in the markup at all.
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Email\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"The address we send the invoice to.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"Work addresses only.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::select("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"That domain is unusual.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Not a valid address.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"ada@example.com\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("leading").code(|_, values| {
                        match values.str("leading").as_str() {
                            "true" => vec!["leading: rsx! { Icon { \"@\" } }".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("trailing").code(|_, values| {
                        match values.str("trailing").as_str() {
                            "true" => vec!["trailing: rsx! { \".com\" }".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    TextField {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Email".to_string()),
                        description: (values.str("description") == "true")
                            .then(|| "The address we send the invoice to.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "Work addresses only.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("That domain is unusual.".to_string()),
                            "error" => FieldStatus::Error("Not a valid address.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "ada@example.com".to_string()),
                        leading: (values.str("leading") == "true")
                            .then(|| rsx! { Icon { "@" } }),
                        trailing: (values.str("trailing") == "true")
                            .then(|| rsx! { ".com" }),
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                },
            }
        }
    }
}
