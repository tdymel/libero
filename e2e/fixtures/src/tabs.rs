//! `Tabs`, the roving-tabindex archetype's pilot.

use dioxus::prelude::*;
use libero::components::{Flex, OptionList, Options, Tabs, TabsActivation, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tabs", || rsx! { TabsPage {} }),
    (
        "/tabs-disabled-selected",
        || rsx! { TabsDisabledSelectedPage {} },
    ),
    ("/tabs-disabled", || rsx! { TabsDisabledPage {} }),
    ("/tabs-crowded", || rsx! { TabsCrowdedPage {} }),
    ("/tabs-named", || rsx! { TabsNamedPage {} }),
    ("/tabs-manual", || rsx! { TabsManualPage {} }),
];

/// Todo 502: manual activation, Billing disabled so the arrows skip it.
#[component]
fn TabsManualPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Tabs {
                activation: TabsActivation::Manual,
                value: section(),
                onchange: move |next| section.set(next),
                options: OptionList::from_options().disabling(|s| *s == Section::Billing),
                panel: |s: Section| rsx! { Text { "panel for {s.label()}" } },
            }
        }
    }
}

/// Todo 501: one strip named by `aria_label`, one by a heading.
#[component]
fn TabsNamedPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Tabs {
                id: "labelled",
                aria_label: "Settings",
                value: section(),
                onchange: move |next| section.set(next),
                panel: |s: Section| rsx! { Text { "panel for {s.label()}" } },
            }
            h2 { id: "heading", "Profile" }
            Tabs {
                id: "labelledby",
                aria_labelledby: "heading",
                value: section(),
                onchange: move |next| section.set(next),
                panel: |s: Section| rsx! { Text { "panel for {s.label()}" } },
            }
        }
    }
}

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

/// A disabled tab that is not selected: a click still focuses it.
#[component]
fn TabsDisabledPage() -> Element {
    let mut section = use_signal(|| Section::Account);

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

#[derive(Clone, PartialEq, Options)]
enum Crowded {
    Overview,
    Activity,
    Settings,
    Members,
    Billing,
    Integrations,
}

/// More tabs than a phone's width holds.
#[component]
fn TabsCrowdedPage() -> Element {
    let mut tab = use_signal(|| Crowded::Overview);

    rsx! {
        Tabs {
            value: tab(),
            onchange: move |next| tab.set(next),
            panel: |t: Crowded| rsx! {
                Text { "panel for {t.label()}" }
            },
        }
    }
}
