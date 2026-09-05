use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, PasswordField, Text};

#[component]
pub fn PasswordFieldPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        DocPage {
            title: "PasswordField",
            source: "libero/src/components/form/password_field.rs",
            markdown: "/md/password_field.md",
            properties: vec![
                props("PasswordField", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("value", "Option<String>")
                        .doc("The secret. `None` leaves the `<input>` uncontrolled - it keeps its own text and needs no handler."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires per keystroke with the text the field should hold next."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("reveal_button", "bool")
                        .default("true")
                        .doc("Offers the reveal button at all. A confirmation field, or one beside a revealed twin, has nothing to add."),
                    prop("reveal_label", "String")
                        .default("Show password")
                        .doc("Announced on the reveal button while the secret is hidden."),
                    prop("hide_label", "String")
                        .default("Hide password")
                        .doc("Announced on the reveal button while the secret is shown."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. Names the field through a `for`/`id` pair."),
                    prop("description", "Caption")
                        .doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control: the password rules."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required, adds `aria-required` and shows an asterisk in the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the field."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "TextField" }
                    " whose "
                    Code { source: "type" }
                    " flips between "
                    Code { source: "password" }
                    " and "
                    Code { source: "text" }
                    ", with the reveal toggle in its trailing slot. It takes the same slots "
                    "every field has, and the reveal state is its own - a password that starts "
                    "visible is not a state a caller should be able to ask for."
                }
            },
            Demo {
                component: "PasswordField",
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
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"That password is weak.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Too short.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Password\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Used to sign you in.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").default("true").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"At least 8 characters.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("placeholder").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Your password\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("reveal_button").default("true"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    PasswordField {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Password".to_string()),
                        description: (values.str("description") == "true")
                            .then(|| "Used to sign you in.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "At least 8 characters.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => {
                                FieldStatus::Warning("That password is weak.".to_string())
                            }
                            "error" => FieldStatus::Error("Too short.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "Your password".to_string()),
                        reveal_button: values.str("reveal_button") == "true",
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
