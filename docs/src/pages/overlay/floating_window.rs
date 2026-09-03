use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, CodeBlock, Dialog, Flex, FloatingWindowOptions, Text, WindowRect},
    hooks::{ModalScope, use_floating_window, use_modal},
    sx::sx,
};

/// Printed verbatim beside the live window - keep the two in step.
const EXAMPLE: &str = r#"let mut last = use_signal(|| None::<WindowRect>);
let inspector = use_floating_window(
    FloatingWindowOptions {
        title: Some("Inspector".into()),
        placement: "bottom-end".into(),
        resizable: true,
        sx: sx().min_width("16rem").min_height("8rem").max_width("40rem").into(),
        onmove: Some(Callback::new(move |rect| last.set(Some(rect)))),
        onresize: Some(Callback::new(move |rect| last.set(Some(rect)))),
        ..Default::default()
    },
    |window| rsx! {
        Text { "Drag the title bar, or focus it and use the arrow keys." }
        Button { onclick: move |_| window.close(), "Done" }
    },
);

rsx! {
    Button { onclick: move |_| inspector.toggle(), "Inspector" }
}"#;

#[component]
pub fn FloatingWindowPage() -> Element {
    let mut last = use_signal(|| None::<WindowRect>);
    let inspector = use_floating_window(
        FloatingWindowOptions {
            title: Some("Inspector".into()),
            placement: "bottom-end".into(),
            resizable: true,
            sx: sx()
                .min_width("16rem")
                .min_height("8rem")
                .max_width("40rem")
                .into(),
            onmove: Some(Callback::new(move |rect| last.set(Some(rect)))),
            onresize: Some(Callback::new(move |rect| last.set(Some(rect)))),
            ..Default::default()
        },
        |window| {
            rsx! {
                Text { "Drag the title bar, or focus it and use the arrow keys." }
                Button { onclick: move |_| window.close(), "Done" }
            }
        },
    );

    // A modal opened from inside a window, to show it covers every window.
    let confirm = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog { title: "A modal",
                Text { "Modals and their overlay sit above every floating window." }
                Button { onclick: move |_| s.close(), "Close" }
            }
        }
    });
    let notes = use_floating_window(
        FloatingWindowOptions {
            title: Some("Notes".into()),
            placement: "top-end".into(),
            ..Default::default()
        },
        move |_| {
            rsx! {
                Text { "Click a window to bring it to the front." }
                Button { onclick: move |_| { confirm.open(); }, "Open a modal" }
            }
        },
    );

    rsx! {
        DocPage {
            title: "Floating window",
            source: "libero/src/hooks/floating_window.rs",
            markdown: "/md/floating-window.md",
            lead: rsx! {
                Text {
                    "A non-modal window over the page: a title bar that drags, an optional corner "
                    "resize handle, and a close button. "
                    Code { source: "use_floating_window" }
                    " owns whether it exists and hands back a handle to "
                    Code { source: "open" }
                    ", "
                    Code { source: "close" }
                    " or "
                    Code { source: "toggle" }
                    " it; the window owns where it is and how big. No overlay, no focus trap."
                }
                Text {
                    Code { source: "placement" }
                    " is where it first appears; once dragged it stays put, clamped into the "
                    "viewport. Size limits are "
                    Code { source: "min_width" }
                    "/"
                    Code { source: "max_width" }
                    " in "
                    Code { source: "sx" }
                    ", and "
                    Code { source: "onmove" }
                    "/"
                    Code { source: "onresize" }
                    " report a "
                    Code { source: "WindowRect" }
                    " to persist. A drag re-renders the whole window, so keep its body shallow."
                }
                Text {
                    "It is a "
                    Code { source: "role=\"dialog\"" }
                    " without "
                    Code { source: "aria-modal" }
                    ", focused on open; Escape or the close button returns focus to the trigger. "
                    "The title bar is a tab stop: Arrow moves 10px, Shift+Arrow 1px. The corner "
                    "handle resizes with Arrow, and Home/End ask for the smallest and largest size "
                    "allowed. Open both windows below: the one you click comes to the front, and a "
                    "modal opened from Notes covers every window."
                }
            },
            DocSection {
                title: "Try it",
                Flex { direction: "row", gap: "sm", align: "center",
                    Button { onclick: move |_| inspector.toggle(), "Inspector" }
                    Button { onclick: move |_| notes.toggle(), "Notes" }
                    Text {
                        match last() {
                            Some(rect) => format!(
                                "Last reported: {:.0}, {:.0} - {:.0} x {:.0}",
                                rect.x, rect.y, rect.width, rect.height
                            ),
                            None => "Move or resize Inspector to see onmove/onresize.".to_string(),
                        }
                    }
                }
                CodeBlock { source: EXAMPLE, language: "rust" }
            }
        }
    }
}
