use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::FieldPart;
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
                    prop("size", "Size").default("md").doc("Height, padding and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of `size`."),
                    prop("value", "Option<String>")
                        .doc("The secret. Leave it out and the input keeps its own text."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires on every keystroke with the text the field should hold next."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the secret, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<String>")
                        .doc("What the field posts as. A path such as `Signup::FIELDS.password()` also binds the secret to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("reveal_button", "bool")
                        .default("theme.password_field.reveal_button")
                        .doc("Shows the reveal button. Turn it off for a confirmation field, which adds nothing beside a revealed twin."),
                    prop("reveal_label", "String")
                        .default("password_field.show")
                        .doc("The reveal button's name, such as \"Show PIN\". Unset, the localization's `password_field.show`, \"Show password\" in English."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. It names the field."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. The password rules."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables and dims the field, reveal button included."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post."),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "The bordered box around the control."),
                    (FieldPart::Control, "The element the label names."),
                    (FieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ]).extends("input"),
            ],
            accessibility: a11y()
                .handles(["The reveal button is a toggle with one name, so a screen reader hears it as pressed or not."])
                .must([
                    "Leave `label` unset only when something else names the field.",
                    "Set `autocomplete` so password managers can fill the field: `\"new-password\"` on a sign-up form, `\"current-password\"` on a sign-in form.",
                ]),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "TextField" }
                    " for secrets, with a button that shows the text. The field keeps the "
                    "reveal state itself, so a password never starts visible. Every submit and "
                    "reset of the surrounding "
                    Code { source: "Form" }
                    " hides the secret again."
                }
            },
            // snippet: let mut value = use_signal(String::new);
            Demo {
                component: "PasswordField",
                children_text: "",
                fixed: vec![
                    "autocomplete: \"current-password\"".to_string(),
                    "value: value()".to_string(),
                    "oninput: move |next| value.set(next)".to_string(),
                ],
                controls: vec![
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
                        .default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .labels(["Valid", "Warning", "Error"])
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
                            _ => vec!["aria_label: \"Password\"".to_string()],
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
                        aria_label: (values.str("label") != "true").then_some("Password"),
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
                        autocomplete: "current-password",
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                },
            }
        }
    }
}
