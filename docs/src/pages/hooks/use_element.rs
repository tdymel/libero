use crate::Route;
use crate::components::{Demo, DemoFile, DemoValues, DocPage, DocSection, a11y};
use dioxus::prelude::*;
use libero::components::{Anchor, Code, Text};

mod demo;
use demo::Measure;

#[component]
pub fn UseElementPage() -> Element {
    rsx! {
        DocPage {
            title: "Element handle",
            source: "libero/src/hooks/element.rs",
            markdown: "/md/use_element.md",
            accessibility: a11y()
                .handles([
                    "It adds nothing to the element: no role, name, tab stop or focus style.",
                ])
                .must([
                    "Move focus only in answer to the reader's action, such as a click or a closing panel, never on a timer or a re-render (WCAG 3.2.1).",
                    "Focus only an element that takes focus, a control or one with `tabindex: \"-1\"`, and that shows a visible focus ring.",
                    "Scroll the page only when the reader asked for it.",
                    "Announce a result the reader asked for, such as a measurement, in a status region, as the demo does.",
                ])
                .example("A \"Measure\" button that reads its card's size through the handle: the result goes into a status region, so a screen reader hears \"320 by 180\" without focus moving."),
            lead: rsx! {
                Text {
                    Code { source: "use_element() -> ElementHandle" }
                    " is a handle to one of your component's own elements, and the only way "
                    "to reach an element at all. Mount it with "
                    Code { source: "onmounted: handle.mount()" }
                    ". The handle is "
                    Code { source: "Copy" }
                    " and implements "
                    Code { source: "ElementApi" }
                    ", so it focuses, scrolls and measures the same way on every renderer."
                }
            },

            Demo {
                component: "Measure",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Measure {} },
                file: DemoFile(include_str!("use_element/demo.rs")),
            }

            DocSection {
                title: "Commands and reads",
                Text {
                    "Commands such as "
                    Code { source: "focus()" }
                    " return at once. Reads such as "
                    Code { source: "dimensions()" }
                    " are futures. Start one in the handler and await it in a "
                    Code { source: "spawn" }
                    ". Until the element mounts, every call answers "
                    Code { source: "PlatformError::Unsupported" }
                    ". "
                    Code { source: "is_mounted()" }
                    " is reactive, so an effect that reads it runs again once the element "
                    "is there. The full method list is on "
                    Anchor { to: Route::PlatformPage {}, "Platform" }
                    "."
                }
            }

            DocSection {
                title: "Renderers",
                Text {
                    "The handle picks the richest backing the renderer offers: the DOM "
                    "element on the web, a Blitz node natively (the "
                    Code { source: "native" }
                    " feature), and dioxus's portable mounted data in a webview. A webview "
                    "still measures, scrolls and focuses, but "
                    Code { source: "query_selector" }
                    " answers "
                    Code { source: "Unsupported" }
                    " there, and "
                    Code { source: "is_focused()" }
                    " always answers "
                    Code { source: "false" }
                    ". Where a webview has to find the element, as "
                    Code { source: "use_intersection" }
                    "'s "
                    Code { source: "root" }
                    ", spread "
                    Code { source: "..handle.attributes()" }
                    " on it; on the web and Blitz they are empty."
                }
            }
        }
    }
}
