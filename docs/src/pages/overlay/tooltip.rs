use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Button, Code, Flex, Text, Tooltip};

#[component]
pub fn TooltipPage() -> Element {
    let mut pinned = use_signal(|| false);

    rsx! {
        DocPage {
            title: "Tooltip",
            lead: rsx! {
                Text {
                    "A label that appears while its child is hovered or focused. Pure CSS - it "
                    "wraps the trigger in a "
                    Code { source: "span" }
                    " and needs no state, so there are no open/close callbacks."
                }
            },
            DocSection {
                title: "Placement",
                Flex {
                    direction: "row",
                    gap: "xl",
                    Tooltip { label: rsx! { "Above" }, Button { variant: "outlined", "Top" } }
                    Tooltip {
                        placement: "right",
                        label: rsx! { "To the right" },
                        Button { variant: "outlined", "Right" }
                    }
                    Tooltip {
                        placement: "bottom",
                        label: rsx! { "Below" },
                        Button { variant: "outlined", "Bottom" }
                    }
                    Tooltip {
                        placement: "left",
                        label: rsx! { "To the left" },
                        Button { variant: "outlined", "Left" }
                    }
                }
            }
            DocSection {
                title: "Sizes and gap",
                Text {
                    Code { source: "gap" }
                    " is rendered as transparent padding, not empty space, so the pointer can "
                    "travel from the trigger into the bubble without it closing."
                }
                Flex {
                    direction: "row",
                    gap: "xl",
                    align: "center",
                    Tooltip { size: "xs", label: rsx! { "Extra small" }, Button { size: "xs", "xs" } }
                    Tooltip { size: "md", label: rsx! { "Medium" }, Button { size: "md", "md" } }
                    Tooltip {
                        size: "xl",
                        gap: "lg",
                        label: rsx! { "Extra large, wide gap" },
                        Button { size: "xl", "xl" }
                    }
                }
            }
            DocSection {
                title: "Delays",
                Text {
                    Code { source: "open_delay" }
                    " and "
                    Code { source: "close_delay" }
                    " are milliseconds. A delay on open keeps a row of triggers quiet while the "
                    "pointer crosses it."
                }
                Flex {
                    direction: "row",
                    gap: "xl",
                    Tooltip {
                        open_delay: 500,
                        label: rsx! { "Waited half a second" },
                        Button { variant: "outlined", "Slow to open" }
                    }
                    Tooltip {
                        close_delay: 800,
                        label: rsx! { "Lingers on the way out" },
                        Button { variant: "outlined", "Slow to close" }
                    }
                }
            }
            DocSection {
                title: "Controlled",
                Text {
                    Code { source: "opened" }
                    " forces the bubble open or closed and overrides hover; leave it "
                    Code { source: "None" }
                    " for the default behaviour. "
                    Code { source: "disabled" }
                    " renders the trigger bare."
                }
                Flex {
                    direction: "row",
                    gap: "xl",
                    align: "center",
                    Tooltip {
                        opened: pinned(),
                        label: rsx! { "Pinned open" },
                        Button { variant: "outlined", "Trigger" }
                    }
                    Button { onclick: move |_| pinned.toggle(), "Toggle" }
                }
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "The wrapper is not focusable, so an "
                    Code { source: "aria-describedby" }
                    " on it would never be announced. Give the bubble an id with "
                    Code { source: "label_id" }
                    " and point your own trigger at it instead."
                }
                Tooltip {
                    label_id: "save-tip",
                    label: rsx! { "Saves the current draft" },
                    Button { aria_describedby: "save-tip", "Save" }
                }
                Text {
                    "Escape does not dismiss it, and an ancestor with "
                    Code { source: "overflow: hidden" }
                    " - a scroll container, a card - clips it. Both need measurement and state; "
                    "reach for a popover there."
                }
            }
            DocSection {
                title: "Styling",
                Text {
                    Code { source: "sx" }
                    ", "
                    Code { source: "class" }
                    ", "
                    Code { source: "states" }
                    " and spread attributes land on the bubble, not the wrapper - the bubble is "
                    "the part worth styling."
                }
                Tooltip {
                    label: rsx! { "Styled bubble" },
                    sx: libero::sx::sx().background("primary.6").white_space("normal").max_width("12rem"),
                    Button { variant: "outlined", "Hover me" }
                }
            }
        }
    }
}
