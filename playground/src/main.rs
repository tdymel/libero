use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Box, HelloWorld},
    sx::sx,
    theme::{HexColor, Sizes, Theme},
};

const THEME: Theme = Theme::new(
    Sizes::new(4, 8, 12, 16, 20),
    HexColor::new(0x228BE6),
    HexColor::new(0xE03131),
);

const BOX_SX: libero::sx::Sx = sx().padding_top("xl").build();

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut show_hello_world = use_signal(|| true);

    rsx! {
        LiberoProvider { theme: &THEME,
            button {
                onclick: move |_| show_hello_world.toggle(),
                if show_hello_world() { "Hide Hello World" } else { "Show Hello World" }
            }

            Box { sx: &BOX_SX,
                if show_hello_world() {
                    HelloWorld {}
                }
                HelloWorld {}
            }
        }
    }
}
