use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Text, Textarea};

#[component]
pub fn TextareaPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        DocPage {
            title: "Textarea",
            source: "libero/src/components/form/textarea.rs",
            markdown: "/md/textarea.md",
            properties: vec![
                props("Textarea", vec![
                    prop("size", "Size").default("md").doc("Controls padding and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("rows", "u32")
                        .default("3")
                        .doc("Visible lines, which is what sets the starting height. The user can still drag it taller."),
                    prop("value", "Option<String>")
                        .doc("The text in the field. `None` leaves the `<textarea>` uncontrolled - it keeps its own text and needs no handler."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires per keystroke with the text the field should hold next."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. Names the field through a `for`/`id` pair."),
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
                    "A multi-line text field, with the same slots every field has. "
                    Code { source: "rows" }
                    " sets the starting height and the browser's own drag handle takes it "
                    "from there - the frame grows with the control, because a field's height "
                    "is its padding plus whatever the control needs."
                }
            },
            Demo {
                component: "Textarea",
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
                    // A `u32`, so it prints unquoted.
                    Control::slider("rows", ["2", "3", "5", "8"]).default("3").code(
                        |control, values| match values.str("rows") {
                            rows if rows == control.default => vec![],
                            rows => vec![format!("rows: {rows}")],
                        },
                    ),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"That is getting long.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Say something.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Notes\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Anything the team should know.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"Markdown is not rendered.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Start typing\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Textarea {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        rows: values.str("rows").parse::<u32>().unwrap_or(3),
                        label: (values.str("label") == "true").then(|| "Notes".to_string()),
                        description: (values.str("description") == "true")
                            .then(|| "Anything the team should know.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "Markdown is not rendered.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("That is getting long.".to_string()),
                            "error" => FieldStatus::Error("Say something.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "Start typing".to_string()),
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
