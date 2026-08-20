use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Chip, Code, Flex, Text};

#[component]
pub fn ChipPage() -> Element {
    let mut selected = use_signal(|| vec!["rust".to_string()]);

    rsx! {
        DocPage {
            title: "Chip",
            lead: rsx! {
                Text {
                    "A compact token. With "
                    Code { "onchange" }
                    " it is a real checkbox - a visually hidden "
                    Code { "input" }
                    " plus a "
                    Code { "label" }
                    ", so selection is announced and Space toggles it. Without it, a plain tag."
                }
            },
            DocSection {
                title: "Variants",
                Flex {
                    direction: "row",
                    gap: "md",
                    Chip { variant: "filled", "Filled" }
                    Chip { variant: "outlined", "Outlined" }
                    Chip { variant: "text", "Text" }
                }
            }
            DocSection {
                title: "Sizes",
                Flex {
                    direction: "row",
                    gap: "md",
                    align: "center",
                    Chip { size: "xs", "Extra small" }
                    Chip { size: "sm", "Small" }
                    Chip { size: "md", "Medium" }
                    Chip { size: "lg", "Large" }
                    Chip { size: "xl", "Extra large" }
                }
            }
            DocSection {
                title: "Colors",
                Flex {
                    direction: "row",
                    gap: "md",
                    Chip { color: "primary", "Primary" }
                    Chip { color: "success", "Success" }
                    Chip { color: "error", "Error" }
                    Chip { color: "warning", "Warning" }
                }
            }
            DocSection {
                title: "Selectable",
                Text {
                    "Strictly controlled: "
                    Code { "checked" }
                    " drives the look, "
                    Code { "onchange" }
                    " reports the value it should take next. A checked chip is filled "
                    "whatever its "
                    Code { "variant" }
                    ", so "
                    Code { "variant" }
                    " describes the unselected state."
                }
                Flex {
                    direction: "row",
                    gap: "md",
                    for language in ["rust", "css", "html"] {
                        Chip {
                            key: "{language}",
                            checked: selected().iter().any(|s| s == language),
                            onchange: move |next: bool| {
                                selected
                                    .with_mut(|selected| {
                                        if next {
                                            selected.push(language.to_string());
                                        } else {
                                            selected.retain(|s| s != language);
                                        }
                                    });
                            },
                            "{language}"
                        }
                    }
                }
                Text { "Selected: {selected():?}" }
            }
            DocSection {
                title: "Disabled",
                Flex {
                    direction: "row",
                    gap: "md",
                    Chip { variant: "filled", disabled: true, "Filled" }
                    Chip { disabled: true, "Outlined" }
                    Chip { disabled: true, checked: true, onchange: move |_| {}, "Selected" }
                }
            }
        }
    }
}
