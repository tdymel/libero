//! `Switch`, in every field slot and state its docs page shows.

use dioxus::prelude::*;
use libero::components::{Fields, Flex, Form, Switch};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/switch", || rsx! { SwitchPage {} }),
    ("/switch/form", || rsx! { SwitchFormPage {} }),
    ("/switch/raw-form", || rsx! { SwitchRawFormPage {} }),
];

/// Todo 660: a switch in a raw `<form>`, no libero `Form`, counting submits.
#[component]
fn SwitchRawFormPage() -> Element {
    let mut on = use_signal(|| false);
    let mut submits = use_signal(|| 0u32);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { id: "submits", "data-submits": "{submits}", "Submits {submits}" }
            form {
                onsubmit: move |event: FormEvent| {
                    event.prevent_default();
                    submits += 1;
                },
                Switch {
                    id: "alerts",
                    label: "Alerts",
                    checked: on(),
                    onchange: move |next| on.set(next),
                }
                button { r#type: "submit", "Save" }
            }
        }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Prefs {
    alerts: bool,
}

/// Todo 508: a bound switch in a `Form` with a submit button, counting submits.
#[component]
fn SwitchFormPage() -> Element {
    let value = use_store(Prefs::default);
    let mut submits = use_signal(|| 0u32);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span {
                id: "submits",
                "data-submits": "{submits}",
                "data-on": "{value().alerts}",
                "Submits {submits}"
            }
            Form { value, onsubmit: move |_| submits += 1,
                Switch { id: "alerts", label: "Alerts", name: Prefs::FIELDS.alerts() }
                button { r#type: "submit", "Save" }
            }
        }
    }
}

/// A controlled switch counting its changes, then one per state: error and
/// required, `aria_label` only, readonly, disabled, card, and uncontrolled.
#[component]
fn SwitchPage() -> Element {
    let mut on = use_signal(|| false);
    let mut changes = use_signal(|| 0u32);
    let mut card = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Switch {
                id: "plain",
                label: "Notifications",
                description: "About once a month.",
                helper: "You can turn this off later.",
                checked: on(),
                onchange: move |next| {
                    on.set(next);
                    changes += 1;
                },
            }
            div { id: "changes", "data-changes": "{changes}", "data-on": "{on}", "Changes {changes}" }
            Switch {
                id: "error",
                label: "Accept terms",
                required: true,
                status: "Turn this on to continue.",
            }
            Switch { id: "aria", aria_label: "Wi-Fi" }
            Switch {
                id: "readonly",
                label: "Locked",
                readonly: true,
                checked: true,
                onchange: move |_| changes += 100,
            }
            Switch { id: "disabled", label: "Disabled", disabled: true }
            Switch {
                id: "card",
                variant: "card",
                label: "Bluetooth",
                description: "Finds nearby devices.",
                checked: card(),
                onchange: move |next| card.set(next),
            }
            Switch { id: "free", label: "Uncontrolled" }
        }
    }
}
