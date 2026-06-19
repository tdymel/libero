use dioxus::prelude::*;
use libero::Button;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            h1 { "libero playground" }
            Button {}
        }
    }
}
