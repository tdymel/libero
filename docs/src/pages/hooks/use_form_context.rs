use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{
    Anchor, Button, Code, CodeBlock, Fields, Flex, Form, Rule, Text, TextField, not_empty,
    use_form_context,
};

const CHECK: &str = r#"#[derive(Clone, PartialEq, Default, Fields)]
struct Contact {
    name: String,
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
}

#[component]
fn ContactForm() -> Element {
    let value = use_store(Contact::default);

    rsx! {
        Form { value,
            TextField { label: "Name", name: Contact::FIELDS.name(), validate: not_empty.error("Enter your name.") }
            Flex { direction: "row", gap: "sm",
                Button { r#type: "submit", "Send" }
                CheckButton {}
            }
        }
    }
}"#;

#[derive(Clone, PartialEq, Default, Fields)]
struct Contact {
    name: String,
}

/// `CHECK`, rendered.
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

#[component]
fn ContactForm() -> Element {
    let value = use_store(Contact::default);

    rsx! {
        Form { value,
            TextField {
                label: "Name",
                name: Contact::FIELDS.name(),
                validate: not_empty.error("Enter your name."),
            }
            Flex { direction: "row", gap: "sm",
                Button { r#type: "submit", "Send" }
                CheckButton {}
            }
        }
    }
}

#[component]
pub fn UseFormContextPage() -> Element {
    rsx! {
        DocPage {
            title: "use_form_context",
            source: "libero/src/components/form/handle.rs",
            markdown: "/md/use_form_context.md",
            lead: rsx! {
                Text {
                    Code { source: "use_form_context() -> Option<FormHandle>" }
                    " returns the handle of the "
                    Code { source: "Form" }
                    " it is called inside, or "
                    Code { source: "None" }
                    " outside one. A part of a form reaches the form without a prop. "
                    Anchor { to: Route::FormPage {}, "Form" }
                    " documents the handle's methods."
                }
            },

            DocSection {
                title: "Usage",
                ContactForm {}
                CodeBlock { source: CHECK, language: "rust" }
                Text {
                    "It is the same handle "
                    Anchor { to: Route::UseFormPage {}, "use_form" }
                    " makes. A form given none makes its own, and this hook reaches that one."
                }
            }
        }
    }
}
