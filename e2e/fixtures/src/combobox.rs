//! Fixtures for the combobox archetype beyond the `Autocomplete` pilot:
//! `Select`, `MultiSelect` and `TagsField`.
//!
//! None of them is `searchable` or free of suggestions, deliberately. A
//! searchable list moves DOM focus into its search box, and a `TagsField`
//! without `suggestions` renders no listbox at all - both are other patterns,
//! and the archetype would be asserting the wrong one.

use dioxus::prelude::*;
use libero::components::{Flex, MultiSelect, Options, Select, TagsField};

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    Damson,
    Elderberry,
}

#[component]
pub fn SelectPage() -> Element {
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

/// One chip already held, so the trigger draws the chip and its remove control
/// at rest - the part of this component that is neither a field nor a list.
#[component]
pub fn MultiSelectPage() -> Element {
    let mut value = use_signal(|| vec![Fruit::Cherry]);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            MultiSelect {
                label: "Fruit",
                placeholder: "Pick fruit",
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

/// A tag already held, and five suggestions of which one is that tag - so the
/// list shows four rows, and "anything already held drops out of the list" is
/// visible in the open state's snapshot.
#[component]
pub fn TagsFieldPage() -> Element {
    let mut topics = use_signal(|| vec!["rust".to_string()]);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TagsField {
                label: "Topics",
                placeholder: "Add a topic",
                suggestions: ["rust", "dioxus", "wasm", "css", "html"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                value: topics(),
                onchange: move |next| topics.set(next),
            }
        }
    }
}
