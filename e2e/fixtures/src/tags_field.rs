//! `TagsField`, for the combobox archetype.
//!
//! With `suggestions`, deliberately: without them it renders no listbox at
//! all, which is another pattern.

use dioxus::prelude::*;
use libero::components::{Flex, TagsField};

use crate::Routes;

pub const ROUTES: Routes = &[("/tags-field", || rsx! { TagsFieldPage {} })];

/// A tag already held, and five suggestions of which one is that tag - so the
/// list shows four rows, and "anything already held drops out of the list" is
/// visible in the open state's snapshot.
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
