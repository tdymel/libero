//! `Title` with one word wider than a phone, alone and in a `Dialog` header.

use dioxus::prelude::*;
use libero::components::{Dialog, Text, Title};

use crate::Routes;

pub const ROUTES: Routes = &[("/title", || rsx! { TitlePage {} })];

const LONG: &str =
    "Donaudampfschifffahrtselektrizitaetenhauptbetriebswerkbauunterbeamtengesellschaft";

#[component]
fn TitlePage() -> Element {
    rsx! {
        Title { id: "long", size: "xxl", {LONG} }
        Dialog { id: "dialog", title: LONG, onclose: |_| {},
            Text { "Narrow the list." }
        }
    }
}
