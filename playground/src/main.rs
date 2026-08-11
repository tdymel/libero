use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Group, HelloWorld, Stack},
    sx::{Sx, sx},
};

const BOX_SX: Sx = sx()
    .padding_top("xl")
    .when("hidden", sx().background("secondary.1"))
    .build();

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut show_hello_world = use_signal(|| true);
    let mut large_gap = use_signal(|| true);
    let mut wrap_group = use_signal(|| true);
    let gap = if large_gap() { "xl" } else { "sm" }.to_string();

    rsx! {
        LiberoProvider {
            button {
                onclick: move |_| {
                    show_hello_world.toggle();
                },
                if show_hello_world() { "Hide Hello World" } else { "Show Hello World" }
            }

            button {
                onclick: move |_| {
                    large_gap.toggle();
                },
                if large_gap() { "Use small gap" } else { "Use large gap" }
            }

            button {
                onclick: move |_| {
                    wrap_group.toggle();
                },
                if wrap_group() { "Disable group wrap" } else { "Enable group wrap" }
            }

            Stack {
                sx: &BOX_SX,
                gap: gap.clone(),
                states: vec![("hidden", !show_hello_world())],
                if show_hello_world() {
                    HelloWorld {}
                }
                HelloWorld {}
            }

            Group {
                gap: gap,
                wrap: wrap_group(),
                for index in 1..=8 {
                    button { "Group item {index}" }
                }
            }
        }
    }
}
