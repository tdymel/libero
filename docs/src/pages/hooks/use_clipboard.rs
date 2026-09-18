use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    hooks::use_clipboard,
};

/// The hook call and its button, as `CopyLink` renders them.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut clipboard = use_clipboard();

rsx! {
    Flex { direction: "row", align: "center", gap: "md",
        Button {
            variant: "outlined",
            onclick: move |_| clipboard.copy("https://github.com/tdymel/libero"),
            onblur: move |_| clipboard.reset(),
            "Copy link"
        }
        // Mounted before it has anything to say, so the change is announced.
        span { role: "status",
            if clipboard.copied() { "Copied" } else if clipboard.failed() { "Copy failed" }
        }
    }
}"#
    .to_string()
}

#[component]
fn CopyLink() -> Element {
    let mut clipboard = use_clipboard();

    rsx! {
        Flex { direction: "row", align: "center", gap: "md",
            Button {
                variant: "outlined",
                onclick: move |_| clipboard.copy("https://github.com/tdymel/libero"),
                onblur: move |_| clipboard.reset(),
                "Copy link"
            }
            span { role: "status",
                if clipboard.copied() { "Copied" } else if clipboard.failed() { "Copy failed" }
            }
        }
    }
}

#[component]
pub fn UseClipboardPage() -> Element {
    rsx! {
        DocPage {
            title: "use_clipboard",
            source: "libero/src/hooks/clipboard.rs",
            markdown: "/md/use_clipboard.md",
            lead: rsx! {
                Text {
                    Code { source: "use_clipboard() -> Clipboard" }
                    " writes text to the system clipboard. "
                    Code { source: "copy(text)" }
                    " starts the write, and "
                    Code { source: "copied()" }
                    " turns true once the platform confirms it. A refused write raises "
                    Code { source: "failed()" }
                    " instead and logs a warning. Either flag stays up until you call "
                    Code { source: "reset()" }
                    ". "
                    Code { source: "Clipboard" }
                    " is "
                    Code { source: "Copy" }
                    ", so handlers take it without a clone."
                }
            },

            Demo {
                component: "use_clipboard",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { CopyLink {} },
                wrap: Wrap(code),
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "Say the result in a status region that is already mounted. A button "
                    "whose own label changes is not announced."
                }
            }

            DocSection {
                title: "Web and native",
                Text {
                    "On the web the browser allows the write only in a secure context "
                    "(HTTPS or localhost) and inside a user action, such as the click "
                    "handler above. Natively (the "
                    Code { source: "native" }
                    " feature) it writes the system clipboard. A desktop build without that "
                    "feature, such as a webview, has no clipboard, so "
                    Code { source: "copy" }
                    " raises "
                    Code { source: "failed()" }
                    " at once."
                }
            }
        }
    }
}
