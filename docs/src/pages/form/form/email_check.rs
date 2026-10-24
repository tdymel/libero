use dioxus::prelude::*;
use std::time::Duration;

use super::sign_in::delay;

// demo-code: start
use libero::components::{Button, Fields, Form, Text, TextField, checked_async, use_form};
use libero::localization::Localization;

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    email: Option<String>,
}

enum EmailError {
    Invalid,
    Taken,
}

fn email(text: String) -> Result<String, EmailError> {
    match text.contains('@') {
        true => Ok(text.trim().to_string()),
        false => Err(EmailError::Invalid),
    }
}

fn email_message(error: &EmailError, _: &Localization) -> String {
    match error {
        EmailError::Invalid => "Enter an email address, such as me@example.com.".into(),
        EmailError::Taken => "This address is taken. Sign in instead.".into(),
    }
}

#[component]
pub fn EmailCheckForm() -> Element {
    let signup = use_store(Signup::default);
    let form = use_form();
    let mut sent = use_signal(String::new);

    rsx! {
        Form {
            sx: libero::sx::sx().width("100%").max_width("320px"),
            form,
            value: signup,
            // A submit waits for the check, then sends or shows its answer.
            onsubmit: move |_| sent.set(signup.read().email.clone().unwrap_or_default()),
            TextField {
                label: "Email",
                autocomplete: "email",
                required: true,
                // Dimmed while the server checks, once the field showed its rules.
                sx: libero::sx::sx().when("pending", libero::sx::sx().opacity("0.6").cursor("progress")),
                // Asked 400 ms after the last keystroke, once the address parsed.
                name: Signup::FIELDS.email().map(String::clone, email).message(email_message)
                    .rules([checked_async(email_free).debounce(400)]),
            }
            Button { r#type: "submit", loading: form.is_submitting(), "Sign up" }
            Text { role: "status",
                if !sent().is_empty() { "Signed up as {sent}." }
            }
        }
    }
}
// demo-code: end

/// Stands in for a server: a second's wait, then `taken@example.com` is taken.
async fn email_free(email: String) -> Result<(), EmailError> {
    delay(Duration::from_secs(1)).await;
    match email == "taken@example.com" {
        true => Err(EmailError::Taken),
        false => Ok(()),
    }
}
