//! `Select` for the combobox archetype. Not `searchable`: that moves DOM focus into a search
//! box, another pattern. `/select/field` is the searchable one.

use dioxus::prelude::*;
use libero::components::{
    DropdownPart, FieldStatus, Flex, OptionItem, OptionList, Options, Parts, Select, Text,
};
use libero::sx::sx;

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
    ("/select/tall/below", || rsx! { SelectTallPage { top: "100px" } }),
    (
        "/select/tall/above",
        || rsx! { SelectTallPage { top: "calc(100vh - 100px)" } },
    ),
];

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Row {
    R01, R02, R03, R04, R05, R06, R07, R08, R09, R10,
    R11, R12, R13, R14, R15, R16, R17, R18, R19, R20,
    R21, R22, R23, R24, R25, R26, R27, R28, R29, R30,
    R31, R32, R33, R34, R35, R36, R37, R38, R39, R40,
}

/// Forty rows and no list cap, the trigger at `top`: only the room on the
/// side it lands on holds the dropdown (1739).
#[component]
fn SelectTallPage(top: &'static str) -> Element {
    let mut value = use_signal(|| None::<Row>);

    rsx! {
        div { position: "absolute", left: "16px", top, width: "240px",
            Select {
                label: "Row",
                value: value(),
                onchange: move |next| value.set(next),
                dropdown_parts: Parts::new().part(DropdownPart::Listbox, sx().max_height("none")),
            }
        }
    }
}

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
