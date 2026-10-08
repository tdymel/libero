//! One themed page: the theme sets what the components below leave unset.

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Button, Flex, HoverCard, OptionList, Options, Tabs, Toolbar},
    theme::{
        HoverCardDefaults, Side, TabsActivation, TabsDefaults, Theme, ToolbarDefaults,
    },
};

use crate::Routes;

pub const ROUTES: Routes = &[("/theme-defaults", || rsx! { ThemedPage {} })];

static THEMED: Theme = Theme {
    hover_card: HoverCardDefaults {
        side: Side::Top,
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

/// The card has room only above its trigger when it opens on the theme's side.
#[component]
fn ThemedPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        LiberoProvider { themes: &THEMED,
            Flex { direction: "column", gap: "md", max_width: "420px",
                div { style: "height: 240px" }
                HoverCard {
                    aria_label: "Ada Lovelace",
                    content: rsx! { "Wrote the first algorithm." },
                    Button { id: "trigger", variant: "outlined", "Ada Lovelace" }
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
            }
        }
    }
}
