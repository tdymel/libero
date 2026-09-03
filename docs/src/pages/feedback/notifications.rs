use std::{cell::Cell, rc::Rc, time::Duration};

use crate::components::{DocPage, DocSection};
use crate::icons::DismissIcon;
use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Button, Code, CodeBlock, Flex, NotificationData, NotificationLive,
        NotificationOptions, NotificationScope, Paper, Placement, ProgressBar, Text,
        use_notifications, use_notifications_with,
    },
    platform::{TimerSubscription, timer},
    sx::sx,
    theme::AutoClose,
};

const SETUP_EXAMPLE: &str = r#"// Once, near the root - the one outlet for every handle.
LiberoProvider {
    Router::<Route> {}
    Notifications {}
}

// Anywhere below it.
let notify = use_notifications();
notify.show("Saved.");
notify.show(NotificationData {
    title: Some("Upload failed".into()),
    message: "archive.zip is over the 10 MB limit.".into(),
    color: "error".into(),
    ..Default::default()
});
notify.show_with("Copied", NotificationOptions {
    position: Some(Placement::TopCenter),
    auto_close: Some(AutoClose::After(2000)),
    ..Default::default()
});"#;

const TEMPLATE_EXAMPLE: &str = r#"#[derive(Clone, PartialEq)]
struct Upload {
    file: &'static str,
    percent: f64,
}

// A `fn`, not a capturing closure: everything it draws travels in `Upload`.
fn upload_notification(s: NotificationScope<Upload>) -> Element {
    let upload = s.args();
    let done = upload.percent >= 100.0;

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "column", gap: "sm",
                Flex { direction: "row", align: "center", justify: "space-between",
                    Text { if done { "Uploaded {upload.file}" } else { "Uploading {upload.file}" } }
                    if done {
                        ActionIcon { variant: "standard", size: "sm", aria_label: "Dismiss",
                            onclick: move |_| s.close(), DismissIcon {} }
                    }
                }
                ProgressBar { value: upload.percent, "aria-label": "{upload.file}" }
            }
        }
    }
}

let uploads = use_notifications_with(upload_notification);
let id = uploads.show_with(Upload { file: "archive.zip", percent: 0.0 },
    NotificationOptions { auto_close: Some(AutoClose::Never), ..Default::default() });
// ...as it progresses:
uploads.update(id, Upload { file: "archive.zip", percent: 40.0 });"#;

#[derive(Clone, PartialEq)]
struct Upload {
    file: &'static str,
    percent: f64,
}

/// The page's own template - the code beside it is what drew it.
fn upload_notification(s: NotificationScope<Upload>) -> Element {
    let upload = s.args();
    let done = upload.percent >= 100.0;

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "column", gap: "sm",
                Flex {
                    direction: "row",
                    align: "center",
                    justify: "space-between",
                    Text {
                        if done {
                            "Uploaded {upload.file}"
                        } else {
                            "Uploading {upload.file}"
                        }
                    }
                    if done {
                        ActionIcon {
                            variant: "standard",
                            size: "sm",
                            aria_label: "Dismiss",
                            onclick: move |_| s.close(),
                            DismissIcon {}
                        }
                    }
                }
                ProgressBar { value: upload.percent, "aria-label": "{upload.file}" }
            }
        }
    }
}

