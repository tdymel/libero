//! `PhoneField`: the country picker, its searchable list and the `tel` input.

use dioxus::prelude::*;
use libero::{
    components::{
        Button, FieldStatus, Flex, Form, PhoneField, Rule, Text, not_empty, use_form,
    },
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
    ("/phone-field/rtl", || rsx! { PhoneFieldRtlPage {} }),
    ("/phone-field/later", || rsx! { PhoneFieldLaterRulesPage {} }),
    ("/phone-field/reset", || rsx! { PhoneFieldResetPage {} }),
];

/// Todo 2326: a typed number in a `Form`, reset by a button; the `name` posts the E.164.
#[component]
fn PhoneFieldResetPage() -> Element {
    let value = use_store(String::new);
    let form = use_form();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Form { value, form,
                PhoneField { label: "Phone", country: "DE", name: "phone" }
            }
            Button { id: "reset", onclick: move |_| form.reset(), "Reset" }
        }
    }
}

/// Under `dir=rtl`, the picker's dial code and a pinned one (todo 1565).
#[component]
fn PhoneFieldRtlPage() -> Element {
    rsx! {
        div { dir: "rtl",
            Flex { direction: "column", gap: "md", max_width: "320px",
                PhoneField { label: "Mobile", country: "DE" }
                PhoneField { label: "Office", country: "DE", country_select: false }
            }
        }
    }
}

/// No rules at mount, a rule after the button (todo 2368).
#[component]
fn PhoneFieldLaterRulesPage() -> Element {
    let mut strict = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { onclick: move |_| strict.set(true), "Strict" }
            PhoneField {
                label: "Mobile",
                country: "DE",
                validate: if strict() { vec![not_empty.error("Enter a number.")] } else { Vec::new() },
            }
        }
    }
}

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
