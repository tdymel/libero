//! `ScrollArea` around a `Virtualize` list.

use dioxus::prelude::*;
use libero::components::{Flex, ScrollArea, Text, Virtualize};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/scroll-area", || rsx! { ScrollAreaPage {} }),
    ("/scroll-area/edges", || rsx! { EdgesPage {} }),
    ("/scroll-area/keyboard", || rsx! { KeyboardPage {} }),
];

/// Two areas of plain text, nothing focusable inside: one left at its
/// defaults, one `focusable` and named as APG's scrollable region asks.
#[component]
fn KeyboardPage() -> Element {
    let lines = || (0..40).map(|i| rsx! { p { key: "{i}", "Line {i}" } });
    rsx! {
        Flex { direction: "column", gap: "md",
            button { id: "before", "Before" }
            div { style: "height: 120px",
                ScrollArea { id: "plain", {lines()} }
            }
            div { style: "height: 120px",
                ScrollArea {
                    id: "region",
                    focusable: true,
                    role: "region",
                    "aria-label": "Release notes",
                    {lines()}
                }
            }
            button { id: "after", "After" }
        }
    }
}

/// A plain area with only `ontopreached`/`onbottomreached`, no `onscroll` and
/// no `Virtualize`: the handlers alone must keep its scroll listener.
#[component]
fn EdgesPage() -> Element {
    let mut top = use_signal(|| 0);
    let mut bottom = use_signal(|| 0);
    rsx! {
        Flex { direction: "column", gap: "md",
            Text { id: "top-reached", "{top}" }
            Text { id: "bottom-reached", "{bottom}" }
            div { style: "height: 120px",
                ScrollArea {
                    id: "edges",
                    ontopreached: move |()| top += 1,
                    onbottomreached: move |()| bottom += 1,
                    div { style: "height: 1000px", "Tall content" }
                }
            }
        }
    }
}

/// A pane a test resizes by script, round a virtualized list whose window must
/// follow the pane's height. The caller's own `onresize` on the `ScrollArea`,
/// as `Scroller` sets one, is counted: it must still fire beside the area's.
#[component]
fn ScrollAreaPage() -> Element {
    let mut resizes = use_signal(|| 0);
    rsx! {
        Flex { direction: "column", gap: "md",
            Text { id: "list-resizes", "{resizes}" }
            div { id: "list-pane", style: "height: 120px",
                ScrollArea {
                    onresize: move |_: Event<ResizeData>| resizes += 1,
                    Virtualize {
                        count: 1000,
                        item_size: Some(20.0),
                        item: move |i: usize| rsx! {
                            div { "data-row": i, style: "height: 20px", "Row {i}" }
                        },
                    }
                }
            }
        }
    }
}