#[component]
pub fn NotificationsPage() -> Element {
    let notify = use_notifications();
    let uploads = use_notifications_with(upload_notification);
    // Each upload's ticker. Dropped with the page, which is what stops them.
    let tickers = use_hook(|| {
        Rc::new(std::cell::RefCell::new(
            Vec::<Box<dyn TimerSubscription>>::new(),
        ))
    });

    let start_upload = move |_| {
        let id = uploads.show_with(
            Upload {
                file: "archive.zip",
                percent: 0.0,
            },
            NotificationOptions {
                auto_close: Some(AutoClose::Never),
                ..Default::default()
            },
        );
        let Some(api) = timer() else {
            return;
        };
        let percent = Cell::new(0.0);
        // The callback runs outside every scope, so it only writes - `update`
        // is a signal write and nothing else.
        let ticker = api.every(
            Duration::from_millis(300),
            Box::new(move || {
                if percent.get() < 100.0 {
                    percent.set(percent.get() + 10.0);
                    uploads.update(
                        id,
                        Upload {
                            file: "archive.zip",
                            percent: percent.get(),
                        },
                    );
                }
            }),
        );
        tickers.borrow_mut().push(ticker);
    };

    rsx! {
        DocPage {
            title: "Notifications",
            source: "libero/src/components/feedback/notifications.rs",
            markdown: "/md/notifications.md",
            lead: rsx! {
                Text {
                    "Notifications are a hook and a host. Render "
                    Code { source: "Notifications {{}}" }
                    " once near the root, and "
                    Code { source: "use_notifications()" }
                    " hands back a handle that shows them from anywhere. A notification outlives "
                    "the component that raised it."
                }
            },
            DocSection {
                title: "Try it",
                Flex {
                    direction: "column",
                    gap: "md",
                    Flex {
                        direction: "row",
                        align: "center",
                        gap: "sm",
                        wrap: "wrap",
                        Button {
                            variant: "outlined",
                            onclick: move |_| {
                                notify.show("Saved.");
                            },
                            "Show one"
                        }
                        Button {
                            variant: "outlined",
                            color: "success",
                            onclick: move |_| {
                                notify.show(NotificationData {
                                    title: Some("Published".into()),
                                    message: "Your post is live.".into(),
                                    color: "success".into(),
                                    ..Default::default()
                                });
                            },
                            "Success"
                        }
                        Button {
                            variant: "outlined",
                            color: "error",
                            onclick: move |_| {
                                notify.show_with(
                                    NotificationData {
                                        title: Some("Upload failed".into()),
                                        message: "archive.zip is over the 10 MB limit.".into(),
                                        color: "error".into(),
                                        ..Default::default()
                                    },
                                    NotificationOptions {
                                        live: NotificationLive::Assertive,
                                        ..Default::default()
                                    },
                                );
                            },
                            "Error, announced at once"
                        }
                    }
                    Flex {
                        direction: "row",
                        align: "center",
                        gap: "sm",
                        wrap: "wrap",
                        Button {
                            variant: "outlined",
                            onclick: move |_| {
                                notify.show_with(
                                    "Stays until you close it.",
                                    NotificationOptions {
                                        auto_close: Some(AutoClose::Never),
                                        ..Default::default()
                                    },
                                );
                            },
                            "Sticky"
                        }
                        Button {
                            variant: "outlined",
                            onclick: move |_| {
                                notify.show_with(
                                    "Gone in two seconds.",
                                    NotificationOptions {
                                        auto_close: Some(AutoClose::After(2000)),
                                        ..Default::default()
                                    },
                                );
                            },
                            "Two seconds"
                        }
                        Button { variant: "outlined", onclick: start_upload, "Your own template" }
                        Button { variant: "text", onclick: move |_| notify.clear(), "Clear all" }
                    }
                    Flex {
                        direction: "column",
                        gap: "xs",
                        for row in Placement::ALL.chunks(3) {
                            Flex {
                                direction: "row",
                                gap: "xs",
                                for &placement in row {
                                    Button {
                                        size: "xs",
                                        variant: "tonal",
                                        sx: sx().width("120px"),
                                        onclick: move |_| {
                                            notify.show_with(
                                                format!("From {}.", placement.as_str()),
                                                NotificationOptions {
                                                    position: Some(placement),
                                                    ..Default::default()
                                                },
                                            );
                                        },
                                        "{placement.as_str()}"
                                    }
                                }
                            }
                        }
                    }
                    Text {
                        "Hovering or focusing any notification pauses every timer, and each one "
                        "starts over from its full time when you leave. A stack shows five at once; "
                        "the rest wait their turn. Nothing here takes focus. A close button is "
                        "reached by Tab, and each stack has a polite and an assertive live region, "
                        "both mounted before anything is announced into them."
                    }
                    CodeBlock { source: SETUP_EXAMPLE, language: "rust" }
                }
            }
            DocSection {
                title: "Your own template",
                Text {
                    Code { source: "use_notifications_with" }
                    " takes your data type and a template. The template is a "
                    Code { source: "fn" }
                    ", so a notification that outlives its caller holds nothing of the caller's. "
                    Code { source: "update(id, data)" }
                    " redraws one in place: the \"Your own template\" button above uploads a file."
                }
                CodeBlock { source: TEMPLATE_EXAMPLE, language: "rust" }
            }
        }
    }
}
