use dioxus::prelude::*;

// demo-code: start
use libero::components::{FieldName, Rule, TextField, Validators, is_email};

/// A text field that only accepts email addresses.
#[component]
pub fn EmailField(
    #[props(into)] label: String,
    #[props(default, into)] name: FieldName<String>,
    #[props(default, into)] validate: Validators<String>,
    #[props(default)] required: bool,
) -> Element {
    // The caller's rules go first, so a missing email shows before a malformed
    // one. Then the rule that makes this an email field.
    let rules = validate.with(is_email.error("That is not an email address."));
    rsx! {
        TextField { r#type: "email", autocomplete: "email", label, name, required, validate: rules, placeholder: "you@example.com" }
    }
}
// demo-code: end
