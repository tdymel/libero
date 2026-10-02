//! `TagsField` for the combobox archetype, with `suggestions`: without them there is no listbox.

use dioxus::prelude::*;
use libero::components::{Flex, TagRejection, TagsField, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tags-field", || rsx! { TagsFieldPage {} }),
    ("/tags-field/cursor", || rsx! { TagsCursorPage {} }),
    ("/tags-field/echo", || rsx! { TagsEchoPage {} }),
    ("/tags-field/reject", || rsx! { TagsRejectPage {} }),
];

/// Todo 1947: room for one tag; a change clears `#rejected`, a rejection fills it.
#[component]
fn TagsRejectPage() -> Element {
    let mut topics = use_signal(Vec::<String>::new);
    let mut rejected = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TagsField {
                id: "reject",
                label: "Topics",
                max_tags: 1,
                value: topics(),
                onchange: move |next| {
                    topics.set(next);
                    rejected.set(String::new());
                },
                onreject: move |rejection: TagRejection| {
                    rejected.set(format!("{}:{:?}", rejection.tag, rejection.reason));
                },
            }
            Text { id: "rejected", "{rejected}" }
        }
    }
}

/// The `/tags-field` field with its value echoed in `#echo`, for the shared
/// web/native scenarios.
#[component]
fn TagsEchoPage() -> Element {
    let mut topics = use_signal(|| vec!["rust".to_string()]);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TagsField {
                label: "Topics",
                suggestions: ["rust", "dioxus", "wasm", "css", "html"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                value: topics(),
                onchange: move |next| topics.set(next),
            }
            Text { id: "echo", "{topics:?}" }
        }
    }
}

/// One tag held among five suggestions, so the open snapshot shows four rows: held tags drop out.
#[component]
fn TagsFieldPage() -> Element {
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

/// Four tags and no suggestions, for the chip cursor.
#[component]
fn TagsCursorPage() -> Element {
    let mut topics = use_signal(|| {
        ["rust", "dioxus", "wasm", "css"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TagsField {
                id: "cursor",
                label: "Topics",
                value: topics(),
                onchange: move |next| topics.set(next),
            }
        }
    }
}
