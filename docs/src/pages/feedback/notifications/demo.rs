use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Avatar, Button, Flex, NotificationScope, Paper, Pictogram, ProgressBar, Text,
};
use libero::sx::sx;
use pictogram_icons_lucide as lucide;

// demo-code: card start
#[derive(Clone, PartialEq)]
pub struct Message {
    pub sender: &'static str,
    pub initials: &'static str,
    pub title: Option<String>,
    pub text: &'static str,
    pub onaction: Callback<String>,
}

// A `fn`, not a capturing closure. Everything it draws and calls travels in `Message`.
pub fn card_notification(s: NotificationScope<Message>) -> Element {
    let data = s.args();
    let (onaction, sender) = (data.onaction, data.sender);

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "row", align: "start", gap: "md",
                Avatar { name: data.sender, initials: data.initials, color: "primary" }
                Flex { direction: "column", gap: "sm", sx: sx().flex("1").min_width("0"),
                    Flex { direction: "row", align: "start", justify: "space-between", gap: "sm",
                        Flex { direction: "column", gap: "xs",
                            if let Some(title) = data.title {
                                Text { sx: sx().font_weight("600"), "{title}" }
                            }
                            Text { "{data.text}" }
                        }
                        if s.closable() {
                            ActionIcon {
                                variant: "standard",
                                size: "sm",
                                aria_label: "Dismiss",
                                onclick: move |_| s.close(),
                                Pictogram { icon: lucide::x::outlined }
                            }
                        }
                    }
                    Flex { direction: "row", gap: "sm",
                        Button {
                            variant: "outlined",
                            size: "xs",
                            onclick: move |_| {
                                onaction.call(format!("Replying to {sender}."));
                                s.close()
                            },
                            "Reply"
                        }
                        Button {
                            variant: "standard",
                            size: "xs",
                            onclick: move |_| {
                                onaction.call(format!("Muted {sender}."));
                                s.close()
                            },
                            "Mute"
                        }
                    }
                }
            }
        }
    }
}
// demo-code: card end

// demo-code: upload start
#[derive(Clone, PartialEq)]
pub struct Upload {
    pub file: &'static str,
    pub percent: f64,
}

// A `fn`, not a capturing closure. Everything it draws travels in `Upload`.
pub fn upload_notification(s: NotificationScope<Upload>) -> Element {
    let upload = s.args();
    let done = upload.percent >= 100.0;

    rsx! {
        Paper { shadow: "md", sx: sx().padding("md"),
            Flex { direction: "column", gap: "sm",
                Flex { direction: "row", align: "center", justify: "space-between",
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
                            Pictogram { icon: lucide::x::outlined }
                        }
                    }
                }
                ProgressBar { value: upload.percent, "aria-label": "{upload.file}" }
            }
        }
    }
}
// demo-code: upload end
