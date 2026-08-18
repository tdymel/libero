use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Button, Drawer, Flex, Text, Title},
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
                    "A panel anchored to one edge - Temporary is a modal drawer (portaled, "
                    "dimmed, focus-trapped); Static is a plain in-flow panel, e.g. a sidebar."
                }
            },
            DocSection {
                title: "Static",
                Flex {
                    direction: "row",
                    sx: sx().height("120px").border("1px solid").border_color("grey.3"),
                    Drawer {
                        variant: "static",
                        anchor: "left",
                        size: "xs",
                        sx: sx().padding("12px"),
                        Text { "Static panel" }
                    }
                    Flex {
                        direction: "column",
                        sx: sx().flex("1").padding("12px"),
                        Text { "Rest of the layout" }
                    }
                }
            }
            DocSection {
                title: "Temporary",
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
