//! `use_back`: a three-step counter Back steps down, and a menu opened over it.

use dioxus::prelude::*;
use libero::components::{Button, Menu, MenuItem, Text, use_menu};
use libero::hooks::use_back;

use crate::Routes;

pub const ROUTES: Routes = &[("/use-back", || rsx! { UseBackPage {} })];

#[component]
fn UseBackPage() -> Element {
    let mut step = use_signal(|| 1);
    use_back(step() > 1, Callback::new(move |()| step -= 1));
    let menu = use_menu();
    rsx! {
        Text { id: "step", "{step}" }
        Button { id: "next", onclick: move |_| step += 1, "Next" }
        Menu { state: menu, items: vec![MenuItem::new("Copy").into()],
            Button { id: "more", "More" }
        }
    }
}
