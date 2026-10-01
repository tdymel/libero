//! `Select` for the combobox archetype. Not `searchable`: that moves DOM focus into a search
//! box, another pattern. `/select/field` is the searchable one.

use dioxus::prelude::*;
use libero::components::{FieldStatus, Flex, OptionItem, OptionList, Select, Text};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    (
        "/select/refused-first",
        || rsx! { SelectRefusedFirstPage {} },
    ),
    ("/select", || rsx! { SelectPage {} }),
    ("/select/field", || rsx! { SelectFieldPage {} }),
    ("/select/readonly", || rsx! { SelectReadonlyPage {} }),
    ("/select/unlabelled", || rsx! { SelectUnlabelledPage {} }),
    ("/select/refused", || rsx! { SelectRefusedPage {} }),
    ("/select/echo", || rsx! { SelectEchoPage {} }),
    ("/select/outside", || rsx! { SelectOutsidePage {} }),
];

/// Searchable, with a field above it to click while the list is open below (1497).
#[component]
fn SelectOutsidePage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            input { id: "outside", "aria-label": "Note" }
            Select {
                label: "Fruit",
                searchable: true,
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

/// Apple picked, the value echoed in `#picked`, for the shared web/native scenarios.
#[component]
fn SelectEchoPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Apple));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select {
                label: "Fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
            Text { id: "picked", "{value:?}" }
        }
    }
}

/// Controlled, and the caller refuses `Cherry`: the select must stay on the
/// old value, form value included (842).
#[component]
fn SelectRefusedPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select {
                label: "Fruit",
                name: "fruit",
                value: value(),
                onchange: move |next| {
                    if next != Some(Fruit::Cherry) {
                        value.set(next);
                    }
                },
            }
        }
    }
}

/// Searchable and named by the caller's `aria-label` alone: the open search
/// box takes the name over from the trigger.
#[component]
fn SelectUnlabelledPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select {
                "aria-label": "Fruit",
                searchable: true,
                search_placeholder: "Search fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

#[component]
fn SelectPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select {
                label: "Fruit",
                placeholder: "Pick a fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

/// Every caption, an error, `required`, `clearable`, `searchable`, groups and
/// a disabled row: the docs page's switches, all on at once.
#[component]
fn SelectFieldPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));
    let options = OptionList::grouped()
        .group(
            "Orchard",
            [
                Fruit::Apple.into(),
                OptionItem::new(Fruit::Cherry).disabled(true),
            ],
        )
        .group(
            "Other",
            [Fruit::Banana, Fruit::Damson, Fruit::Elderberry].map(OptionItem::new),
        );

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select {
                label: "Fruit",
                description: "Delivered with your next box.",
                helper: "You can swap it until Friday.",
                status: FieldStatus::Error("Pick a fruit.".into()),
                required: true,
                clearable: true,
                searchable: true,
                search_placeholder: "Search fruit",
                placeholder: "Pick a fruit",
                options,
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

#[component]
fn SelectReadonlyPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select {
                label: "Fruit",
                readonly: true,
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

/// Apple, the first row, is refused: the list opens on Banana (1271).
#[component]
fn SelectRefusedFirstPage() -> Element {
    let mut value = use_signal(|| None::<Fruit>);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select {
                label: "Fruit",
                options: OptionList::from_options().disabling(|f| matches!(f, Fruit::Apple)),
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}
