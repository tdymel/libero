//! Five `NavigationBarItem`s in a 320px column, a three-item RTL bar, and a
//! fixed bar over a tall page with only the selected label shown, and a rail in
//! either direction, and a fixed responsive one that is a bar or a rail by CSS alone.

use dioxus::prelude::*;
use libero::components::{NavigationBar, NavigationBarItem, Indicator, Pictogram, SvgData};
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/navigation-bar", || rsx! { NavigationBarPage {} }),
    ("/navigation-bar/fixed", || rsx! { FixedPage {} }),
    ("/navigation-bar/scroller", || rsx! { ScrollerPage {} }),
    ("/navigation-bar/unselected", || rsx! { UnselectedPage {} }),
    ("/navigation-bar/rail", || rsx! { RailPage { dir: "ltr" } }),
    ("/navigation-bar/rail-rtl", || rsx! { RailPage { dir: "rtl" } }),
    ("/navigation-bar/responsive", || rsx! { ResponsivePage {} }),
];

/// A fixed `responsive` bar: a bar on a phone, a rail from `sm` up (todo 4127).
#[component]
fn ResponsivePage() -> Element {
    rsx! {
        div { id: "content", style: "padding-inline-start: var(--lsx-navigation-rail-width, 0px); padding-bottom: var(--lsx-navigation-bar-height, 0px);",
            div { style: "height: 2000px", "Page" }
        }
        NavigationBar { id: "responsive", "aria-label": "Main", variant: "responsive", position: "fixed",
            for (index, (id, label, icon)) in ITEMS.into_iter().take(3).enumerate() {
                NavigationBarItem {
                    key: "{id}",
                    id: "responsive-{id}",
                    selected: index == 0,
                    onclick: |_| {},
                    icon: rsx! { Pictogram { icon } },
                    "{label}"
                }
            }
        }
    }
}

/// A rail beside a tall page, as a tablet layout (todo 3890).
#[component]
fn RailPage(dir: &'static str) -> Element {
    let mut current = use_signal(|| 0usize);
    rsx! {
        div { id: "layout", dir, style: "display: flex;",
            NavigationBar { id: "rail", "aria-label": "Main", variant: "rail",
                for (index, (id, label, icon)) in ITEMS.into_iter().enumerate() {
                    NavigationBarItem {
                        key: "{id}",
                        id: "rail-{id}",
                        selected: current() == index,
                        onclick: move |_| current.set(index),
                        icon: rsx! { Pictogram { icon } },
                        "{label}"
                    }
                }
            }
            div { id: "content", style: "flex: 1; min-width: 0; padding: 16px;",
                a { id: "after", href: "#after", "After the rail" }
                div { style: "height: 2000px", "Page" }
            }
        }
    }
}

/// A link to the page it is on, with `selected: false`.
#[component]
fn UnselectedPage() -> Element {
    rsx! {
        NavigationBar { "aria-label": "Links",
            NavigationBarItem { id: "not-current", to: "/navigation-bar/unselected", selected: false,
                icon: rsx! { Pictogram { icon: lucide::house::outlined } },
                "Same"
            }
        }
    }
}

/// `/navigation-bar/fixed`'s `#link-0..`, past any viewport's height.
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
        NavigationBar { id: "{prefix}bar", "aria-label": "Main", show_labels,
            for (index, (id, label, icon)) in ITEMS.into_iter().take(count).enumerate() {
                NavigationBarItem {
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
fn NavigationBarPage() -> Element {
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
                NavigationBar { "aria-label": "Links",
                    NavigationBarItem { id: "link", to: "/navigation-bar", selected: true,
                        icon: rsx! { Pictogram { icon: lucide::house::outlined } },
                        "Here"
                    }
                    NavigationBarItem { id: "disabled", to: "/navigation-bar/fixed", disabled: true,
                        icon: rsx! { Pictogram { icon: lucide::user::outlined } },
                        "Away"
                    }
                    NavigationBarItem { id: "disabled-button", disabled: true,
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
        div { id: "pane", style: "max-width: 320px; height: 300px; overflow-y: auto; scroll-padding-bottom: var(--lsx-navigation-bar-height); --lsx-scroll-padding-bottom: var(--lsx-navigation-bar-height);",
            for index in 0..20 {
                a { key: "{index}", id: "row-{index}", href: "#row-{index}", style: "display: block; padding: 12px;", "Row {index}" }
            }
            NavigationBar { id: "sticky", "aria-label": "Main", position: "sticky",
                for (index, (id, label, icon)) in ITEMS.into_iter().take(3).enumerate() {
                    NavigationBarItem {
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
        div { id: "page", style: "padding-bottom: var(--lsx-navigation-bar-height);",
            for index in 0..LINKS {
                a { key: "{index}", id: "link-{index}", href: "#link-{index}", style: "display: block; padding: 12px;", "Link {index}" }
            }
            div { style: "height: 2000px", "Page" }
        }
        NavigationBar { id: "fixed", "aria-label": "Main", position: "fixed", show_labels: "selected",
            for (index, (id, label, icon)) in ITEMS.into_iter().take(3).enumerate() {
                NavigationBarItem {
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
