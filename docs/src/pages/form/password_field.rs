use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, a11y, field_controls, field_props, prop, props,
    readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::components::{Code, PasswordField, Text};
use libero::use_theme;

struct PasswordCopy;

impl FieldCopy for PasswordCopy {
    const LABEL: &'static str = "Password";
    const DESCRIPTION: &'static str = "Used to sign you in.";
    const HELPER: &'static str = "At least 8 characters.";
    const WARNING: &'static str = "That password is weak.";
    const ERROR: &'static str = "Too short.";
}

#[component]
pub fn PasswordFieldPage() -> Element {
    let mut value = use_signal(String::new);
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "PasswordField",
            source: "libero/src/components/form/password_field.rs",
            markdown: "/md/password_field.md",
            properties: vec![
                props("PasswordField", vec![
                    prop("size", "Size")
                        .default(theme.text_field.size.as_str())
                        .doc("Height, padding and font size."),
                    prop("radius", "ThemeAwareValue")
                        .default(theme.text_field.radius.as_str())
                        .doc("Corner radius, independent of `size`. Or any CSS, e.g. `radius: \"0\"`."),
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
                        .doc("The field's caption, above the control. It names the field."),                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. The password rules."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label. Inside a `Form`, an empty one fails the submit."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables and dims the field, reveal button included."),
                    readonly_prop("field"),
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
                    "Leave `label` unset only when something else names the field, such as an `aria_label` attribute.",
                    "Set `autocomplete` so password managers can fill the field: `\"new-password\"` on a sign-up form, `\"current-password\"` on a sign-in form.",
                ])
                .example("A sign-in password, `PasswordField { label: \"Password\", autocomplete: \"current-password\" }`: the password manager fills it, and the reveal button reads as pressed while the password shows."),
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
                controls: [vec![
                    Control::sizes("size")
                        .default(theme.text_field.size.as_str()),
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.text_field.radius.as_str()),
                ], field_controls::<PasswordCopy>(), vec![
                    Control::switch("placeholder").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Your password\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("reveal_button").default("true"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<PasswordCopy>(&values);
                    rsx! {
                    PasswordField {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: field.label,
                        aria_label: field.aria_label,
                        description: field.description,
                        helper: field.helper,
                        status: field.status,
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "Your password".to_string()),
                        reveal_button: values.str("reveal_button") == "true",
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        autocomplete: "current-password",
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                }
                },
            }
        }
    }
}
