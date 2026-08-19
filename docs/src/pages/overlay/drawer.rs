use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Button, Drawer, Text, Title},
    sx::sx,
};

#[component]
pub fn DrawerPage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        DocPage {
            title: "Drawer",
            lead: rsx! {
                Text {
                    "A portaled, dimmed, focus-trapped panel docked to one edge, closing on "
                    "Escape or a backdrop click. For an in-flow panel, see Sidebar."
                }
            },
            DocSection {
                title: "Usage",
                Button { variant: "outlined", onclick: move |_| open.set(true), "Open drawer" }
                if open() {
                    Drawer {
                        anchor: "right",
                        onclose: move |_| open.set(false),
                        sx: sx().padding("16px"),
                        Title { size: "lg", "Temporary drawer" }
                        Text { "Closes on Escape or backdrop click." }
                        Button { variant: "outlined", onclick: move |_| open.set(false), "Close" }
                    }
                }
            }
        }
    }
}
