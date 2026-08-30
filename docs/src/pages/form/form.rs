use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, Code, CodeBlock, FieldName, Fields, Fieldset, Flex, Form, PasswordField,
    Rule, Text, TextField, Validators, is_email, min_length, not_empty, use_form, use_form_context,
};

#[derive(Clone, PartialEq, Default, Fields)]
pub struct NewPassword {
    pub value: String,
    pub repeat: String,
}

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Signup {
    pub email: String,
    #[fields(nested)]
    pub password: NewPassword,
    pub terms: bool,
}

const FORM_CODE: &str = r#"use dioxus::prelude::*;
use libero::components::{Button, Checkbox, Fields, Form, Rule, Text, not_empty};

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    email: String,
    #[fields(nested)]
    password: NewPassword,
    terms: bool,
}

#[component]
fn SignupForm() -> Element {
    let signup = use_store(Signup::default);
    let mut sent = use_signal(|| false);

    rsx! {
        Form {
            value: signup,
            // Only the whole signup sees the email and the password together.
            validate: (|s: &Signup| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Signup::FIELDS.password().value()]),__SUMMARY__
            onsubmit: move |_| sent.set(true),
            // A specialized field and a composed part - built in Getting Started.
            EmailField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            NewPasswordFieldset { path: Signup::FIELDS.password() }
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                validate: not_empty.error("Accept the terms to continue."),
            }
            Button { r#type: "submit", "Create account" }
            if sent() {
                Text { "Account created." }
            }
        }
    }
}"#;

