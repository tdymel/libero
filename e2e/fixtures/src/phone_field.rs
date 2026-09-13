//! `PhoneField`: the country picker, its searchable list and the `tel` input.

use dioxus::prelude::*;
use libero::components::{FieldStatus, Flex, PhoneField, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/phone-field", || rsx! { PhoneFieldPage {} }),
    ("/phone-field/error", || rsx! { PhoneFieldErrorPage {} }),
];

/// A three-band flag, `aria-hidden`: the docs page's own.
#[component]
fn Flag(iso: String) -> Element {
    let colour = match iso.as_str() {
        "DE" => "#dd0000",
        "FR" => "#002395",
        _ => "#cccccc",
    };
    rsx! {
        svg { width: "16", height: "12", view_box: "0 0 3 3", "aria-hidden": "true",
            rect { width: "3", height: "3", fill: "{colour}" }
        }
    }
}

#[component]
fn PhoneFieldPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PhoneField {
                label: "Mobile",
                country: "DE",
                flag: move |iso: String| rsx! { Flag { iso } },
                value: value(),
                oninput: move |next| value.set(next),
            }
            Text { size: "sm", id: "e164", "{value}" }
        }
    }
}

/// An error and `required`, the docs page's switches.
#[component]
fn PhoneFieldErrorPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PhoneField {
                label: "Mobile",
                country: "US",
                required: true,
                status: FieldStatus::Error("Enter a phone number.".into()),
            }
        }
    }
}
