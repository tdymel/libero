//! `Form` with a bound, validated `TextField`, and buttons that write the
//! form's value and re-render the page from outside the form.

use dioxus::prelude::*;
use libero::components::{Button, Fields, Flex, Form, Rule, TextField, not_empty};

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

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { "data-renders": "{renders}", "Renders {renders}" }
            Form {
                value,
                TextField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Email needed") }
            }
            Button { onclick: move |_| value.write().email.clear(), "Clear" }
            Button { onclick: move |_| renders += 1, "Rerender" }
        }
    }
}
