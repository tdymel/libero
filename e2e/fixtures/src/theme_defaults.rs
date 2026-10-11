//! One themed page: the theme sets what the components below leave unset.

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Anchor, Button, Flex, HoverCard, OptionList, Options, Tabs, Toolbar,
    },
    theme::{
        Align, AnchorDefaults, HoverCardDefaults, Side, TabsActivation, TabsDefaults, Theme,
        ToolbarDefaults,
    },
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/theme-defaults", || rsx! { ThemedPage {} }),
    ("/theme-defaults/plain", || rsx! { PlainPage {} }),
];

static THEMED: Theme = Theme {
    anchor: AnchorDefaults {
        new_tab_icon: true,
        ..Theme::DEFAULT.anchor
    },
    hover_card: HoverCardDefaults {
        side: Side::Top,
        align: Align::End,
        ..Theme::DEFAULT.hover_card
    },
    tabs: TabsDefaults {
        activation: TabsActivation::Manual,
        ..Theme::DEFAULT.tabs
    },
    toolbar: ToolbarDefaults { loop_focus: false },
    ..Theme::DEFAULT
};

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    Billing,
}

/// The card has room only above its trigger when it opens on the theme's side, and
/// the trigger sits right of the card's start so the themed `End` align has room too.
#[component]
fn ThemedPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        LiberoProvider { themes: &THEMED,
            Flex { direction: "column", gap: "md", max_width: "420px",
                div { style: "height: 240px" }
                div { style: "margin-left: 240px",
                    HoverCard {
                        aria_label: "Ada Lovelace",
                        content: rsx! { "Wrote the first algorithm." },
                        Button { id: "trigger", variant: "outlined", "Ada" }
                    }
                }
                Tabs {
                    aria_label: "Settings",
                    value: section(),
                    onchange: move |next| section.set(next),
                    options: OptionList::from_options(),
                    panel: |_: Section| rsx! { "panel" },
                }
                Toolbar { "aria-label": "Tools",
                    ActionIcon { id: "one", aria_label: "One", "1" }
                    ActionIcon { id: "two", aria_label: "Two", "2" }
                }
                Anchor { id: "themed-link", to: "https://example.com", target: "_blank", "Docs" }
                Anchor {
                    id: "themed-link-off",
                    to: "https://example.com",
                    target: "_blank",
                    new_tab_icon: false,
                    "Docs"
                }
            }
        }
    }
}

/// The library's own theme: a `_blank` link draws no icon but still says it opens a new tab.
#[component]
fn PlainPage() -> Element {
    rsx! {
        LiberoProvider {
            Flex { direction: "column", gap: "md",
                Anchor { id: "anchor", to: "https://example.com", target: "_blank", "Anchor" }
                Button { id: "button", to: "https://example.com", target: "_blank", "Button" }
                ActionIcon {
                    id: "action-icon",
                    aria_label: "Action",
                    to: "https://example.com",
                    target: "_blank",
                    "A"
                }
            }
        }
    }
}
