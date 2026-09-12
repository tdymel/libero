//! `Select`, for the combobox archetype.
//!
//! Not `searchable`, deliberately: a searchable list moves DOM focus into its
//! search box, which is another pattern, and the archetype would be asserting
//! the wrong one. `/select/field` is the searchable one, with every caption.

use dioxus::prelude::*;
use libero::components::{FieldStatus, Flex, OptionItem, OptionList, Select};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/select", || rsx! { SelectPage {} }),
    ("/select/field", || rsx! { SelectFieldPage {} }),
    ("/select/readonly", || rsx! { SelectReadonlyPage {} }),
];

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
