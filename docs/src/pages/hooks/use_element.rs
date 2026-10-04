use crate::Route;
use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Button, Code, Flex, Text},
    hooks::use_element,
    platform::ElementApi,
    sx::sx,
};

/// The hook call and the markup it measures, as `Measure` renders them.
// snippet: mirrors Measure
fn code(_: &DemoValues, _: &str) -> String {
    r#"let panel = use_element();
let mut size = use_signal(String::new);

rsx! {
    Flex { direction: "column", align: "flex-start", gap: "sm",
        Box {
            onmounted: panel.mount(),
            sx: sx().width("240px").height("96px").padding("md")
                .background("muted.1").border_radius("8px"),
            style: "resize: both; overflow: auto",
            "Drag the corner to resize me."
        }
        Button {
            variant: "outlined",
            onclick: move |_| {
                // Start the read in the handler, await it in a task.
                let read = panel.dimensions();
                spawn(async move {
                    if let Ok(box_size) = read.await {
                        size.set(format!("{:.0} × {:.0} px", box_size.width, box_size.height));
                    }
                });
            },
            "Measure"
        }
        span { role: "status", "{size}" }
    }
}"#
    .to_string()
}

#[component]
fn Measure() -> Element {
    let panel = use_element();
    let mut size = use_signal(String::new);

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Box {
                onmounted: panel.mount(),
                sx: sx().width("240px").height("96px").padding("md")
                    .background("muted.1").border_radius("8px"),
                style: "resize: both; overflow: auto",
                "Drag the corner to resize me."
            }
            Button {
                variant: "outlined",
                onclick: move |_| {
                    let read = panel.dimensions();
                    spawn(async move {
                        if let Ok(box_size) = read.await {
                            size.set(format!("{:.0} × {:.0} px", box_size.width, box_size.height));
                        }
                    });
                },
                "Measure"
            }
            span { role: "status", "{size}" }
        }
    }
}

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
                    ", so it focuses, scrolls and measures the same way on every renderer."
                }
            },

            Demo {
                component: "use_element",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Measure {} },
                wrap: Wrap(code),
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
