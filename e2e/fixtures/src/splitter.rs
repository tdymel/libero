//! `Splitter`.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Orientation, Splitter, SplitterResizeEvent, Text},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/splitter", || rsx! { SplitterPage {} }),
    ("/splitter/min-size", || rsx! { MinSizePage {} }),
    ("/splitter/rtl", || rsx! { RtlPage {} }),
    ("/splitter/scroll", || rsx! { ScrollPage {} }),
    ("/splitter/content", || rsx! { ContentPage {} }),
];

/// Pane A overflows, and a button sits flush to each edge of both panes (todo 2407):
/// the panes' own scroll (1581), forced colours, and the gutter over pane edges.
#[component]
fn ContentPage() -> Element {
    let edge = |id: &'static str, side: &'static str| {
        rsx! {
            div { style: "position: absolute; top: 0; {side}: 0", Button { id, "{id}" } }
        }
    };
    rsx! {
        div { style: "height: 160px; max-width: 320px",
            Splitter {
                initial_size: 50.0,
                aria_label: "Resize panes",
                panel_a: rsx! {
                    div { id: "pane-a", style: "position: relative; min-height: 400px",
                        {edge("a-start", "inset-inline-start")}
                        {edge("a-end", "inset-inline-end")}
                        Text { "Pane A" }
                    }
                },
                panel_b: rsx! {
                    div { id: "pane-b", style: "position: relative; height: 100%",
                        {edge("b-start", "inset-inline-start")}
                        {edge("b-end", "inset-inline-end")}
                        Text { "Pane B" }
                    }
                },
            }
        }
    }
}

/// Both orientations on a page taller than the screen (todo 1039). `#row-log`
/// lists the row's `start`/`end`, `#row-end` and `#column-end` their last `End`.
#[component]
fn ScrollPage() -> Element {
    let mut row_log = use_signal(String::new);
    let mut row_end = use_signal(String::new);
    let mut column_end = use_signal(String::new);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            sx: sx().padding_top("40vh").padding_bottom("150vh"),
            div { id: "row", style: "height: 120px",
                Splitter {
                    initial_size: 50.0,
                    aria_label: "Resize the row",
                    panel_a: rsx! { Text { "Pane A" } },
                    panel_b: rsx! { Text { "Pane B" } },
                    onresize: move |event: SplitterResizeEvent| match event {
                        SplitterResizeEvent::Start(..) => row_log.write().push_str("start "),
                        SplitterResizeEvent::End(a, _) => {
                            row_log.write().push_str("end ");
                            row_end.set(format!("{a:.0}"));
                        }
                        SplitterResizeEvent::Change(..) => {}
                    },
                }
            }
            Text { id: "row-log", "{row_log}" }
            Text { id: "row-end", "{row_end}" }
            div { id: "column", style: "height: 240px",
                Splitter {
                    initial_size: 50.0,
                    orientation: Orientation::Horizontal,
                    aria_label: "Resize the column",
                    panel_a: rsx! { Text { "Pane A" } },
                    panel_b: rsx! { Text { "Pane B" } },
                    onresize: move |event: SplitterResizeEvent| {
                        if let SplitterResizeEvent::End(a, _) = event {
                            column_end.set(format!("{a:.0}"));
                        }
                    },
                }
            }
            Text { id: "column-end", "{column_end}" }
        }
    }
}

/// Under `dir="rtl"` the row puts pane A on the right.
#[component]
fn RtlPage() -> Element {
    rsx! {
        div { dir: "rtl",
            Flex { direction: "column", gap: "md", max_width: "320px",
                Button { id: "before", "Before" }
                div { style: "height: 160px",
                    Splitter {
                        initial_size: 50.0,
                        aria_label: "Resize panes",
                        panel_a: rsx! { Text { "Pane A" } },
                        panel_b: rsx! { Text { "Pane B" } },
                    }
                }
            }
        }
    }
}

/// A `Splitter` between two buttons, so focus has somewhere to be before a
/// drag and a Shift+Tab never sits at the document edge. `#box[data-changes]`
/// counts `Change` events, for the checks that something did not move it.
#[component]
fn SplitterPage() -> Element {
    let mut changes = use_signal(|| 0);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            div { id: "box", style: "height: 160px", "data-changes": "{changes}",
                Splitter {
                    initial_size: 50.0,
                    aria_label: "Resize panes",
                    panel_a: rsx! { Text { "Pane A" } },
                    panel_b: rsx! { Text { "Pane B" } },
                    onresize: move |event: SplitterResizeEvent| {
                        if let SplitterResizeEvent::Change(..) = event {
                            changes += 1;
                        }
                    },
                }
            }
            Button { id: "after", "After" }
        }
    }
}

/// `#raise` lifts `min_size` from 10 to 40 after the divider has moved.
#[component]
fn MinSizePage() -> Element {
    let mut min_size = use_signal(|| 10.0);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "raise", onclick: move |_| min_size.set(40.0), "Raise" }
            div { style: "height: 160px",
                Splitter {
                    initial_size: 50.0,
                    min_size: min_size(),
                    aria_label: "Resize panes",
                    panel_a: rsx! { Text { "Pane A" } },
                    panel_b: rsx! { Text { "Pane B" } },
                }
            }
        }
    }
}
