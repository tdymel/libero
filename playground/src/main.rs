use dioxus::prelude::*;
use libero::{Button, LiberoProvider, sx};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        LiberoProvider {
            div {
                h1 { "libero playground" }
                Button { sx: sx().width(200).background_color("yellow") }
            }
        }
    }
}
