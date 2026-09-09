//! `ScrollArea` around a `Virtualize` list.

use dioxus::prelude::*;
use libero::components::{Flex, ScrollArea, Text, Virtualize};

use crate::Routes;

pub const ROUTES: Routes = &[("/scroll-area", || rsx! { ScrollAreaPage {} })];

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
