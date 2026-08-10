use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::HelloWorld,
    theme::{HexColor, Sizes, Theme},
};

const THEME: Theme = Theme::new(
    Sizes::new(4, 8, 12, 16, 20),
    HexColor::new(0x228BE6),
    HexColor::new(0xE03131),
);

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        LiberoProvider { theme: &THEME,
            HelloWorld {}
        }
    }
}
