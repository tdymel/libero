use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{
    Anchor, Button, Code, CodeBlock, Fields, Flex, Form, Rule, Text, TextField, not_empty, use_form,
};

const NEWSLETTER: &str = r#"#[derive(Clone, PartialEq, Default, Fields)]
struct Newsletter {
    email: String,
}

#[component]
fn NewsletterForm() -> Element {
    let value = use_store(Newsletter::default);
    let form = use_form();

    rsx! {
        Form { form, value,
            TextField { label: "Email", name: Newsletter::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            Text { if form.is_valid() { "Ready to send." } else { "Not ready yet." } }
            Flex { direction: "row", gap: "sm",
                Button { r#type: "submit", "Subscribe" }
                Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
            }
        }
    }
}"#;

#[derive(Clone, PartialEq, Default, Fields)]
struct Newsletter {
    email: String,
}

/// `NEWSLETTER`, rendered.
#[component]
fn NewsletterForm() -> Element {
    let value = use_store(Newsletter::default);
    let form = use_form();

    rsx! {
        Form { form, value,
            TextField {
                label: "Email",
                name: Newsletter::FIELDS.email(),
                validate: not_empty.error("Enter your email."),
            }
            Text {
                if form.is_valid() {
                    "Ready to send."
                } else {
                    "Not ready yet."
                }
            }
            Flex { direction: "row", gap: "sm",
                Button { r#type: "submit", "Subscribe" }
                Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
            }
        }
    }
}

#[component]
pub fn UseFormPage() -> Element {
    rsx! {
        DocPage {
            title: "use_form",
            source: "libero/src/components/form/handle.rs",
            markdown: "/md/use_form.md",
            lead: rsx! {
                Text {
                    Code { source: "use_form() -> FormHandle" }
                    " controls a "
                    Code { source: "Form" }
                    " from the component that renders it: whether it is valid, a check, a "
                    "submit and a reset. Pass it as the form's "
                    Code { source: "form" }
                    ". "
                    Anchor { to: Route::FormPage {}, "Form" }
                    " documents validation and the error summary."
                }
            },

            DocSection {
                title: "Usage",
                NewsletterForm {}
                CodeBlock { source: NEWSLETTER, language: "rust" }
                Text {
                    Code { source: "is_valid()" }
                    " follows every change, so it can drive other UI. "
                    Code { source: "reset()" }
                    " puts the value back to its default with nothing touched."
                }
            }
        }
    }
}
