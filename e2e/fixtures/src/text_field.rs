//! `TextField`, `PasswordField` and `Textarea`: the label, caption and status
//! wiring, rules shown on blur, and the password's reveal toggle. One field per
//! `data-case`.

use dioxus::prelude::*;
use libero::components::{
    Fieldset, Flex, Form, PasswordField, Rule, TextField, Textarea, not_empty,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/text-field", || rsx! { TextFieldPage {} }),
    ("/text-field/form", || rsx! { PasswordFormPage {} }),
];

#[component]
fn TextFieldPage() -> Element {
    let mut handle = use_signal(|| "ada".to_string());

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { "data-case": "wired",
                TextField {
                    label: "Username",
                    description: "How other people see you.",
                    helper: "Letters and digits.",
                    status: "That name is taken.",
                    required: true,
                    "aria-describedby": "outside-note",
                    value: handle(),
                    oninput: move |next| handle.set(next),
                }
                span { id: "outside-note", "Shown on your profile." }
            }
            div { "data-case": "slots",
                TextField {
                    label: "Handle",
                    leading: rsx! { "@" },
                    trailing: rsx! { "{handle().len()}/20" },
                    value: handle(),
                    oninput: move |next| handle.set(next),
                }
            }
            div { "data-case": "validated",
                TextField { label: "Email", validate: not_empty.error("Email needed") }
            }
            div { "data-case": "password",
                PasswordField { label: "Password", helper: "At least 8 characters." }
            }
            div { "data-case": "password-xs",
                PasswordField { label: "PIN", size: "xs" }
            }
            div { "data-case": "fieldset",
                Fieldset::<String> { label: "Locked", disabled: true,
                    PasswordField { label: "Old password" }
                }
            }
            div { "data-case": "textarea",
                Textarea {
                    label: "Bio",
                    helper: "Markdown works.",
                    required: true,
                    validate: not_empty.error("Bio needed"),
                }
            }
        }
    }
}

/// On a page of its own: the other cases walk the whole tab cycle.
#[component]
fn PasswordFormPage() -> Element {
    rsx! {
        div { "data-case": "password-form",
            Form::<()> {
                PasswordField { label: "Secret" }
                button { r#type: "submit", "Send" }
                button { r#type: "reset", "Clear" }
            }
        }
    }
}
