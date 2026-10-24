//! `TextField`, `PasswordField` and `Textarea`: label, caption and status wiring, rules on blur,
//! the reveal toggle. One field per `data-case`.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{
        ActionIcon, ChronoField, Fieldset, Flex, Form, Icon, Options, PasswordField, Rule, Select, SvgData, Text,
        TextField, Textarea, not_empty,
    },
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/text-field", || rsx! { TextFieldPage {} }),
    ("/text-field/form", || rsx! { PasswordFormPage {} }),
    ("/text-field/echo", || rsx! { TextFieldEchoPage {} }),
    ("/text-field/upper", || rsx! { TextFieldUpperPage {} }),
    ("/text-field/password-echo", || rsx! { PasswordEchoPage {} }),
    ("/text-field/slot-icons", || rsx! { SlotIconsPage {} }),
    ("/text-field/field-icons", || rsx! { FieldIconsPage {} }),
];

const MAIL: SvgData = SvgData::new(
    r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="5" width="18" height="14" rx="2"/></svg>"#,
);

/// Todo 4113: a trailing `Icon` beside the reveal button, per field size, in both directions.
#[component]
fn SlotIconsPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            for dir in ["ltr", "rtl"] {
                for (size, step) in [("xs", "xs"), ("md", "sm"), ("xl", "md")] {
                    div { dir, "data-case": "{dir}-{size}",
                        TextField { label: "Email", r#type: "email", size, trailing: rsx! { Icon { svg: MAIL, variant: "standard" } } }
                        PasswordField { label: "Password", size }
                        TextField { label: "Sized", size, trailing: rsx! { Icon { svg: MAIL, size: "lg" } } }
                        TextField { label: "Leading icon", size, leading: rsx! { Icon { svg: MAIL, variant: "standard" } } }
                        TextField {
                            label: "Leading button",
                            size,
                            leading: rsx! {
                                ActionIcon { aria_label: "Search", size: step,
                                    svg { view_box: "0 0 24 24", rect { x: "3", y: "5", width: "18", height: "14" } }
                                }
                            },
                        }
                    }
                }
            }
            for (size, step) in [("xs", "xs"), ("md", "sm"), ("xl", "md")] {
                div { "data-filled": "{size}",
                    TextField { label: "Filled icon", size, trailing: rsx! { Icon { svg: MAIL } } }
                    TextField {
                        label: "Filled button",
                        size,
                        trailing: rsx! {
                            ActionIcon { aria_label: "Mail", size: step, variant: "filled",
                                svg { view_box: "0 0 24 24", rect { x: "3", y: "5", width: "18", height: "14" } }
                            }
                        },
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Options)]
enum Gender {
    Unspecified,
    Other,
}

/// Todo 4126: the trailing icon of a text field, a date field and a select, per field size.
#[component]
fn FieldIconsPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            for size in ["xs", "md", "xl"] {
                div { "data-align": "{size}",
                    TextField { label: "Name", size, trailing: rsx! { Icon { svg: MAIL, variant: "standard" } } }
                    ChronoField::<NaiveDate> {
                        label: "Birthday",
                        size,
                        trailing: rsx! { Icon { svg: MAIL, variant: "standard" } },
                    }
                    Select { label: "Gender", size, value: Gender::Unspecified, onchange: move |_| {} }
                }
            }
        }
    }
}

/// The typed value echoed in `#echo`, for the shared web/native scenarios.
#[component]
fn TextFieldEchoPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TextField { label: "Name", value: value(), oninput: move |next| value.set(next) }
            Text { id: "echo", "{value}" }
        }
    }
}

/// The app rewrites what was typed: the WebView's typed-value guard must let it land (1026).
#[component]
fn TextFieldUpperPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TextField { label: "Code", value: value(), oninput: move |next: String| value.set(next.to_uppercase()) }
        }
    }
}

#[component]
fn PasswordEchoPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            PasswordField { label: "Password", value: value(), oninput: move |next| value.set(next) }
            Text { id: "echo", "{value}" }
        }
    }
}

#[component]
fn TextFieldPage() -> Element {
    let mut handle = use_signal(|| "ada".to_string());

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { "data-case": "wired",
                TextField {
                    label: "Username",
                    description: "How other people see you.",
                    helper: "Letters and digits.",
                    status: "That name is taken.",
                    required: true,
                    "aria-describedby": "outside-note",
                    value: handle(),
                    oninput: move |next| handle.set(next),
                }
                span { id: "outside-note", "Shown on your profile." }
            }
            div { "data-case": "slots",
                TextField {
                    label: "Handle",
                    leading: rsx! { "@" },
                    trailing: rsx! { "{handle().len()}/20" },
                    value: handle(),
                    oninput: move |next| handle.set(next),
                }
            }
            div { "data-case": "validated",
                TextField { label: "Email", validate: not_empty.error("Email needed") }
            }
            div { "data-case": "password",
                PasswordField { label: "Password", helper: "At least 8 characters." }
            }
            div { "data-case": "password-xs",
                PasswordField { label: "PIN", size: "xs" }
            }
            div { "data-case": "fieldset",
                Fieldset::<String> { label: "Locked", disabled: true,
                    PasswordField { label: "Old password" }
                }
            }
            div { "data-case": "textarea",
                Textarea {
                    label: "Bio",
                    helper: "Markdown works.",
                    required: true,
                    validate: not_empty.error("Bio needed"),
                }
            }
        }
    }
}

/// On a page of its own: the other cases walk the whole tab cycle.
#[component]
fn PasswordFormPage() -> Element {
    rsx! {
        div { "data-case": "password-form",
            Form::<()> {
                PasswordField { label: "Secret" }
                button { r#type: "submit", "Send" }
                button { r#type: "reset", "Clear" }
            }
        }
    }
}
