use dioxus::prelude::*;
use libero::components::{Button, Dialog, Flex, Modal, Text, Title};

#[component]
pub fn ModalPage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Modal" }
                Text {
                    "A focus-trapped, dimmed layer that locks scroll. No opinion on content - "
                    "pair it with Dialog for role/aria-modal semantics."
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Example" }
                Button { variant: "outlined", onclick: move |_| open.set(true), "Open modal" }
                if open() {
                    Modal {
                        onclose: move |_| open.set(false),
                        Dialog {
                            aria_label: "Example modal",
                            Title { size: "lg", "Example modal" }
                            Text { "Closes on Escape or by clicking the backdrop." }
                            Button { variant: "outlined", onclick: move |_| open.set(false), "Close" }
                        }
                    }
                }
            }
        }
    }
}
