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
    ("/tabs-full-width", || rsx! { TabsFullWidthPage {} }),
    ("/tabs-named", || rsx! { TabsNamedPage {} }),
    ("/tabs-manual", || rsx! { TabsManualPage {} }),
    ("/tabs-late", || rsx! { TabsLatePage {} }),
];

/// Todo 502: manual activation, Billing disabled so the arrows skip it.
#[component]
fn TabsManualPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Tabs {
                aria_label: "Settings",
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
                aria_label: "Settings",
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
                aria_label: "Settings",
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
                aria_label: "Settings",
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

/// Todo 1595: the last tab selected from the start, and by a button outside the strip.
#[component]
fn TabsLatePage() -> Element {
    let mut late = use_signal(|| Crowded::Overview);

    rsx! {
        Flex { direction: "column", gap: "md",
            Tabs {
                id: "initial",
                aria_label: "Initial",
                value: Crowded::Integrations,
                onchange: |_| {},
                panel: |t: Crowded| rsx! { Text { "panel for {t.label()}" } },
            }
            button { id: "late-button", r#type: "button",
                onclick: move |_| late.set(Crowded::Integrations),
                "Integrations"
            }
            Tabs {
                id: "late",
                aria_label: "Late",
                value: late(),
                onchange: move |next| late.set(next),
                panel: |t: Crowded| rsx! { Text { "panel for {t.label()}" } },
            }
        }
    }
}

/// Todos 2422 and 2423: a crowded `full_width` strip, and a default Tabs nested
/// in a `full_width` one's panel.
#[component]
fn TabsFullWidthPage() -> Element {
    let mut tab = use_signal(|| Crowded::Overview);
    let mut section = use_signal(|| Section::Account);
    let mut inner = use_signal(|| Section::Account);

    rsx! {
        Flex { direction: "column", gap: "md",
            Tabs {
                id: "crowded",
                aria_label: "Crowded",
                full_width: true,
                value: tab(),
                onchange: move |next| tab.set(next),
                panel: |t: Crowded| rsx! { Text { "panel for {t.label()}" } },
            }
            Tabs {
                id: "outer",
                aria_label: "Outer",
                full_width: true,
                value: section(),
                onchange: move |next| section.set(next),
                panel: move |_: Section| rsx! {
                    Tabs {
                        id: "inner",
                        aria_label: "Inner",
                        value: inner(),
                        onchange: move |next| inner.set(next),
                        panel: |s: Section| rsx! { Text { "panel for {s.label()}" } },
                    }
                },
            }
        }
    }
}

/// More tabs than a phone's width holds.
#[component]
fn TabsCrowdedPage() -> Element {
    let mut tab = use_signal(|| Crowded::Overview);

    rsx! {
        Tabs {
            aria_label: "Settings",
            value: tab(),
            onchange: move |next| tab.set(next),
            panel: |t: Crowded| rsx! {
                Text { "panel for {t.label()}" }
            },
        }
    }
}
