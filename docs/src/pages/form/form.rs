use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, Code, CodeBlock, Fields, Form, PasswordField, Rule, Text, TextField,
    is_email, min_length, not_empty,
};

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Signup {
    pub email: String,
    pub password: String,
    pub confirm: String,
    pub terms: bool,
}

const FORM_CODE: &str = r#"use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, Fields, Form, PasswordField, Rule, Text, TextField, is_email, min_length,
    not_empty,
};

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    email: String,
    password: String,
    confirm: String,
    terms: bool,
}

#[component]
fn SignupForm() -> Element {
    let mut signup = use_signal(Signup::default);
    let mut sent = use_signal(|| false);

    rsx! {
        Form {
            value: signup(),
            validate: (|s: &Signup| s.password == s.confirm)
                .error("The passwords differ.")
                .on([Signup::FIELDS.password(), Signup::FIELDS.confirm()]),__SUMMARY__
            onsubmit: move |_| sent.set(true),
            TextField {
                label: "Email",
                name: Signup::FIELDS.email(),
                value: signup().email,
                oninput: move |next| signup.write().email = next,
                validate: [
                    not_empty.error("Enter your email."),
                    is_email.error("That is not an email address."),
                ],
            }
            PasswordField {
                label: "Password",
                name: Signup::FIELDS.password(),
                value: signup().password,
                oninput: move |next| signup.write().password = next,
                validate: min_length(8).error("Use at least 8 characters."),
            }
            PasswordField {
                label: "Repeat password",
                name: Signup::FIELDS.confirm(),
                value: signup().confirm,
                oninput: move |next| signup.write().confirm = next,
            }
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                checked: signup().terms,
                onchange: move |next| signup.write().terms = next,
                validate: not_empty.error("Accept the terms to continue."),
            }
            Button { r#type: "submit", "Create account" }
            if sent() {
                Text { "Account created." }
            }
        }
    }
}"#;

const RULES_CODE: &str = r#"use libero::components::{Rule, TextField, max_length, min_length, not_empty};

// A rule is any `Fn(&V) -> bool` - your own function is a peer of the catalog.
fn not_admin(name: &String) -> bool {
    name != "admin"
}

rsx! {
    // One rule needs no brackets.
    TextField { label: "Nickname", value: nickname(), oninput: .., validate: not_empty.error("Pick one.") }

    // Several run in order: the first error wins, a warning shows only without one.
    TextField {
        label: "Username",
        value: username(),
        oninput: move |next| username.set(next),
        validate: [
            not_empty.error("Pick a username."),
            min_length(3).and(not_admin).error("At least 3 characters, and not \"admin\"."),
            max_length(12).warn("Long names get cut off in lists."),
        ],
    }
}"#;

const CATALOG_CODE: &str = r#"use libero::components::{Rule, is_email, max, max_length, min, min_length, not_empty};

not_empty                        // String (whitespace is empty), Option<T>, Vec<T>, bool
min_length(3) / max_length(12)   // characters, not bytes
min(18) / max(99)                // any PartialOrd value, bounds included
is_email                         // a shape check: one @, a dotted domain

rule.error("..")                 // an error blocks a submit
rule.warn("..")                  // a warning never does
rule.and(other) / rule.or(other) // compose before the message"#;

const PATHS_CODE: &str = r#"use libero::components::{Fields, path};

#[derive(Fields)]
struct Order {
    name: String,
    #[fields(nested)]
    address: Address, // derives Fields too
}

Order::FIELDS.name()              // FieldPath<Order, String>, posts as "name"
Order::FIELDS.address().zip()     // FieldPath<Order, String>, posts as "address.zip"
Order::FIELDS.address().path()    // FieldPath<Order, Address>, "address"
Order::FIELDS.adress().zip()      // does not compile

