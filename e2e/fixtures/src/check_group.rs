//! `CheckboxGroup`, `SwitchGroup` and a required `Fieldset` (todos 3984, 3992).

use dioxus::prelude::*;
use libero::components::{CheckboxGroup, Fieldset, SwitchGroup, Text, TextField};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/check-group/checkbox", || rsx! { CheckboxGroupPage {} }),
    ("/check-group/switch", || rsx! { SwitchGroupPage {} }),
    (
        "/check-group/fieldset-required",
        || rsx! { RequiredFieldsetPage {} },
    ),
];

fn toppings() -> Vec<String> {
    ["Cheese", "Olives", "Basil", "Chili"]
        .map(String::from)
        .to_vec()
}

/// At most two checked; `#value` shows the selection in the order the group emits it.
#[component]
fn CheckboxGroupPage() -> Element {
    let mut value = use_signal(Vec::<String>::new);
    rsx! {
        CheckboxGroup::<String> {
            label: "Toppings",
            description: "Two at most.",
            options: toppings(),
            max_checked: 2,
            value: value(),
            onchange: move |next| value.set(next),
        }
        Text { id: "value", "{value().join(\",\")}" }
    }
}

#[component]
fn SwitchGroupPage() -> Element {
    let mut value = use_signal(|| vec!["Push".to_string()]);
    rsx! {
        SwitchGroup::<String> {
            label: "Alerts",
            options: ["Email", "Push", "Sms"].map(String::from).to_vec(),
            value: value(),
            onchange: move |next| value.set(next),
        }
        Text { id: "value", "{value().join(\",\")}" }
    }
}

/// The group's `required` reaches both fields but the one that opts out.
#[component]
fn RequiredFieldsetPage() -> Element {
    rsx! {
        Fieldset::<()> { label: "Contact", required: true,
            TextField { id: "name", label: "Name", oninput: |_| {} }
            TextField { id: "email", label: "Email", oninput: |_| {} }
            TextField { id: "phone", label: "Phone", required: false, oninput: |_| {} }
        }
    }
}
