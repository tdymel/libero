//! `RadioGroup`, for the `RadioSet` archetype.

use dioxus::prelude::*;
use libero::components::{FieldStatus, Flex, OptionList, Options, Radio, RadioGroup, Text};

use crate::{Routes, common::Between};

pub const ROUTES: Routes = &[
    ("/radio-group", || rsx! { RadioGroupPage {} }),
    ("/radio-group/field", || rsx! { RadioGroupFieldPage {} }),
    ("/radio-group/echo", || rsx! { RadioGroupEchoPage {} }),
    ("/radio-group/empty", || rsx! { RadioGroupEmptyPage {} }),
    ("/radio-group/ignored", || rsx! { RadioGroupIgnoredPage {} }),
    (
        "/radio-group/readonly",
        || rsx! { RadioGroupReadonlyPage {} },
    ),
];

#[component]
fn RadioGroupReadonlyPage() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                RadioGroup {
                    label: "Plan",
                    readonly: true,
                    value: plan(),
                    onchange: move |next| plan.set(Some(next)),
                }
                // A hand-laid radio: `required` and `readonly` on the radio itself.
                Radio {
                    label: "Terms",
                    required: true,
                    readonly: true,
                    checked: false,
                    onselect: |_| {},
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Plan {
    Free,
    Pro,
    Team,
}

/// `RadioGroupPage` with the pick echoed in `#picked`, for the shared
/// web/native scenarios.
#[component]
fn RadioGroupEchoPage() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                RadioGroup {
                    label: "Plan",
                    value: plan(),
                    onchange: move |next| plan.set(Some(next)),
                }
            }
            Text { id: "picked", "{plan:?}" }
        }
    }
}

/// Starts on the **second** option, so "Tab enters at the checked radio" differs from
/// "Tab enters at the first radio".
#[component]
fn RadioGroupPage() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                RadioGroup {
                    label: "Plan",
                    value: plan(),
                    onchange: move |next| plan.set(Some(next)),
                }
            }
        }
    }
}

/// The docs page's switches on at once: cards with descriptions in a row,
/// every caption, an error, `required` and a disabled option.
#[component]
fn RadioGroupFieldPage() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "640px",
            Between {
                RadioGroup {
                    label: "Plan",
                    description: "What your seats cost.",
                    helper: "You can change it later.",
                    status: FieldStatus::Error("Pick a plan to continue.".into()),
                    required: true,
                    variant: "card",
                    orientation: "horizontal",
                    option_description: |plan: Plan| format!("About {}", plan.label()),
                    options: OptionList::from_options().disabling(|plan| *plan == Plan::Team),
                    value: plan(),
                    onchange: move |next| plan.set(Some(next)),
                }
            }
        }
    }
}

/// A parent that never applies the pick, as one awaiting a confirmation does (todo 2443).
#[component]
fn RadioGroupIgnoredPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                RadioGroup {
                    label: "Plan",
                    value: Some(Plan::Free),
                    onchange: move |_: Plan| {},
                }
            }
        }
    }
}

/// Nothing selected and the first option disabled: Tab has to enter at the
/// first option that can be picked.
#[component]
fn RadioGroupEmptyPage() -> Element {
    let mut plan = use_signal(|| None::<Plan>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                RadioGroup {
                    label: "Plan",
                    options: OptionList::from_options().disabling(|plan| *plan == Plan::Free),
                    value: plan(),
                    onchange: move |next| plan.set(Some(next)),
                }
            }
        }
    }
}
