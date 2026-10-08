use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, Code, FieldName, Fields, Fieldset, Flex, Form, FormPart, PasswordField, Rule,
    Text, TextField, Validators, is_email, min_length, not_empty, use_form, use_form_context,
};

#[derive(Clone, PartialEq, Default, Fields)]
pub struct NewPassword {
    pub value: String,
    pub repeat: String,
}

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Signup {
    pub name: String,
    pub email: String,
    #[fields(nested)]
    pub password: NewPassword,
    pub company: bool,
    pub company_name: String,
    pub terms: bool,
}

// snippet: ignore - builds on Getting Started's `EmailField` and `NewPasswordFieldset`
// snippet: mirrors SignupForm, Signup
const FORM_CODE: &str = r#"use dioxus::prelude::*;
use libero::components::{Button, Checkbox, Fields, Form, Rule, Text, TextField, not_empty};__IMPORTS__

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    name: String,
    email: String,
    #[fields(nested)]
    password: NewPassword,__COMPANY_FIELDS__
    terms: bool,
}

#[component]
fn SignupForm() -> Element {
__STATE____HANDLE__

    rsx! {
        Form {__FORM__
            value: signup,
__RULE____SUMMARY__
            onsubmit: move |_| sent.set(true),
__FIELDS____COMPANY__
__TERMS__
__BUTTONS__
__STATUS__
        }
    }
}"#;

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("form.rs"));

/// A section of the live form at the depth `FORM_CODE` splices it in.
fn nest(name: &str) -> String {
    indent(&indent(&indent(&FILE.section(name))))
        .trim_end()
        .to_string()
}

