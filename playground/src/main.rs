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

    rsx! {
        LiberoProvider { theme: &THEME,
            button {
                onclick: move |_| {
                    show_hello_world.toggle();
                },
                if show_hello_world() { "Hide Hello World" } else { "Show Hello World" }
            }

            Stack {
                sx: &BOX_SX,
                gap: "xl".to_string(),
                states: vec![("hidden", !show_hello_world())],
                if show_hello_world() {
                    HelloWorld {}
                }
                HelloWorld {}
            }
        }
    }
}
