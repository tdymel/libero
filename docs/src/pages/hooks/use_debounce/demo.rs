use dioxus::prelude::*;
use libero::{
    components::{Flex, Text, TextField},
    hooks::{use_debounced_value, use_throttled_value},
};

#[component]
pub fn LiveSearch() -> Element {
    let mut query = use_signal(String::new);
    let settled = use_debounced_value(query.into(), 400);
    let throttled = use_throttled_value(query.into(), 400);

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Search",
                value: query(),
                oninput: move |next| query.set(next),
            }
            Text { "Typed: {query}" }
            Text { "Debounced: {settled}" }
            Text { "Throttled: {throttled}" }
        }
    }
}