fn form_code(values: &DemoValues, _: &str) -> String {
    let handle = values.str("form") == "true";
    let pick = |on: &'static str| if handle { on } else { "" };
    let company = |on: &'static str| {
        if values.str("company") == "true" {
            on
        } else {
            ""
        }
    };
    let company_fields = match values.str("company") == "true" {
        true => format!("\n{}", nest("company")),
        false => String::new(),
    };
    let code = FORM_CODE
        .replace("__STATE__", indent(&FILE.section("state")).trim_end())
        .replace("__RULE__", &nest("rule"))
        .replace("__FIELDS__", &nest("fields"))
        .replace("__TERMS__", &nest("terms"))
        .replace("__STATUS__", &nest("status"))
        .replace(
            "__SUMMARY__",
            match values.str("summary_title").as_str() {
                "true" => "\n            summary_title: \"Please fix these first:\",",
                _ => "",
            },
        )
        .replace(
            "__IMPORTS__",
            pick("\nuse libero::components::{Flex, use_form, use_form_context};"),
        )
        .replace(
            "__COMPANY_FIELDS__",
            company("\n    company: bool,\n    company_name: String,"),
        )
        .replace("__COMPANY__", &company_fields)
        .replace("__HANDLE__", pick("\n    let form = use_form();"))
        .replace("__FORM__", pick("\n            form,"))
        .replace(
            "__BUTTONS__",
            &nest(if handle { "handle_buttons" } else { "submit" }),
        );
    match handle {
        true => format!("{code}\n\n{}", FILE.section("check_button")),
        false => code,
    }
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
                        .doc("The whole form's value. `validate` checks it, and fields named by a path read and write it. `V` is inferred from it. A form with no `value` and no rules needs `Form::<()>`."),
                    prop("validate", "Validators<V>")
                        .doc("Rules over `value`, one or an array. A rule with `.on(..)` shows its status on each field it names."),
                    prop("onsubmit", "EventHandler<FormEvent>")
                        .doc("Fires on a submit with no errors. Without an `action`, the browser's own submit is cancelled."),
                    prop("summary_title", "String")
                        .doc("A heading over the error summary."),
                    prop("form", "FormHandle")
                        .doc("Controls the form from outside, made with `use_form()`. Without it the form makes its own, which `use_form_context()` returns inside the form."),
                    prop("children", "Element")
                        .default("required")
                        .doc("The fields, fieldsets and buttons."),
                ]).extends("form")
                .parts("FormPart", vec![
                    (FormPart::Summary, "The error summary, an `Alert`, shown after a blocked submit."),
                    (FormPart::SummaryTitle, "The summary's heading, with `summary_title`."),
                    (FormPart::SummaryList, "The summary's list of errors."),
                ]),
                props("FormHandle", vec![
                    prop("validate()", "bool")
                        .doc("Checks like a submit without calling `onsubmit`. Every status shows, and with an error the summary appears and takes focus. `true` when nothing is an error."),
                    prop("submit()", "Result<(), PlatformError>")
                        .doc("Submits as the submit button would. `Unsupported` once the form is gone."),
                    prop("reset()", "()")
                        .doc("Puts the value back to `V::default()` and clears touched fields, the submit and the summary; a `type=\"reset\"` button clears those too. A field with its own `value` and handler keeps what it shows. On the desktop WebView it cannot clear a control that is not bound to the value. Under Blitz it clears unbound text fields, not checkboxes or selects."),
                    prop("is_valid()", "bool")
                        .doc("Whether nothing is an error, shown or not. It follows changes, so it can drive other UI."),
                    prop("clear_summary()", "()")
                        .doc("Hides the summary and resets nothing."),
                ]).without_base_props(),
                props("Every field", vec![
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the label. Inside a `Form`, an empty field fails the submit with `form.required` unless its own rules fail first: `form.required_check` for an unchecked `Checkbox` or `Switch`, `form.required_select` for a select. A read-only field never fails it. A slider and a `ColorField` always hold a value, and a lone `Radio` leaves it to its `RadioGroup`. A field with a handler but no `value` or binding counts as filled: pass its `value` or a `name` path."),
                    prop("validate", "Validators<T>")
                        .doc("Rules over the field's own value, one or an array. Shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<T>")
                        .doc("What the field posts as, and how rules address it. A path from `#[derive(Fields)]` also binds the field to the form's value, unless the field has a handler of its own. `T` is the field's value type."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .handles([
                    "A form becomes a `form` landmark only once it has a name. A form without a name is still valid.",
                    "Each summary line starts with its field's label, or its `aria_label` when it has no text label.",
                    "A `Fieldset` with an error `status` is one field in error: it blocks the submit and has a line named by its legend.",
                    "A rule shows once focus leaves its field, not on a move inside it, such as from one `PinField` cell to the next. The desktop WebView cannot tell the two apart, so there any move shows it.",
                ])
                .must([
                    "Name the form when the page holds more than one form, or when the form is the page's main task, such as a checkout. Pass `aria-labelledby` pointing at a visible heading, or `aria-label`.",
                    "Join intro text through `aria-describedby`, if any.",
                    "Put the field's name into its messages when its label is markup (`Caption::Node`): the summary cannot read markup, so the line has no name.",
                ])
                .example("A checkout, `Form { \"aria-labelledby\": \"checkout-title\", .. }` under a visible \"Checkout\" heading: a screen reader lists it as the \"Checkout\" form landmark, and each line of the error summary starts with its field's label."),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "<form novalidate>" }
                    " that holds the whole value in one store, runs rules across its fields and "
                    "validates on submit. The Form getting started page shows how fields, "
                    "fieldsets, rules and paths fit together."
                }
            },
            // snippet: ignore - builds on Getting Started's `EmailField` and `NewPasswordFieldset`
            Demo {
                component: "Form",
                children_text: "",
                wrap: Wrap(form_code),
                // `form` passes a `use_form()` handle, and the buttons
                // that use it appear.
                controls: vec![
                    Control::switch("summary_title").code(silent),
                    Control::switch("form").code(silent),
                    Control::switch("company").code(silent),
                ],
                render: move |values: DemoValues| rsx! {
                    SignupForm {
                        summary_title: values.str("summary_title") == "true",
                        handle: values.str("form") == "true",
                        company: values.str("company") == "true",
                    }
                },
            }

            DocSection {
                title: "Submit",
                Text {
                    "A submit shows every status. With an error, it is cancelled, and a summary of "
                    "every problem appears above the fields and takes focus. Warnings never block. "
                    "A "
                    Code { source: "required" }
                    " field left empty is an error too, \"Fill in this field.\", or \"Check this box.\" "
                    "and \"Select an item in the list.\" for a box or a select ("
                    Code { source: "Localization::form" }
                    "), unless a rule of its own says more; a disabled or read-only one never blocks. "
                    "The error shows only once the form was submitted, so tabbing through an empty "
                    "form paints nothing; after that it follows the value like any rule. "
                    "The summary keeps the problems of that submit. A line leaves once it is fixed, "
                    "and new ones wait for the next submit."
                }
            }

            DocSection {
                title: "Conditional fields",
                Text {
                    "A field shown only for some values is a plain "
                    Code { source: "if" }
                    " around it, as the company switch shows. Its rules leave with it, so a "
                    "hidden field never blocks a submit. Its value stays in the store, and a "
                    "rule on the whole form still runs, so check the condition there too."
                }
            }
        }
    }
}

