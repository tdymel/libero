use dioxus::prelude::*;
use libero::{Button, LiberoProvider};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        LiberoProvider {
            div {
                h1 { "libero playground" }
                Button {}
            }
        }
    }
}
