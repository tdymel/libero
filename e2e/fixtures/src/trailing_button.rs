//! Clearable fields holding a value, between two buttons (todo 620): the x is a
//! trailing button inside the field, and Tab can leave the field from it.

use dioxus::prelude::*;
use libero::components::{
    Autocomplete, Button, Cascader, CascaderOption, Flex, MultiSelect, Select, TagsField,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/trailing-button/select", || rsx! { SelectPage {} }),
    (
        "/trailing-button/multi-select",
        || rsx! { MultiSelectPage {} },
    ),
    (
        "/trailing-button/autocomplete",
        || rsx! { AutocompletePage {} },
    ),
    ("/trailing-button/cascader", || rsx! { CascaderPage {} }),
    ("/trailing-button/tags-field", || rsx! { TagsFieldPage {} }),
    (
        "/trailing-button/unlabelled",
        || rsx! { UnlabelledPage {} },
    ),
];

/// Named by `aria-label` alone: no label for the x to borrow (1498).
#[component]
fn UnlabelledPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));
    rsx! {
        Around {
            Select {
                "aria-label": "Fruit",
                clearable: true,
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

#[component]
fn Around(children: Element) -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            {children}
            Button { id: "after", "After" }
        }
    }
}

#[component]
fn SelectPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));
    rsx! {
        Around {
            Select {
                label: "Fruit",
                clearable: true,
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

#[component]
fn MultiSelectPage() -> Element {
    let mut value = use_signal(|| vec![Fruit::Cherry]);
    rsx! {
        Around {
            MultiSelect {
                label: "Fruit",
                clearable: true,
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

#[component]
fn AutocompletePage() -> Element {
    let mut value = use_signal(|| "B".to_string());
    rsx! {
        Around {
            Autocomplete {
                label: "City",
                clearable: true,
                options: ["Berlin", "Bern", "Bonn"].map(String::from).to_vec(),
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
}

#[component]
fn CascaderPage() -> Element {
    let mut place = use_signal(|| Some("paris".to_string()));
    rsx! {
        Around {
            Cascader {
                label: "Place",
                clearable: true,
                data: vec![
                    CascaderOption::new("france", "France").children(vec![
                        CascaderOption::new("paris", "Paris"),
                        CascaderOption::new("lyon", "Lyon"),
                    ]),
                ],
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
        }
    }
}

#[component]
fn TagsFieldPage() -> Element {
    let mut topics = use_signal(|| vec!["rust".to_string()]);
    rsx! {
        Around {
            TagsField {
                label: "Topics",
                clearable: true,
                suggestions: ["rust", "dioxus", "wasm"].map(String::from).to_vec(),
                value: topics(),
                onchange: move |next| topics.set(next),
            }
        }
    }
}