#[component]
fn SignupForm(summary_title: bool, handle: bool, company: bool) -> Element {
    // demo-code: state start
    let signup = use_store(Signup::default);
    let mut sent = use_signal(|| false);
    // demo-code: state end
    // Always made, so the hook order holds; the form only gets it while the
    // switch is on.
    let form = use_form();

    rsx! {
        Form {
            sx: libero::sx::sx().width("100%").max_width("320px"),
            form: handle.then_some(form),
            value: signup,
            // demo-code: rule start
            // Only the whole signup sees the email and the password together.
            validate: (|s: &Signup| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Signup::FIELDS.password().value()]),
            // demo-code: rule end
            summary_title: summary_title.then(|| "Please fix these first:".to_string()),
            onsubmit: move |_| sent.set(true),
            // demo-code: fields start
            // `required` alone blocks an empty submit with "Fill in this field.".
            TextField { label: "Name", name: Signup::FIELDS.name(), required: true }
            // A specialized field and a composed part, built in Getting started.
            EmailField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            NewPasswordFieldset { path: Signup::FIELDS.password() }
            // demo-code: fields end
            if company {
                // demo-code: company start
                Checkbox { label: "Sign up as a company", name: Signup::FIELDS.company() }
                // Rendered only while ticked, so its rules leave with it.
                if signup.read().company {
                    TextField {
                        label: "Company name",
                        name: Signup::FIELDS.company_name(),
                        validate: not_empty.error("Enter the company name."),
                    }
                }
                // demo-code: company end
            }
            // demo-code: terms start
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                validate: not_empty.error("Accept the terms to continue."),
            }
            // demo-code: terms end
            if handle {
                // demo-code: handle_buttons start
                Text { role: "status", if form.is_valid() { "Ready to send." } else { "Not ready yet." } }
                Flex { gap: "sm",
                    Button { r#type: "submit", "Create account" }
                    CheckButton {}
                    Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
                }
                // demo-code: handle_buttons end
            } else {
                // demo-code: submit start
                Button { r#type: "submit", "Create account" }
                // demo-code: submit end
            }
            // demo-code: status start
            // Mounted before the submit, so a screen reader hears the text it gains.
            Text { role: "status",
                if sent() { "Account created." }
            }
            // demo-code: status end
        }
    }
}

// demo-code: check_button start
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
}
// demo-code: check_button end

/// The specialized field the example uses, the one Getting started builds.
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
        TextField { r#type: "email", autocomplete: "email", label, name, validate: rules, placeholder: "you@example.com" }
    }
}

/// The composed part the example uses: a new password and its repetition.
#[component]
fn NewPasswordFieldset(#[props(into)] path: FieldName<NewPassword>) -> Element {
    rsx! {
        Fieldset {
            label: "Choose a password",
            path,
            validate: (|p: &NewPassword| p.value == p.repeat)
                .error("The passwords differ.")
                .on([NewPassword::FIELDS.repeat()]),
            PasswordField {
                label: "New password",
                autocomplete: "new-password",
                name: NewPassword::FIELDS.value(),
                validate: min_length(8).error("Use at least 8 characters."),
            }
            PasswordField {
                label: "Repeat password",
                autocomplete: "new-password",
                name: NewPassword::FIELDS.repeat(),
            }
        }
    }
}
