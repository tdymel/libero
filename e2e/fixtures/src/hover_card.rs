//! `HoverCard`.

use dioxus::prelude::*;
use libero::components::{Button, ColorCode, ColorField, Flex, HoverCard, Select, Text};
use libero::hooks::Side;

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/hover-card", || rsx! { HoverCardPage {} }),
    (
        "/hover-card-color-field",
        || rsx! { ColorFieldInCardPage {} },
    ),
    ("/hover-card-disable", || rsx! { DisableWhileOpenPage {} }),
    ("/hover-card-toggle", || rsx! { ToggleDisabledPage {} }),
    ("/hover-card-text", || rsx! { TextTriggerPage {} }),
    ("/hover-card-sides", || rsx! { SidesPage {} }),
    ("/hover-card-select", || rsx! { SelectInCardPage {} }),
    ("/hover-card-pair", || rsx! { PairTriggerPage {} }),
    ("/hover-card-scroll", || rsx! { ScrollingTextPage {} }),
];

/// A card of text taller than the room beside the trigger (todo 2445).
#[component]
fn ScrollingTextPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            HoverCard {
                aria_label: "Ada Lovelace",
                content: rsx! {
                    for line in 0..80 {
                        Text { "Line {line} of a long biography that scrolls inside the card." }
                    }
                },
                Button { id: "trigger", variant: "outlined", "Ada Lovelace" }
            }
            Button { id: "after", "After" }
        }
    }
}

/// A trigger holding two buttons: the card sits after the second (todo 1614).
#[component]
fn PairTriggerPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            HoverCard {
                aria_label: "Ada Lovelace",
                content: rsx! {
                    Flex { direction: "column", gap: "xs",
                        Button { id: "card-first", variant: "outlined", "Profile" }
                        Button { id: "card-last", variant: "outlined", "Follow" }
                    }
                },
                Flex { gap: "xs",
                    Button { id: "trigger", variant: "outlined", "Ada Lovelace" }
                    Button { id: "trigger-second", variant: "outlined", "Remove" }
                }
            }
            Button { id: "after", "After" }
        }
    }
}

/// A `Select` in the card, whose open list is a layer of its own (todo 348).
#[component]
fn SelectInCardPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Apple));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            HoverCard {
                aria_label: "Ada Lovelace",
                open_delay: 10,
                close_delay: 10,
                content: rsx! {
                    Select { label: "Fruit", value: value(), onchange: move |next| value.set(next) }
                },
                Button { id: "trigger", "Ada Lovelace" }
            }
        }
    }
}

/// A plain-text trigger, which no keyboard can focus (todo 523).
#[component]
fn TextTriggerPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            HoverCard {
                aria_label: "Ada Lovelace",
                content: rsx! { Text { "Wrote the first algorithm meant for a machine." } },
                Text { id: "trigger", "Ada Lovelace" }
            }
        }
    }
}

/// A card forced open, and a button that disables it (todo 449).
#[component]
fn DisableWhileOpenPage() -> Element {
    let mut disabled = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "disable", onclick: move |_| disabled.set(true), "Disable" }
            HoverCard {
                aria_label: "Ada Lovelace",
                id: "card",
                open: true,
                disabled: disabled(),
                content: rsx! { Text { "Wrote the first algorithm meant for a machine." } },
                Button { id: "trigger", "Ada Lovelace" }
            }
        }
    }
}

/// The trigger's press disables the card, `#enable` turns it back on (todo 2446).
#[component]
fn ToggleDisabledPage() -> Element {
    let mut disabled = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            HoverCard {
                aria_label: "Ada Lovelace",
                open_delay: 0,
                close_delay: 0,
                disabled: disabled(),
                content: rsx! { Text { "Wrote the first algorithm meant for a machine." } },
                Button { id: "trigger", onclick: move |_| disabled.set(true), "Ada Lovelace" }
            }
            Button { id: "enable", onclick: move |_| disabled.set(false), "Enable" }
        }
    }
}

/// A card with two controls between two buttons. Its delays are unique on the page, so the
/// test can hold exactly those two timers.
#[component]
fn HoverCardPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            HoverCard {
                aria_label: "Ada Lovelace",
                open_delay: 707,
                close_delay: 808,
                content: rsx! {
                    Flex { direction: "column", gap: "xs",
                        Text { "Wrote the first algorithm meant for a machine." }
                        Button { id: "card-first", variant: "outlined", "Profile" }
                        Button { id: "card-last", variant: "outlined", "Follow" }
                    }
                },
                Button { id: "trigger", variant: "outlined", "Ada Lovelace" }
            }
            Button { id: "after", "After" }
        }
    }
}

/// A read-only `ColorField` in the card: its dropdown never shows, so Escape
/// in it is the card's (todo 446).
#[component]
fn ColorFieldInCardPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            HoverCard {
                aria_label: "Theme",
                id: "card",
                content: rsx! {
                    ColorField {
                        label: "Accent",
                        value: "#ff0000".parse::<ColorCode>().unwrap(),
                        readonly: true,
                    }
                },
                Button { id: "trigger", "Theme" }
            }
            Button { id: "after", "After" }
        }
    }
}

/// `#start-card` and `#below-card` (start-aligned, todo 711), forced open by `#open`'s press
/// so the test can set `dir` first.
#[component]
fn SidesPage() -> Element {
    let card = || rsx! { Text { "Wrote the first algorithm meant for a machine." } };
    let mut open = use_signal(|| false);
    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 240px; padding-top: 80px;",
            Button { id: "open", onclick: move |_| open.set(true), "Open" }
            HoverCard { aria_label: "Start", id: "start-card", open: open(), side: Side::Start, content: card(),
                Button { id: "start-trigger", "Start" }
            }
            HoverCard { aria_label: "Below", id: "below-card", open: open(), content: card(),
                Button { id: "below-trigger", "Below" }
            }
        }
    }
}
