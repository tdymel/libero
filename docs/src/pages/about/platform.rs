use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Code, Table, Text, column},
    sx::sx,
};

/// Every public platform API: what it gives, and what that makes possible.
const APIS: [(&str, &str, &str); 7] = [
    (
        "use_element()",
        "Focus, scroll, measure and query one element you mount.",
        "Focus a field when a dialog opens, scroll a row into view, size a popup to its trigger.",
    ),
    (
        "timer()",
        "A callback after a delay or on an interval.",
        "Autoplay, a toast that closes itself, a debounced search.",
    ),
    (
        "keyboard()",
        "Key presses anywhere in the document.",
        "App-wide shortcuts, such as Ctrl+K for a command palette.",
    ),
    (
        "scroll()",
        "Scrolling anywhere, not only the page.",
        "Close a popup when its trigger scrolls away, a header that hides on scroll.",
    ),
    (
        "document()",
        "The focused element, the viewport size, and attributes on the root element.",
        "Put focus back where it was, lay out by window size, set a theme attribute on the root.",
    ),
    (
        "color_scheme()",
        "The system's light or dark, its changes, and a stored choice.",
        "Follow the system theme and remember what the reader picked.",
    ),
    (
        "clock()",
        "Today's date in the user's time zone.",
        "Mark today in a calendar, start a date field on today.",
    ),
];

type Row = (&'static str, &'static str, &'static str);

#[component]
pub fn PlatformPage() -> Element {
    rsx! {
        DocPage {
            title: "Platform",
            markdown: "/md/platform.md",
            lead: rsx! {
                Text {
                    "Everything that reaches past dioxus to the machine goes through "
                    Code { source: "libero::platform" }
                    ". Components use it, and so can you. One call works on the web and "
                    "natively, and each renderer answers with what it has."
                }
            },

            DocSection {
                title: "The APIs",
                Table {
                    aria_label: "Platform APIs",
                    data: APIS.to_vec(),
                    columns: vec![
                        column("API")
                            .value(|row: &Row| row.0)
                            .render(|row: &Row| rsx! { Code { source: row.0, sx: sx().white_space("nowrap") } }),
                        column("What it gives").value(|row: &Row| row.1),
                        column("What you can build").value(|row: &Row| row.2),
                    ],
                }
                Text {
                    "The web and Blitz answer all of them. Android's WebView has no "
                    Code { source: "keyboard()" }
                    ", "
                    Code { source: "scroll()" }
                    ", "
                    Code { source: "document()" }
                    " or "
                    Code { source: "color_scheme()" }
                    " yet, and some element calls there are unsupported."
                }
            }

            DocSection {
                title: "Using them",
                Text {
                    "Each accessor returns an "
                    Code { source: "Option" }
                    ". "
                    Code { source: "None" }
                    " means the running renderer can't do it, so branch once and let the "
                    "feature be absent there instead of unwrapping."
                }
                Text {
                    "A callback API hands back a subscription, and dropping it stops the "
                    "callback. Keep it in a signal for as long as the component lives, and "
                    "write what the callback learns into a signal."
                }
                Text {
                    "Element reads such as "
                    Code { source: "dimensions()" }
                    " are futures. Start one in an event handler and await it in a "
                    Code { source: "spawn" }
                    ". A renderer that can't serve one answers "
                    Code { source: "PlatformError::Unsupported" }
                    "."
                }
            }

            DocSection {
                title: "Clipping in a native window",
                Text {
                    "Natively, a box with a "
                    Code { source: "z-index" }
                    " paints past the clip of an "
                    Code { source: "overflow" }
                    " box around it, unless that box is a stacking context. "
                    "Libero's own scrollers are one. Give yours "
                    Code { source: "position: relative; z-index: 0" }
                    ", or use "
                    Code { source: "ScrollArea" }
                    ". This covers a sticky "
                    Code { source: "Header" }
                    ", and a "
                    Code { source: "Tooltip" }
                    " or "
                    Code { source: "HoverCard" }
                    " trigger, which natively carries a "
                    Code { source: "z-index" }
                    " so the pointer reaches it."
                }
            }
        }
    }
}
