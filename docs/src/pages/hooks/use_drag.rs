use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Text},
    hooks::{DragMove, DragOptions, DragStart, drag_handle_sx, use_drag, use_element},
    sx::sx,
};

/// The hook call and the knob it drives, as `Knob` renders them.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let track = use_element();
let mut x = use_signal(|| 0.0);
let mut from = use_signal(|| 0.0);
let drag = use_drag(DragOptions {
    capture: track,
    onstart: Callback::new(move |_: DragStart| from.set(x())),
    onmove: Callback::new(move |step: DragMove| {
        x.set((from() + step.delta().x).clamp(0.0, 200.0));
    }),
    onend: Callback::new(|()| {}),
});

rsx! {
    Box {
        onmounted: track.mount(),
        onpointermove: move |event| drag.onpointermove.call(event),
        onpointerup: move |event| drag.onpointerup.call(event),
        onpointercancel: move |event| drag.onpointercancel.call(event),
        sx: sx().position("relative").width("232px").height("32px")
            .background("muted.1").border_radius("16px"),
        Box {
            onpointerdown: move |event| drag.onpointerdown.call(event),
            // The keyboard path a drag needs: focusable, named, and on the arrows.
            tabindex: 0,
            role: "slider",
            aria_label: "Knob position",
            aria_valuemin: 0,
            aria_valuemax: 200,
            aria_valuenow: "{x}",
            onkeydown: move |event: KeyboardEvent| {
                let next = match event.key() {
                    Key::ArrowLeft | Key::ArrowDown => x() - 10.0,
                    Key::ArrowRight | Key::ArrowUp => x() + 10.0,
                    Key::Home => 0.0,
                    Key::End => 200.0,
                    _ => return,
                };
                event.prevent_default();
                x.set(next.clamp(0.0, 200.0));
            },
            sx: drag_handle_sx().position("absolute").width("32px").height("32px")
                .border_radius("16px").background("primary.6").cursor("grab")
                // Inset the focus ring: outside a filled knob it turns white on white.
                .focus_visible(sx().outline_offset("-4px")),
            style: "left: {x}px",
        }
    }
}"#
    .to_string()
}

#[component]
fn Knob() -> Element {
    let track = use_element();
    let mut x = use_signal(|| 0.0);
    let mut from = use_signal(|| 0.0);
    let drag = use_drag(DragOptions {
        capture: track,
        onstart: Callback::new(move |_: DragStart| from.set(x())),
        onmove: Callback::new(move |step: DragMove| {
            x.set((from() + step.delta().x).clamp(0.0, 200.0));
        }),
        onend: Callback::new(|()| {}),
    });

    rsx! {
        Box {
            onmounted: track.mount(),
            onpointermove: move |event| drag.onpointermove.call(event),
            onpointerup: move |event| drag.onpointerup.call(event),
            onpointercancel: move |event| drag.onpointercancel.call(event),
            sx: sx().position("relative").width("232px").height("32px")
                .background("muted.1").border_radius("16px"),
            Box {
                onpointerdown: move |event| drag.onpointerdown.call(event),
                tabindex: 0,
                role: "slider",
                aria_label: "Knob position",
                aria_valuemin: 0,
                aria_valuemax: 200,
                aria_valuenow: "{x}",
                onkeydown: move |event: KeyboardEvent| {
                    let next = match event.key() {
                        Key::ArrowLeft | Key::ArrowDown => x() - 10.0,
                        Key::ArrowRight | Key::ArrowUp => x() + 10.0,
                        Key::Home => 0.0,
                        Key::End => 200.0,
                        _ => return,
                    };
                    event.prevent_default();
                    x.set(next.clamp(0.0, 200.0));
                },
                sx: drag_handle_sx().position("absolute").width("32px").height("32px")
                    .border_radius("16px").background("primary.6").cursor("grab")
                    .focus_visible(sx().outline_offset("-4px")),
                style: "left: {x}px",
            }
        }
    }
}

#[component]
pub fn UseDragPage() -> Element {
    rsx! {
        DocPage {
            title: "use_drag",
            source: "libero/src/hooks/drag.rs",
            markdown: "/md/use_drag.md",
            lead: rsx! {
                Text {
                    Code { source: "use_drag(options: DragOptions) -> Drag" }
                    " is the pointer plumbing for a drag. It captures the pointer, keeps the "
                    "start point and reports each move as a delta against it. Release and "
                    "cancel end the same way. It knows no axes and no units, so convert the "
                    "delta yourself."
                }
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
            },

            Demo {
                component: "use_drag",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Knob {} },
                wrap: Wrap(code),
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "A pointer is not a keyboard, so anything a drag sets needs a second way "
                    "in. The knob is a focusable, named slider that takes the arrow keys, "
                    "Home and End. A right or middle button never starts a drag, and a "
                    "second finger is ignored."
                }
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
        }
    }
}
