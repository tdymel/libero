use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Text, ToggleButton, ToggleButtonGroup};
use std::collections::HashMap;

#[component]
pub fn ToggleButtonGroupPage() -> Element {
    let mut alignment = use_signal(|| vec!["left".to_string()]);
    let mut formats = use_signal(Vec::<String>::new);
    let mut vertical = use_signal(|| vec!["b".to_string()]);
    let mut range = use_signal(|| vec!["day".to_string()]);
    let mut disabled = use_signal(|| vec!["a".to_string()]);
    // One entry per demo group, so the looped examples below do not all move
    // together.
    let mut groups = use_signal(HashMap::<&'static str, Vec<String>>::new);

    rsx! {
        DocPage {
            title: "ToggleButtonGroup",
            lead: rsx! {
                Text {
                    "A row of connected buttons sharing one selection. Each member is a real "
                    Code { "button" }
                    " carrying "
                    Code { "aria-pressed" }
                    ", so tab order and Space/Enter are the browser's own."
                }
            },
            DocSection {
                title: "Basic",
                Text {
                    "Strictly controlled: "
                    Code { "value" }
                    " drives the look, "
                    Code { "onchange" }
                    " reports the selection the group should take next."
                }
                ToggleButtonGroup {
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                    ToggleButton { value: "left", "Left" }
                    ToggleButton { value: "center", "Center" }
                    ToggleButton { value: "right", "Right" }
                }
                Text { "Alignment: {alignment():?}" }
            }
            DocSection {
                title: "Multiple",
                Text {
                    "Selection is exclusive by default. Set "
                    Code { "exclusive: false" }
                    " to let any number of buttons be pressed at once."
                }
                ToggleButtonGroup {
                    exclusive: false,
                    value: formats(),
                    onchange: move |next| formats.set(next),
                    ToggleButton { value: "bold", "Bold" }
                    ToggleButton { value: "italic", "Italic" }
                    ToggleButton { value: "underline", "Underline" }
                }
                Text { "Formats: {formats():?}" }
            }
            DocSection {
                title: "Variants and colors",
                Text {
                    Code { "variant" }
                    " is the unselected look; a selected button keeps it and tints its background."
                }
                Flex {
                    direction: "column",
                    gap: "md",
                    align: "start",
                    for variant in ["outlined", "filled", "text"] {
                        ToggleButtonGroup {
                            key: "{variant}",
                            variant: "{variant}",
                            color: "primary",
                            value: groups().get(variant).cloned().unwrap_or_default(),
                            onchange: move |next| {
                                groups.with_mut(|groups| groups.insert(variant, next));
                            },
                            ToggleButton { value: "list", "List" }
                            ToggleButton { value: "grid", "Grid" }
                        }
                    }
                }
            }
            DocSection {
                title: "Sizes",
                Flex {
                    direction: "column",
                    gap: "md",
                    align: "start",
                    for size in ["xs", "sm", "md", "lg", "xl"] {
                        ToggleButtonGroup {
                            key: "{size}",
                            size: "{size}",
                            value: groups().get(size).cloned().unwrap_or_default(),
                            onchange: move |next| {
                                groups.with_mut(|groups| groups.insert(size, next));
                            },
                            ToggleButton { value: "a", "A" }
                            ToggleButton { value: "b", "B" }
                            ToggleButton { value: "c", "C" }
                        }
                    }
                }
            }
            DocSection {
                title: "Vertical",
                Text {
                    Code { "orientation: \"vertical\"" }
                    " stacks the buttons and moves the shared corners to the top and bottom."
                }
                ToggleButtonGroup {
                    orientation: "vertical",
                    value: vertical(),
                    onchange: move |next| vertical.set(next),
                    ToggleButton { value: "a", "Top" }
                    ToggleButton { value: "b", "Middle" }
                    ToggleButton { value: "c", "Bottom" }
                }
            }
            DocSection {
                title: "Full width",
                Text {
                    Code { "full_width" }
                    " spreads the group across its container. A horizontal group shares the width "
                    "out evenly; a vertical one is already as wide as its widest button."
                }
                ToggleButtonGroup {
                    full_width: true,
                    value: range(),
                    onchange: move |next| range.set(next),
                    ToggleButton { value: "day", "Day" }
                    ToggleButton { value: "week", "Week" }
                    ToggleButton { value: "month", "Month" }
                }
            }
            DocSection {
                title: "Disabled",
                Text {
                    Code { "disabled" }
                    " on the group covers every button; a single "
                    Code { "ToggleButton" }
                    " can set its own."
                }
                Flex {
                    direction: "column",
                    gap: "md",
                    align: "start",
                    ToggleButtonGroup {
                        disabled: true,
                        value: vec!["a".to_string()],
                        onchange: move |_| {},
                        ToggleButton { value: "a", "One" }
                        ToggleButton { value: "b", "Two" }
                    }
                    ToggleButtonGroup {
                        value: disabled(),
                        onchange: move |next| disabled.set(next),
                        ToggleButton { value: "a", "One" }
                        ToggleButton { value: "b", disabled: true, "Two" }
                    }
                }
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "The group is a "
                    Code { "role=\"group\"" }
                    " of ordinary buttons, each carrying "
                    Code { "aria-pressed" }
                    ". Tab reaches every button and Space or Enter toggles it - there is no "
                    "arrow-key navigation to learn, and none is added."
                }
                Text {
                    "For a single-select group, APG would prefer a "
                    Code { "radiogroup" }
                    ". Following MUI, this component stays with pressed buttons: the roving "
                    "tabindex a radiogroup requires costs more than it returns here, and a "
                    "toggle button reads correctly to a screen reader either way. Name the group "
                    "with an "
                    Code { "aria_label" }
                    " where its purpose is not obvious from the buttons themselves."
                }
                ToggleButtonGroup {
                    aria_label: "Text alignment",
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                    ToggleButton { value: "left", "Left" }
                    ToggleButton { value: "center", "Center" }
                    ToggleButton { value: "right", "Right" }
                }
            }
        }
    }
}
