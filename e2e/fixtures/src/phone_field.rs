//! `PhoneField`: the country picker, its searchable list and the `tel` input.

use dioxus::prelude::*;
use libero::{
    components::{FieldStatus, Flex, PhoneField, Text},
    hooks::use_localization_handle,
    localization::Localization,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/phone-field", || rsx! { PhoneFieldPage {} }),
    ("/phone-field/error", || rsx! { PhoneFieldErrorPage {} }),
    (
        "/phone-field/readonly",
        || rsx! { PhoneFieldReadonlyPage {} },
    ),
    ("/phone-field/german", || rsx! { PhoneFieldGermanPage {} }),
    ("/phone-field/echo", || rsx! { PhoneFieldEchoPage {} }),
    ("/phone-field/low", || rsx! { PhoneFieldLowPage {} }),
];

/// The field 320px down, where a phone's soft keyboard covers the room below it (todo 2129).
#[component]
fn PhoneFieldLowPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { style: "height: 300px" }
            PhoneField { label: "Mobile", country: "DE" }
        }
    }
}

/// Germany, the E.164 value echoed in `#echo`, for the shared web/native scenarios.
#[component]
fn PhoneFieldEchoPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PhoneField {
                label: "Phone",
                country: "DE",
                value: value(),
                oninput: move |next: String| value.set(next),
            }
            Text { id: "echo", "{value}" }
        }
    }
}

/// The German country names, which sort elsewhere than the English ones.
#[component]
fn PhoneFieldGermanPage() -> Element {
    let localization = use_localization_handle();
    use_effect(move || localization.set(&Localization::GERMAN));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PhoneField { label: "Mobile", country: "DE" }
        }
    }
}

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

#[component]
fn PhoneFieldReadonlyPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PhoneField { label: "Mobile", country: "DE", readonly: true }
        }
    }
}
