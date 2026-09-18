use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
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
                    prop("size", "Size").default("md").doc("Padding and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of `size`."),
                    prop("rows", "u32")
                        .default("3")
                        .doc("Visible lines, which set the starting height. The user can still drag it taller."),
                    prop("value", "Option<String>")
                        .doc("The text in the field. Leave it out and the textarea keeps its own text."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires on every keystroke with the text the field should hold next."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the text, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<String>")
                        .doc("What the field posts as. A path such as `Signup::FIELDS.bio()` also binds the text to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("counter", "bool")
                        .default("false")
                        .doc("Shows `12/200` under the control while a `maxlength` attribute is set. It counts as `maxlength` does, so an emoji counts two."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. It names the field."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. Formatting rules or limits."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables and dims the field."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post."),
                ]).extends("textarea"),
            ],
            lead: rsx! {
                Text {
                    "A multi-line text field with the same slots as "
                    Code { source: "TextField" }
                    ". "
                    Code { source: "rows" }
                    " sets the starting height, and the user can drag it taller. With server "
                    "rendering, the box stays empty until the app hydrates."
                }
            },
            // snippet: let mut value = use_signal(String::new);
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
                        .labels(["Valid", "Warning", "Error"])
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
                            _ => vec!["aria_label: \"Notes\"".to_string()],
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
                    Control::switch("counter").code(|_, values| {
                        match values.str("counter").as_str() {
                            "true" => vec!["counter: true".to_string(), "maxlength: 200".to_string()],
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
                        aria_label: (values.str("label") != "true").then_some("Notes"),
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
                        counter: values.str("counter") == "true",
                        maxlength: (values.str("counter") == "true").then_some(200),
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                },
            }
            DocSection { title: "Accessibility",
                Text {
                    "Leave "
                    Code { source: "label" }
                    " unset only when something else names the field. The visible counter is "
                    "hidden from screen readers. Instead, a polite status says how many characters "
                    "are left once a tenth of the limit remains. Its words come from the "
                    "localization's "
                    Code { source: "textarea.characters_left" }
                    "."
                }
            }
        }
    }
}
