//! `Title` with one word wider than a phone, alone and in a `Dialog` header, and a cut
//! title at each end with a link inside.

use dioxus::prelude::*;
use libero::components::{Dialog, Text, Title};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/title", || rsx! { TitlePage {} }),
    ("/title/cut", || rsx! { CutPage {} }),
];

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

#[component]
fn CutPage() -> Element {
    rsx! {
        for cut in ["start", "end"] {
            Title { id: "cut-{cut}", order: 2, truncate: cut,
                "Read the "
                a { id: "link-{cut}", href: "#notes", "release notes" }
                " for the long list of everything that changed in this version"
            }
        }
    }
}
