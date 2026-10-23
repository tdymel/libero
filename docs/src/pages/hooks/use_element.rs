use crate::Route;
use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
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
                .example("A \"Measure\" button that reads its card's size through the handle: the result goes into a status region, so a screen reader hears \"320 by 180\" without focus moving.")
                .limits([
                    "Until the element mounts, every call answers `PlatformError::Unsupported`.",
                    "A webview answers `query_selector` with `Unsupported` and `is_focused()` with `false`; spread `..handle.attributes()` where it has to find the element.",
                ]),
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
                    ", so it focuses, scrolls and measures the same way on every renderer. Commands such as "
                    Code { source: "focus()" }
                    " return at once, reads such as "
                    Code { source: "dimensions()" }
                    " are futures. The full method list is on "
                    Anchor { to: Route::PlatformPage {}, "Platform" }
                    "."
                }
            },

            Demo {
                component: "Measure",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Measure {} },
                file: DemoFile(include_str!("use_element/demo.rs")),
            }
        }
    }
}
