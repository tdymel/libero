//! `Dialog` outside a modal, `Paper` and `VisuallyHidden`: the plain surfaces.

use dioxus::prelude::*;
use libero::{
    components::{Anchor, Dialog, Icon, Paper, Text, VisuallyHidden},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/dialog", || rsx! { DialogPage {} }),
    ("/dialog/long-title", || rsx! { LongTitlePage {} }),
];

/// Todo 2947: one word in the title wider than a 320px dialog.
#[component]
fn LongTitlePage() -> Element {
    rsx! {
        Dialog {
            id: "long",
            title: "Donaudampfschifffahrtselektrizitaetenhauptbetriebswerkbauunterbeamtengesellschaft",
            onclose: |_| {},
            Text { "Narrow the list." }
        }
    }
}

#[component]
fn DialogPage() -> Element {
    let mut open = use_signal(|| true);
    rsx! {
        div { style: "position: relative",
            VisuallyHidden { sx: sx().position("absolute"),
                a { id: "skip", href: "#main", "Skip to content" }
            }
        }
        if open() {
            Dialog {
                id: "inline",
                title: "Filters",
                onclose: move |_| open.set(false),
                Text { "Narrow the list." }
            }
        }
        Paper { id: "paper", bordered: true, sx: sx().padding("md"), "Bordered paper" }
        Paper { id: "card", component: "a", href: "#card", sx: sx().padding("md"), "Card link" }
        Icon { id: "icon",
            svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "8" } }
        }
        Text { id: "main",
            Anchor { to: "https://example.com", "Read more" VisuallyHidden { " about filters" } }
        }
    }
}
