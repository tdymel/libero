use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Anchor, Button, Code, CodeBlock, Flex, Kbd, Text, use_notifications};

const SAVE: &str = r#"#[component]
fn SaveButton() -> Element {
    let notify = use_notifications();

    rsx! {
        Button { variant: "outlined", onclick: move |_| { notify.show("Saved."); }, "Save" }
    }
}"#;

/// `SAVE`, rendered.
#[component]
fn SaveButton() -> Element {
    let notify = use_notifications();

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                notify.show("Saved.");
            },
            "Save"
        }
    }
}

#[component]
pub fn UseNotificationsPage() -> Element {
    rsx! {
        DocPage {
            title: "use_notifications",
            source: "libero/src/components/feedback/notifications.rs",
            markdown: "/md/use_notifications.md",
            lead: rsx! {
                Text {
                    Code { source: "use_notifications() -> NotificationHandle<NotificationData>" }
                    " shows notifications drawn as an "
                    Code { source: "Alert" }
                    ". They appear in the "
                    Code { source: "Notifications" }
                    " host placed once near the root. "
                    Anchor { to: Route::NotificationsPage {}, "Notifications" }
                    " covers the host, placement, timing and titles."
                }
            },

            DocSection {
                title: "Usage",
                Flex { align: "flex-start", SaveButton {} }
                CodeBlock { source: SAVE, language: "rust" }
                Text {
                    Code { source: "show" }
                    " takes a message, or a "
                    Code { source: "NotificationData" }
                    " with a title, a colour and an icon, and returns an id for "
                    Code { source: "update" }
                    " and "
                    Code { source: "hide" }
                    "."
                }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "A notification is announced politely unless its options say "
                    Code { source: "Assertive" }
                    ". Showing one never moves focus. "
                    Kbd { "F8" }
                    " focuses the newest one from anywhere."
                }
            }
        }
    }
}
