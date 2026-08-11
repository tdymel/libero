use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{HelloWorld, Stack},
    sx::{Sx, sx},
    theme::{HexColor, Sizes, Theme},
};

const THEME: Theme = Theme::new(
    Sizes::new(4, 8, 12, 16, 20),
    HexColor::new(0x228BE6),
    HexColor::new(0xE03131),
    HexColor::new(0x000000),
    HexColor::new(0xFFFFFF),
);

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
    let gap = if large_gap() { "xl" } else { "sm" }.to_string();

    rsx! {
        LiberoProvider { theme: &THEME,
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

            Stack {
                sx: &BOX_SX,
                gap: gap,
                states: vec![("hidden", !show_hello_world())],
                if show_hello_world() {
                    HelloWorld {}
                }
                HelloWorld {}
            }
        }
    }
}
