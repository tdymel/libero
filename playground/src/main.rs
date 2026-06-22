use dioxus::prelude::*;
use libero::{Button, LiberoProvider, sx};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        LiberoProvider {
            div {
                h1 { "libero playground" }
                p { "Clicked {count} times" }
                Button {
                    onclick: move |_| count += 1,
                    "Click me"
                }
            }
        }
    }
}
