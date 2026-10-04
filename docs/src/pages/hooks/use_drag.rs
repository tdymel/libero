use crate::components::{Demo, DemoFile, DemoValues, DocPage, DocSection, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Knob;

#[component]
pub fn UseDragPage() -> Element {
    rsx! {
        DocPage {
            title: "Drag",
            source: "libero/src/hooks/drag.rs",
            markdown: "/md/use_drag.md",
            accessibility: a11y()
                .handles(["A right or middle button never starts a drag, and a second finger is ignored."])
                .must([
                    "Give anything a drag sets a second way in: a pointer is not a keyboard. The demo's knob is a focusable, named slider that takes the arrow keys, Home and End.",
                ])
                .example("A volume knob dragged with `use_drag`: the knob is also a focusable slider named \"Volume\", so a keyboard user turns it with the arrows, Home and End."),
            lead: rsx! {
                Text {
                    Code { source: "use_drag(options: DragOptions) -> Drag" }
                    " is the pointer plumbing for a drag. It captures the pointer, keeps the "
                    "start point and reports each move as a delta against it. Release and "
                    "cancel end the same way. It knows no axes and no units, so convert the "
                    "delta yourself."
                }
            },

            Demo {
                component: "Knob",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Knob {} },
                file: DemoFile(include_str!("use_drag/demo.rs")),
            }

            DocSection {
                title: "Web and native",
                Text {
                    "The hook cancels the press's default, so on the web it focuses the "
                    "pressed tab stop itself. Natively, or to focus something else, focus it "
                    "in "
                    Code { source: "onstart" }
                    ". Blitz and a webview have no pointer capture; Blitz follows the "
                    "pointer instead, and in a webview the drag stops once the pointer "
                    "leaves the capture element."
                }
            }

            DocSection {
                title: "The handlers",
                Text {
                    "The returned "
                    Code { source: "Drag" }
                    " holds four handlers and a "
                    Code { source: "dragging" }
                    " signal. "
                    Code { source: "onpointerdown" }
                    " goes on the grab handle, the other three on the "
                    Code { source: "capture" }
                    " element, which owns the geometry. Measure in "
                    Code { source: "onstart" }
                    ", and call its "
                    Code { source: "cancel" }
                    " to refuse the drag. Give the handle "
                    Code { source: "drag_handle_sx()" }
                    ", or a touch scrolls the page and the handle never moves."
                }
            }
        }
    }
}
