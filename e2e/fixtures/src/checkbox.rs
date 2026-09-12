//! `Checkbox`, in every variant its docs page shows.

use dioxus::prelude::*;
use libero::components::{Checkbox, FieldStatus, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/checkbox", || rsx! { CheckboxPage {} })];

/// Every checkbox reports what it last emitted into `data-emitted`, so a test
/// reads the value a press produced rather than the look.
#[component]
fn CheckboxPage() -> Element {
    let mut terms = use_signal(|| false);
    let mut mail = use_signal(|| true);
    let mut sms = use_signal(|| false);
    let mut support = use_signal(|| false);
    let mut consent = use_signal(|| false);
    let mut bare = use_signal(|| false);
    let mut emitted = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px", "data-emitted": emitted(),
            Checkbox {
                id: "terms",
                label: "Accept the terms",
                checked: terms(),
                onchange: move |next| {
                    terms.set(next);
                    emitted.set(format!("terms:{next}"));
                },
            }
            Checkbox {
                id: "all",
                label: "All channels",
                checked: mail() && sms(),
                indeterminate: (mail() != sms()).then_some(true),
                onchange: move |next| {
                    mail.set(next);
                    sms.set(next);
                    emitted.set(format!("all:{next}"));
                },
            }
            Checkbox {
                id: "bare",
                aria_label: "Select row",
                checked: bare(),
                onchange: move |next| {
                    bare.set(next);
                    emitted.set(format!("bare:{next}"));
                },
            }
            Checkbox {
                id: "support",
                variant: "card",
                label: "Priority support",
                description: "Answers within four hours, around the clock.",
                checked: support(),
                onchange: move |next| {
                    support.set(next);
                    emitted.set(format!("support:{next}"));
                },
            }
            Checkbox {
                id: "consent",
                label: "Share usage data",
                helper: "You can withdraw consent at any time.",
                required: true,
                status: FieldStatus::Error("Accept to continue.".to_string()),
                checked: consent(),
                onchange: move |next| {
                    consent.set(next);
                    emitted.set(format!("consent:{next}"));
                },
            }
            Checkbox {
                id: "locked",
                label: "Locked",
                readonly: true,
                checked: true,
                onchange: move |next| emitted.set(format!("locked:{next}")),
            }
            Checkbox {
                id: "off",
                label: "Unavailable",
                disabled: true,
                checked: false,
                onchange: move |next| emitted.set(format!("off:{next}")),
            }
        }
    }
}
