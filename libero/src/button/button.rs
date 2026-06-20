use dioxus::prelude::*;

use crate::{sx, use_sx};

#[component]
pub fn Button() -> Element {
    let class = use_sx(sx().background_color("red"));

    rsx! {
        button {
            class: class,
            "Sample button"
        }
    }
}
