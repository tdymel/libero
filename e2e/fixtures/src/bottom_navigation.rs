//! Five `BottomNavigationItem`s in a 320px column, a three-item RTL bar, and a
//! fixed bar over a tall page with only the selected label shown.

use dioxus::prelude::*;
use libero::components::{BottomNavigation, BottomNavigationItem, Indicator, Pictogram, SvgData};
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/bottom-navigation", || rsx! { BottomNavigationPage {} }),
    ("/bottom-navigation/fixed", || rsx! { FixedPage {} }),
    ("/bottom-navigation/scroller", || rsx! { ScrollerPage {} }),
];

/// `/bottom-navigation/fixed`'s `#link-0..`, past any viewport's height.
const LINKS: usize = 30;

const ITEMS: [(&str, &str, SvgData); 5] = [
    ("home", "Home", lucide::house::outlined),
    ("search", "Search", lucide::search::outlined),
    ("inbox", "Inbox", lucide::inbox::outlined),
    ("long", "Notifications and messages", lucide::bell::outlined),
    ("profile", "Profile", lucide::user::outlined),
];

#[component]
fn Bar(prefix: &'static str, count: usize, show_labels: &'static str) -> Element {
    let mut current = use_signal(|| 0usize);
    rsx! {
        BottomNavigation { id: "{prefix}bar", "aria-label": "Main", show_labels,
            for (index, (id, label, icon)) in ITEMS.into_iter().take(count).enumerate() {
                BottomNavigationItem {
                    key: "{id}",
                    id: "{prefix}{id}",
                    selected: current() == index,
                    onclick: move |_| current.set(index),
                    "aria-label": (id == "inbox").then_some("Inbox, 3 unread"),
                    icon: rsx! { Pictogram { icon } },
                    badge: (id == "inbox").then(|| rsx! { Indicator { label: 3u32 } }),
                    "{label}"
                }
            }
        }
    }
}

#[component]
fn BottomNavigationPage() -> Element {
    let mut clicked = use_signal(|| false);
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 24px;",
            // `max-width`: a phone column that still reflows at 320px (WCAG 1.4.10).
            div { id: "column", style: "max-width: 320px;",
                Bar { prefix: "", count: 5, show_labels: "always" }
            }
            div { id: "rtl", dir: "rtl", style: "max-width: 320px;",
                Bar { prefix: "rtl-", count: 3, show_labels: "always" }
            }
            div { style: "max-width: 320px;",
                BottomNavigation { "aria-label": "Links",
                    BottomNavigationItem { id: "link", to: "/bottom-navigation", selected: true,
                        icon: rsx! { Pictogram { icon: lucide::house::outlined } },
                        "Here"
                    }
                    BottomNavigationItem { id: "disabled", to: "/bottom-navigation/fixed", disabled: true,
                        icon: rsx! { Pictogram { icon: lucide::user::outlined } },
                        "Away"
                    }
                    BottomNavigationItem { id: "disabled-button", disabled: true,
                        onclick: move |_| clicked.set(true),
                        icon: rsx! { Pictogram { icon: lucide::bell::outlined } },
                        "Off"
                    }
                }
                if clicked() {
                    p { id: "clicked", "Clicked" }
                }
            }
        }
    }
}

/// A sticky bar closing a scrolling pane of links, as in a phone frame.
#[component]
fn ScrollerPage() -> Element {
    rsx! {
        div { id: "pane", style: "max-width: 320px; height: 300px; overflow-y: auto; scroll-padding-bottom: var(--lsx-bottom-navigation-height); --lsx-scroll-padding-bottom: var(--lsx-bottom-navigation-height);",
            for index in 0..20 {
                a { key: "{index}", id: "row-{index}", href: "#row-{index}", style: "display: block; padding: 12px;", "Row {index}" }
            }
            BottomNavigation { id: "sticky", "aria-label": "Main", position: "sticky",
                for (index, (id, label, icon)) in ITEMS.into_iter().take(3).enumerate() {
                    BottomNavigationItem {
                        key: "{id}",
                        id: "sticky-{id}",
                        selected: index == 0,
                        onclick: |_| {},
                        icon: rsx! { Pictogram { icon } },
                        "{label}"
                    }
                }
            }
        }
    }
}

#[component]
fn FixedPage() -> Element {
    rsx! {
        // Links down past the fold: Tab reaches one in view but under the bar (todo 1827).
        div { id: "page", style: "padding-bottom: var(--lsx-bottom-navigation-height);",
            for index in 0..LINKS {
                a { key: "{index}", id: "link-{index}", href: "#link-{index}", style: "display: block; padding: 12px;", "Link {index}" }
            }
            div { style: "height: 2000px", "Page" }
        }
        BottomNavigation { id: "fixed", "aria-label": "Main", position: "fixed", show_labels: "selected",
            for (index, (id, label, icon)) in ITEMS.into_iter().take(3).enumerate() {
                BottomNavigationItem {
                    key: "{id}",
                    id: "fixed-{id}",
                    selected: index == 0,
                    onclick: |_| {},
                    icon: rsx! { Pictogram { icon } },
                    "{label}"
                }
            }
        }
    }
}
