//! `Tabs`, the roving-tabindex archetype's pilot.

use dioxus::prelude::*;
use libero::components::{Flex, OptionList, Options, Tabs, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tabs", || rsx! { TabsPage {} }),
    (
        "/tabs-disabled-selected",
        || rsx! { TabsDisabledSelectedPage {} },
    ),
];

/// The enum is the tab strip, so it is the fixture's whole configuration.
#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    Billing,
    Admin,
}

/// The roving-tabindex archetype: one tab stop, arrows inside.
#[component]
fn TabsPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Tabs {
                value: section(),
                onchange: move |next| section.set(next),
                panel: |s: Section| rsx! {
                    Text { "panel for {s.label()}" }
                },
            }
        }
    }
}

/// Todo 403: the selected tab is disabled, which a controlled `value` allows.
/// The arrows must still part ways from it.
#[component]
fn TabsDisabledSelectedPage() -> Element {
    let mut section = use_signal(|| Section::Billing);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Tabs {
                value: section(),
                onchange: move |next| section.set(next),
                options: OptionList::from_options().disabling(|s| *s == Section::Billing),
                panel: |s: Section| rsx! {
                    Text { "panel for {s.label()}" }
                },
            }
        }
    }
}