const CONTROL_CODE: &str = r#"#[component]
fn TermsForm() -> Element {
    let terms = use_store(Terms::default);
    let form = use_form();

    rsx! {
        Form {
            form,
            value: terms,
            EmailField { label: "Email", name: Terms::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            Checkbox { label: "I accept the terms", name: Terms::FIELDS.accepted(), validate: not_empty.error("Accept the terms to continue.") }
            Text { if form.is_valid() { "Ready to send." } else { "Not ready yet." } }
            Flex { gap: "sm",
                Button { r#type: "submit", "Send" }
                CheckButton {}
                Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
            }
        }
    }
}

/// Anything inside a form reaches its handle without a prop.
#[component]
fn CheckButton() -> Element {
    let form = use_form_context();
    rsx! {
        Button {
            variant: "tonal",
            onclick: move |_| {
                if let Some(form) = form {
                    form.validate();
                }
            },
            "Check"
        }
    }
}"#;

fn form_code(values: &DemoValues, _: &str) -> String {
    FORM_CODE.replace(
        "__SUMMARY__",
        match values.str("summary_title").as_str() {
            "true" => "\n            summary_title: \"Please fix these first:\",",
            _ => "",
        },
    )
}

fn silent(_: &Control, _: &DemoValues) -> Vec<String> {
    Vec::new()
}

#[component]
pub fn FormPage() -> Element {
    rsx! {
        DocPage {
            title: "Form",
            source: "libero/src/components/form/form.rs",
            markdown: "/md/form.md",
            properties: vec![
                props("Form", vec![
                    prop("value", "Store<V>")
                        .doc("The whole form's value, which `validate` checks and fields named by a path read and write. `V` is inferred from it."),
                    prop("validate", "Validators<V>")
                        .doc("Composite rules over `value` - one rule, or an array. A rule naming fields with `.on(..)` shows its status on each of them."),
                    prop("onsubmit", "EventHandler<FormEvent>")
                        .doc("Fires on a submit nothing blocks. Without an `action` the browser's own submit is cancelled."),
                    prop("summary_title", "String")
                        .doc("A heading over the error summary."),
                    prop("form", "FormHandle")
                        .doc("Controls the form from outside, made with `use_form()`. Without it the form makes its own, which `use_form_context()` reaches from inside."),
                    prop("children", "Element")
                        .doc("The fields, fieldsets and buttons."),
                ]),
                props("Every field", vec![
                    prop("validate", "Validators<V>")
                        .doc("Rules over the field's own value - one rule, or an array. Shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<T>")
                        .doc("What the field posts as, and what composite rules address it by. A path from `#[derive(Fields)]` also binds the field to the form's value unless it has a handler of its own. `T` is the field's value type."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "<form novalidate>" }
                    " that holds the whole value in one store, runs rules across its fields, and "
                    "validates on submit. How fields, fieldsets, rules and paths fit together is "
                    "explained in Forms: Getting Started."
                }
            },
            Demo {
                component: "Form",
                children_text: "",
                wrap: Wrap(form_code),
                controls: vec![Control::switch("summary_title").code(silent)],
                render: move |values: DemoValues| rsx! {
                    SignupForm { summary_title: values.str("summary_title") == "true" }
                },
            }
            DocSection { title: "Submitting",
                Text {
                    "A submit reveals every status. With any error it is cancelled, and a summary of "
                    "every problem appears above the fields and takes focus. Without errors "
                    Code { source: "onsubmit" }
                    " fires; the browser's own submit is cancelled unless the form has an "
                    Code { source: "action" }
                    ". Warnings never block. The summary keeps the problems of that submit: a line "
                    "leaves once it is fixed, and none is added until the next submit."
                }
            }
            DocSection { title: "Controlling a form",
                Text {
                    Code { source: "use_form()" }
                    " makes a handle to pass as "
                    Code { source: "form" }
                    ". Inside a form, "
                    Code { source: "use_form_context()" }
                    " returns the same handle. "
                    Code { source: "validate()" }
                    " checks like a submit without calling "
                    Code { source: "onsubmit" }
                    " and returns whether nothing is an error. "
                    Code { source: "submit()" }
                    " submits as the submit button would. "
                    Code { source: "reset()" }
                    " puts the value back to its default and clears touched fields, the submit and "
                    "the summary. A field with its own "
                    Code { source: "value" }
                    " and handler keeps what it shows - reset that state yourself. "
                    Code { source: "is_valid()" }
                    " checks without showing anything and follows changes. "
                    Code { source: "clear_summary()" }
                    " only hides the summary."
                }
                Text {
                    "Off the web, "
                    Code { source: "submit()" }
                    " does nothing and returns "
                    Code { source: "PlatformError::Unsupported" }
                    ", and "
                    Code { source: "reset()" }
                    " cannot clear a control that is not bound to the value."
                }
                TermsForm {}
                CodeBlock { source: CONTROL_CODE, language: "rust" }
            }
            DocSection { title: "Accessibility",
                Text {
                    "A status joins the field's "
                    Code { source: "aria-describedby" }
                    ", and an error sets "
                    Code { source: "aria-invalid" }
                    ". Checking while someone types stays visual; the announcement happens on submit, "
                    "where the summary - a "
                    Code { source: "role=\"alert\"" }
                    " that takes focus - reads every problem at once, each linked to its field."
                }
                Text {
                    "A form becomes a "
                    Code { source: "form" }
                    " landmark only once it has a name. Name it when the page holds more than one "
                    "form, or when the form is the page's main task, like a checkout: pass "
                    Code { source: "aria-labelledby" }
                    " pointing at a visible heading, or "
                    Code { source: "aria-label" }
                    ". Intro text can join through "
                    Code { source: "aria-describedby" }
                    ". A form without a name is still valid."
                }
            }
        }
    }
}

#[component]
fn SignupForm(summary_title: bool) -> Element {
    let signup = use_store(Signup::default);
    let mut sent = use_signal(|| false);

    rsx! {
        Form {
            sx: libero::sx::sx().width("320px"),
            value: signup,
            validate: (|s: &Signup| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Signup::FIELDS.password().value()]),
            summary_title: summary_title.then(|| "Please fix these first:".to_string()),
            onsubmit: move |_| sent.set(true),
            EmailField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            NewPasswordFieldset { path: Signup::FIELDS.password() }
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                validate: not_empty.error("Accept the terms to continue."),
            }
            Button { r#type: "submit", "Create account" }
            if sent() {
                Text { "Account created." }
            }
        }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Terms {
    pub email: String,
    pub accepted: bool,
}

#[component]
fn TermsForm() -> Element {
    let terms = use_store(Terms::default);
    let form = use_form();

    rsx! {
        Form {
            sx: libero::sx::sx().width("320px"),
            form,
            value: terms,
            EmailField { label: "Email", name: Terms::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            Checkbox { label: "I accept the terms", name: Terms::FIELDS.accepted(), validate: not_empty.error("Accept the terms to continue.") }
            Text { if form.is_valid() { "Ready to send." } else { "Not ready yet." } }
            Flex { gap: "sm",
                Button { r#type: "submit", "Send" }
                CheckButton {}
                Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
            }
        }
    }
}

#[component]
fn CheckButton() -> Element {
    let form = use_form_context();
    rsx! {
        Button {
            variant: "tonal",
            onclick: move |_| {
                if let Some(form) = form {
                    form.validate();
                }
            },
            "Check"
        }
    }
}

/// The specialized field the example uses - the same one Getting Started builds.
#[component]
fn EmailField(
    #[props(into)] label: String,
    #[props(default, into)] name: FieldName<String>,
    #[props(default, into)] validate: Validators<String>,
) -> Element {
    let rules: Vec<_> = validate
        .iter()
        .cloned()
        .chain(std::iter::once(
            is_email.error("That is not an email address."),
        ))
        .collect();
    rsx! {
        TextField { r#type: "email", label, name, validate: rules, placeholder: "you@example.com" }
    }
}

/// The composed part the example uses: a new password and its repetition.
#[component]
fn NewPasswordFieldset(#[props(into)] path: FieldName<NewPassword>) -> Element {
    rsx! {
        Fieldset {
            label: "Password",
            path,
            validate: (|p: &NewPassword| p.value == p.repeat)
                .error("The passwords differ.")
                .on([NewPassword::FIELDS.repeat()]),
            PasswordField {
                label: "Password",
                name: NewPassword::FIELDS.value(),
                validate: min_length(8).error("Use at least 8 characters."),
            }
            PasswordField { label: "Repeat password", name: NewPassword::FIELDS.repeat() }
        }
    }
}
