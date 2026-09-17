use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{
        Anchor, Button, Code, CodeBlock, Flex, NotificationScope, Paper, Text,
        use_notifications_with,
    },
    sx::sx,
};

const REMINDER: &str = r#"#[derive(Clone, PartialEq)]
struct Reminder {
    text: &'static str,
}

// A `fn`, not a capturing closure: everything it draws travels in `Reminder`.
fn reminder(s: NotificationScope<Reminder>) -> Element {
    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "row", justify: "space-between", gap: "sm",
                Text { "{s.args().text}" }
                if s.closable() {
                    Button { variant: "text", onclick: move |_| s.close(), "Dismiss" }
                }
            }
        }
    }
}

#[component]
fn RemindMe() -> Element {
    let reminders = use_notifications_with(reminder);

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| { reminders.show(Reminder { text: "Stand up and stretch." }); },
            "Remind me"
        }
    }
}"#;

#[derive(Clone, PartialEq)]
struct Reminder {
    text: &'static str,
}

/// `REMINDER`'s template, rendered.
fn reminder(s: NotificationScope<Reminder>) -> Element {
    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "row", justify: "space-between", gap: "sm",
                Text { "{s.args().text}" }
                if s.closable() {
                    Button { variant: "text", onclick: move |_| s.close(), "Dismiss" }
                }
            }
        }
    }
}

#[component]
fn RemindMe() -> Element {
    let reminders = use_notifications_with(reminder);

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                reminders
                    .show(Reminder {
                        text: "Stand up and stretch.",
                    });
            },
            "Remind me"
        }
    }
}

#[component]
pub fn UseNotificationsWithPage() -> Element {
    rsx! {
        DocPage {
            title: "use_notifications_with",
            source: "libero/src/components/feedback/notifications.rs",
            markdown: "/md/use_notifications_with.md",
            lead: rsx! {
                Text {
                    Code { source: "use_notifications_with(template) -> NotificationHandle<T>" }
                    " shows notifications of your own data type, drawn by your own template. "
                    "The host keeps the list, the live region, the timers and the hover "
                    "pause. "
                    Anchor { to: Route::NotificationsPage {}, "Notifications" }
                    " has the full story."
                }
            },

            DocSection {
                title: "Usage",
                Flex { align: "flex-start", RemindMe {} }
                CodeBlock { source: REMINDER, language: "rust" }
                Text {
                    "The template is a "
                    Code { source: "fn" }
                    ", not a closure that captures. A notification outlives the component "
                    "that raised it, so everything it draws travels in "
                    Code { source: "T" }
                    ". "
                    Code { source: "update(id, value)" }
                    " redraws one with new data, such as a progress bar."
                }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "The template draws the content only. Give it a close control when "
                    Code { source: "s.closable()" }
                    " says so, because not every reader can wait for the timer."
                }
            }
        }
    }
}
