use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Switch, Text};

#[component]
pub fn SwitchPage() -> Element {
    let mut notifications = use_signal(|| true);
    let mut colors = use_signal(Vec::<String>::new);

    rsx! {
        DocPage {
            title: "Switch",
            lead: rsx! {
                Text {
                    "A checkbox styled as a track and thumb. A visually hidden "
                    Code { source: "input" }
                    " does the real work, so it is announced as a switch and Space toggles it."
                }
            },
            DocSection {
                title: "Basic",
                Text {
                    "Strictly controlled: "
                    Code { source: "checked" }
                    " drives the look, "
                    Code { source: "onchange" }
                    " reports the value it should take next. The click is cancelled, so the "
                    "switch only moves when its state does."
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    align: "center",
                    Switch {
                        checked: notifications(),
                        onchange: move |next| notifications.set(next),
                        "Notifications"
                    }
                }
                Text { "Notifications: {notifications()}" }
            }
            DocSection {
                title: "Sizes",
                Flex {
                    direction: "column",
                    gap: "md",
                    Switch { size: "xs", "Extra small" }
                    Switch { size: "sm", "Small" }
                    Switch { size: "md", "Medium" }
                    Switch { size: "lg", "Large" }
                    Switch { size: "xl", "Extra large" }
                }
            }
            DocSection {
                title: "Colors",
                Flex {
                    direction: "row",
                    gap: "md",
                    for color in ["primary", "success", "error", "warning"] {
                        Switch {
                            key: "{color}",
                            color: "{color}",
                            checked: colors().iter().any(|c| c == color),
                            onchange: move |next: bool| {
                                colors
                                    .with_mut(|colors| {
                                        if next {
                                            colors.push(color.to_string());
                                        } else {
                                            colors.retain(|c| c != color);
                                        }
                                    });
                            },
                            "{color}"
                        }
                    }
                }
            }
            DocSection {
                title: "Radius",
                Text {
                    "Independent of "
                    Code { source: "size" }
                    "; the thumb stays a circle."
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    align: "center",
                    Switch { radius: "xs", checked: true, onchange: move |_| {}, "Square" }
                    Switch { radius: "xl", checked: true, onchange: move |_| {}, "Pill" }
                }
            }
            DocSection {
                title: "Without a label",
                Text {
                    "Give it an "
                    Code { source: "aria_label" }
                    " - "
                    Code { source: "attributes" }
                    " land on the root, not on the input that carries the role."
                }
                Switch { aria_label: "Airplane mode", checked: true, onchange: move |_| {} }
            }
            DocSection {
                title: "Disabled",
                Flex {
                    direction: "column",
                    gap: "md",
                    Switch { disabled: true, "Off" }
                    Switch { disabled: true, checked: true, onchange: move |_| {}, "On" }
                }
            }
        }
    }
}