// For a type that cannot derive, the macro compiles the same field access:
path!(Order => address.zip)"#;

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
                    prop("value", "V")
                        .doc("The whole form's value, which `validate` checks. `V` is inferred from it."),
                    prop("validate", "Validators<V>")
                        .doc("Composite rules over `value` - one rule, or an array. A rule naming fields with `.on(..)` shows its status on each of them."),
                    prop("onsubmit", "EventHandler<FormEvent>")
                        .doc("Fires on a submit nothing blocks. Without an `action` the browser's own submit is cancelled."),
                    prop("summary_title", "String")
                        .doc("A heading over the error summary."),
                    prop("children", "Element")
                        .doc("The fields, fieldsets and buttons."),
                ]),
                props("Every field", vec![
                    prop("validate", "Validators<V>")
                        .doc("Rules over the field's own value - one rule, or an array. Shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldPath | String")
                        .doc("What the field posts as, and the path composite rules address it by."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "<form novalidate>" }
                    " that validates on submit. Every field inside checks its own value through "
                    Code { source: "validate" }
                    "; the form checks rules across several fields and puts a problem on the fields "
                    "it names. A submit with an error is cancelled, every field shows its status, and "
                    "a summary of every problem takes focus. A rule is a plain function from a value "
                    "to a "
                    Code { source: "bool" }
                    ", given a message with "
                    Code { source: ".error(..)" }
                    " or "
                    Code { source: ".warn(..)" }
                    "."
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
            DocSection { title: "Field rules",
                Text {
                    "Every field takes "
                    Code { source: "validate" }
                    ", typed over its own value: "
                    Code { source: "String" }
                    " for text fields, "
                    Code { source: "bool" }
                    " for a checkbox, "
                    Code { source: "Option<T>" }
                    " for a select. A field validates itself with or without a "
                    Code { source: "Form" }
                    " around it."
                }
                CodeBlock { source: RULES_CODE, language: "rust" }
                CodeBlock { source: CATALOG_CODE, language: "rust" }
            }
            DocSection { title: "Typed field paths",
                Text {
                    Code { source: "#[derive(Fields)]" }
                    " gives a struct a "
                    Code { source: "FIELDS" }
                    " constant with one method per field. A path is also the field's "
                    Code { source: "name" }
                    ", so what a composite rule names and what the form posts are one string, and a "
                    "typo is a compile error rather than an error that never shows. "
                    Code { source: ".on(..)" }
                    " only takes paths rooted at the rule's own value type."
                }
                CodeBlock { source: PATHS_CODE, language: "rust" }
            }
            DocSection { title: "When a status shows",
                Text {
                    "A field's rules wait until it loses focus for the first time, then follow every "
                    "change. A composite problem waits until every field it names was touched. A "
                    "submit reveals everything. An explicit "
                    Code { source: "status" }
                    " - a server's answer - never waits, and when it and the rules disagree the worse "
                    "one shows."
                }
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
            }
        }
    }
}

#[component]
fn SignupForm(summary_title: bool) -> Element {
    let mut signup = use_signal(Signup::default);
    let mut sent = use_signal(|| false);

    rsx! {
        Form {
            sx: libero::sx::sx().width("320px"),
            value: signup(),
            validate: (|s: &Signup| s.password == s.confirm)
                .error("The passwords differ.")
                .on([Signup::FIELDS.password(), Signup::FIELDS.confirm()]),
            summary_title: summary_title.then(|| "Please fix these first:".to_string()),
            onsubmit: move |_| sent.set(true),
            TextField {
                label: "Email",
                name: Signup::FIELDS.email(),
                value: signup().email,
                oninput: move |next| signup.write().email = next,
                validate: [
                    not_empty.error("Enter your email."),
                    is_email.error("That is not an email address."),
                ],
            }
            PasswordField {
                label: "Password",
                name: Signup::FIELDS.password(),
                value: signup().password,
                oninput: move |next| signup.write().password = next,
                validate: min_length(8).error("Use at least 8 characters."),
            }
            PasswordField {
                label: "Repeat password",
                name: Signup::FIELDS.confirm(),
                value: signup().confirm,
                oninput: move |next| signup.write().confirm = next,
            }
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                checked: signup().terms,
                onchange: move |next| signup.write().terms = next,
                validate: not_empty.error("Accept the terms to continue."),
            }
            Button { r#type: "submit", "Create account" }
            if sent() {
                Text { "Account created." }
            }
        }
    }
}
