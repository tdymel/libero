//! `Splitter`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Splitter, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/splitter", || rsx! { SplitterPage {} }),
    ("/splitter/min-size", || rsx! { MinSizePage {} }),
    ("/splitter/rtl", || rsx! { RtlPage {} }),
];

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
/// drag and a Shift+Tab never sits at the document edge.
#[component]
fn SplitterPage() -> Element {
    rsx! {
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
