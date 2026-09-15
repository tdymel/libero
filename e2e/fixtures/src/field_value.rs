//! Controlled fields whose value and slots redraw without their shell (todo 29).
//! One field per `data-case`; each echo shows what the caller holds.

use dioxus::prelude::*;
use libero::components::{Flex, NativeSelect, NumberField, Options, PasswordField, TextField};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[("/field-value", || rsx! { FieldValuePage {} })];

#[component]
fn FieldValuePage() -> Element {
    let mut text = use_signal(String::new);
    let mut quantity = use_signal(|| Some(3i32));
    let mut pick = use_signal(|| Fruit::Banana);
    let mut secret = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { "data-case": "text",
                TextField {
                    label: "Handle",
                    // The slot reads the text, so every keystroke gives it a new `Element`.
                    trailing: rsx! { span { "data-echo": "text", "{text().len()}" } },
                    value: text(),
                    oninput: move |next| text.set(next),
                }
            }
            div { "data-case": "number",
                NumberField {
                    label: "Quantity",
                    value: quantity(),
                    onchange: move |next| quantity.set(next),
                }
                span { "data-echo": "number", {quantity().map(|q| q.to_string()).unwrap_or_default()} }
            }
            div { "data-case": "native",
                NativeSelect {
                    label: "Pick",
                    value: pick(),
                    onchange: move |next| pick.set(next),
                }
                span { "data-echo": "native", "{pick().value()}" }
            }
            div { "data-case": "password",
                PasswordField {
                    label: "Password",
                    value: secret(),
                    oninput: move |next| secret.set(next),
                }
            }
        }
    }
}
