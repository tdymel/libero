//! `ScrollArea` around a `Virtualize` list.

use dioxus::prelude::*;
use libero::components::{
    Flex, ScrollArea, ScrollPositionEvent, Text, Virtualize, use_scroll_area,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/scroll-area", || rsx! { ScrollAreaPage {} }),
    ("/scroll-area/edges", || rsx! { EdgesPage {} }),
    ("/scroll-area/keyboard", || rsx! { KeyboardPage {} }),
    ("/scroll-area/rtl", || rsx! { RtlPage {} }),
];

/// A horizontal area under `dir="rtl"`, which starts at its right edge: the
/// last `onscroll` percent, both `on*reached` counts, and `#to-end`, which
/// asks the handle for 100%.
#[component]
fn RtlPage() -> Element {
    let area = use_scroll_area();
    let mut x = use_signal(|| 0.0);
    let mut left = use_signal(|| 0);
    let mut right = use_signal(|| 0);
    rsx! {
        div { dir: "rtl",
            Flex { direction: "column", gap: "md",
                Text { id: "x", "{x}" }
                Text { id: "left-reached", "{left}" }
                Text { id: "right-reached", "{right}" }
                button { id: "to-end", onclick: move |_| area.scroll_to_percent(Some(100.0), None), "End" }
                div { style: "width: 200px; height: 80px",
                    ScrollArea {
                        id: "wide",
                        handle: area,
                        scrollbars: "horizontal",
                        "aria-label": "Wide",
                        onscroll: move |event: ScrollPositionEvent| {
                            let (ScrollPositionEvent::Start(at, _)
                            | ScrollPositionEvent::Change(at, _)
                            | ScrollPositionEvent::End(at, _)) = event;
                            x.set(at.round());
                        },
                        onleftreached: move |()| left += 1,
                        onrightreached: move |()| right += 1,
                        div { style: "width: 1000px", "Wide content" }
                    }
                }
            }
        }
    }
}

/// Two areas of plain text, nothing focusable inside: one left at its
/// defaults, one `focusable` and named as APG's scrollable region asks. After
/// them, the automatic tab stop's other cases (585): content that fits, links
/// inside, and content that grows past the area.
#[component]
fn KeyboardPage() -> Element {
    let lines = || (0..40).map(|i| rsx! { p { key: "{i}", "Line {i}" } });
    let mut grown = use_signal(|| 2);
    let mut nested = use_context_provider(|| Signal::new(0u8));
    rsx! {
        Flex { direction: "column", gap: "md",
            button { id: "before", "Before" }
            div { style: "height: 120px",
                ScrollArea { id: "plain", "aria-label": "Changelog", {lines()} }
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
            div { id: "short-pane", style: "height: 120px",
                ScrollArea { id: "short", "aria-label": "Note", p { "One line" } }
            }
            // Fits, but the caller's role takes the name all the same.
            div { style: "height: 120px",
                ScrollArea { id: "listed", role: "list", "aria-label": "Items",
                    div { role: "listitem", "One item" }
                }
            }
            div { style: "height: 120px",
                ScrollArea { id: "links",
                    for i in 0..40 {
                        p { key: "{i}", a { href: "#l{i}", "Link {i}" } }
                    }
                }
            }
            button { id: "grow-more", onclick: move |_| grown.set(40), "More" }
            div { style: "height: 120px",
                ScrollArea { id: "grow", "aria-label": "Growing",
                    for i in 0..grown() {
                        p { key: "{i}", "Line {i}" }
                    }
                }
            }
            button { id: "nested-step", onclick: move |_| nested += 1, "Step" }
            div { style: "height: 120px",
                ScrollArea { id: "nested", "aria-label": "Nested", NestedLines {} }
            }
        }
    }
}

/// Reads the step itself, so a step re-renders it and not the `ScrollArea`
/// (681): two lines, then forty, then forty and a link.
#[component]
fn NestedLines() -> Element {
    let step = use_context::<Signal<u8>>();
    let count = if step() == 0 { 2 } else { 40 };
    rsx! {
        for i in 0..count {
            p { key: "{i}", "Line {i}" }
        }
        if step() > 1 {
            a { href: "#nested-link", "Link" }
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
                    "aria-label": "Edges",
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
                    "aria-label": "Rows",
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
