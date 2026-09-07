use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Button, Code, CodeBlock, DataList, DataListItem, Flex, Text},
    hooks::{
        DragMove, DragOptions, DragStart, drag_handle_sx, use_clipboard, use_drag, use_element,
    },
    sx::sx,
};

const DRAG: &str = r#"#[component]
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
                .background("grey.1").border_radius("16px"),
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
                    // The fill publishes its contrast colour, white, for what is
                    // drawn *on* it - and the ring is drawn outside, where it
                    // vanishes. Inset it into the fill: 4.86:1 rather than 1.11:1.
                    .focus_visible(sx().outline_offset("-4px")),
                style: "left: {x}px",
            }
        }
    }
}"#;

const CLIPBOARD: &str = r#"#[component]
fn CopyLink() -> Element {
    let mut clipboard = use_clipboard();

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| clipboard.copy("https://github.com/tdymel/libero"),
            onblur: move |_| clipboard.reset(),
            "Copy link"
        }
        // Mounted before it has anything to say, so the change is announced.
        span { role: "status", if clipboard.copied() { "Copied" } }
    }
}"#;

const ROOT_ID: &str = r#"#[component]
fn Disclosure(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    // The caller's `id` if it gave one, a generated one otherwise.
    let id = use_root_id(&attributes);
    let mut open = use_signal(|| false);

    rsx! {
        div { id: "{id}", ..attributes,
            button {
                aria_controls: "{id}-panel",
                aria_expanded: "{open}",
                onclick: move |_| open.toggle(),
                "Details"
            }
            div { id: "{id}-panel", hidden: !open(), {children} }
        }
    }
}"#;

/// Each row is `(hook, what it is for, the page that uses it, that page's title)`.
const OTHERS: [(&str, &str, Route, &str); 5] = [
    (
        "use_element() -> ElementHandle",
        "A handle to one of the component's own elements, mounted with onmounted: handle.mount(). It implements ElementApi.",
        Route::PlatformPage {},
        "Platform",
    ),
    (
        "use_id() -> Signal<String>",
        "A process-unique id, stable for the component's lifetime, for the aria wiring between one instance's parts.",
        Route::PopoverPage {},
        "Popover",
    ),
    (
        "use_portal(content: Option<Element>)",
        "Renders the content at the document root, out of any clipping or stacking ancestor. None takes it away.",
        Route::FloatPage {},
        "Float",
    ),
    (
        "use_presence(open, property) -> Presence",
        "Keeps closing content mounted until its exit transition on property ends: mounted() and visible() drive the markup, on_mounted() and on_transition_end(event) go on the element.",
        Route::CollapsePage {},
        "Collapse",
    ),
    (
        "use_focus_return() -> FocusReturn",
        "Remembers where focus came from, with remember(event) on a trigger's onmounted or remember_active() as it opens, and restore() puts it back - onto fallback(handle) if the trigger is gone. restore() keeps a remember(event) element and consumes a remember_active() snapshot, so arm that one on every open.",
        Route::CollapsePage {},
        "Collapse",
    ),
];

/// `DRAG`, rendered. Kept in step with the snippet by hand.
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
                .background("grey.1").border_radius("16px"),
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
                    // The fill publishes its contrast colour, white, for what is
                    // drawn *on* it - and the ring is drawn outside, where it
                    // vanishes. Inset it into the fill: 4.86:1 rather than 1.11:1.
                    .focus_visible(sx().outline_offset("-4px")),
                style: "left: {x}px",
            }
        }
    }
}

/// `CLIPBOARD`, rendered.
#[component]
fn CopyLink() -> Element {
    let mut clipboard = use_clipboard();

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| clipboard.copy("https://github.com/tdymel/libero"),
            onblur: move |_| clipboard.reset(),
            "Copy link"
        }
        span { role: "status", if clipboard.copied() { "Copied" } }
    }
}

#[component]
pub fn HooksPage() -> Element {
    rsx! {
        DocPage {
            title: "Hooks",
            markdown: "/md/hooks.md",
            lead: rsx! {
                Text {
                    "The hooks below are what libero's own components are built from, and "
                    "they are public for yours. Like every dioxus hook they are positional: "
                    "call them unconditionally, in the same order every render. The "
                    "overlay hooks have pages of their own: "
                    Code { source: "use_popover" }
                    ", "
                    Code { source: "use_modal" }
                    ", "
                    Code { source: "use_drawer" }
                    ", "
                    Code { source: "use_lightbox" }
                    " and "
                    Code { source: "use_floating_window" }
                    "."
                }
            },

            DocSection {
                title: "use_drag",
                Text {
                    "Pointer plumbing for a drag: capture, the start point, a delta against "
                    "it, and one end path for both release and cancel. It knows no axes and "
                    "no units - convert the delta yourself. "
                    Code { source: "onpointerdown" }
                    " goes on the grab handle, the other three on the "
                    Code { source: "capture" }
                    " element, which owns the geometry. Measure in "
                    Code { source: "onstart" }
                    ", and call its "
                    Code { source: "cancel" }
                    " to refuse the drag. Give the handle "
                    Code { source: "drag_handle_sx()" }
                    ", or a touch scrolls the page and never moves."
                }
                Knob {}
                CodeBlock { source: DRAG, language: "rust" }
                Text {
                    "A pointer is not a keyboard, so anything a drag sets needs a second way "
                    "in. The knob is a focusable, named slider that takes the arrow keys, "
                    "Home and End."
                }
            }

            DocSection {
                title: "use_clipboard",
                Text {
                    Code { source: "copy(text)" }
                    " writes to the clipboard, and "
                    Code { source: "copied()" }
                    " turns true once the platform confirms the write - a denied permission "
                    "leaves it false and logs a warning. The flag stays up until you call "
                    Code { source: "reset()" }
                    ". Say the result in a status region that is already mounted; a button "
                    "whose own label changes is not announced."
                }
                Flex {
                    direction: "row",
                    align: "center",
                    gap: "md",
                    CopyLink {}
                }
                CodeBlock { source: CLIPBOARD, language: "rust" }
            }

            DocSection {
                title: "use_root_id",
                Text {
                    "An id for a component that also spreads the caller's "
                    Code { source: "attributes" }
                    ": the caller's own "
                    Code { source: "id" }
                    " when it passed one, a generated one otherwise. Build the ids of the "
                    "inner parts on it, so the aria wiring still holds when the caller "
                    "names the root. "
                    Code { source: "use_id()" }
                    " alone would render a second, different "
                    Code { source: "id" }
                    ", and the browser keeps the caller's."
                }
                CodeBlock { source: ROOT_ID, language: "rust" }
            }

            DocSection {
                title: "Other hooks",
                DataList {
                    for (hook, purpose, route, page) in OTHERS {
                        DataListItem {
                            key: "{hook}",
                            label: rsx! {
                                Code { source: hook }
                            },
                            Text {
                                "{purpose} Used on "
                                Anchor { to: route, "{page}" }
                                "."
                            }
                        }
                    }
                }
            }
        }
    }
}
