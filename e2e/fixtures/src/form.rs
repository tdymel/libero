//! `Form` with a bound, validated `TextField`, and buttons that write the
//! form's value and re-render the page from outside the form, submit it and
//! reset it.

use dioxus::prelude::*;
use libero::components::{Button, Fields, Flex, Form, Rule, TextField, not_empty, use_form};

use crate::Routes;

pub const ROUTES: Routes = &[("/form", || rsx! { FormPage {} })];

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    email: String,
}

#[component]
pub fn FormPage() -> Element {
    let mut value = use_store(Signup::default);
    let mut renders = use_signal(|| 0u32);
    let mut submits = use_signal(|| 0u32);
    let form = use_form();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { "data-renders": "{renders}", "Renders {renders}" }
            span { "data-submits": "{submits}", "Submits {submits}" }
            Form {
                value,
                form,
                onsubmit: move |_| submits += 1,
                TextField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Email needed") }
                button { r#type: "submit", "Send" }
            }
            Button { onclick: move |_| value.write().email.clear(), "Clear" }
            Button { onclick: move |_| renders += 1, "Rerender" }
            Button { onclick: move |_| { let _ = form.submit(); }, "Submit" }
            Button { onclick: move |_| form.reset(), "Reset" }
        }
    }
}
