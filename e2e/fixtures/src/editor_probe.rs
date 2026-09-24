//! Editor epic 1159, probe 0e: which editing events reach Rust from a `contenteditable` on the
//! web, and whether `prevent_default` in `onbeforeinput` cancels the browser's mutation.
//! Probe 0c: `CodeBlock` re-highlight cost per keystroke, Rust against plain.

use dioxus::prelude::*;
use libero::components::CodeBlock;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/editor-probe", || rsx! { EditorProbePage {} }),
    ("/editor-probe/highlight", || rsx! { HighlightProbePage {} }),
];

/// Every event lands as one line in `#log`; `#cancel` makes `onbeforeinput` and `onpaste` cancel.
#[component]
fn EditorProbePage() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut cancel = use_signal(|| false);

    // `selectionchange` fires on `document`, not on the element: pushed through `eval`.
    use_future(move || async move {
        let mut changes = document::eval(
            "document.addEventListener('selectionchange', () => { \
                const s = document.getSelection(); \
                const editor = document.getElementById('editor'); \
                if (s && s.anchorNode && editor && editor.contains(s.anchorNode)) \
                    dioxus.send(`${s.anchorOffset}:${s.focusOffset}`); \
            }); \
            await new Promise(() => {});",
        );
        while let Ok(at) = changes.recv::<String>().await {
            log.push(format!("document selectionchange {at}"));
        }
    });

    rsx! {
        label {
            input {
                id: "cancel",
                r#type: "checkbox",
                checked: cancel(),
                onchange: move |e| cancel.set(e.checked()),
            }
            "Cancel beforeinput and paste"
        }
        div {
            id: "editor",
            contenteditable: "true",
            role: "textbox",
            aria_multiline: "true",
            aria_label: "Probe",
            style: "min-height: 4em; border: 1px solid; padding: 4px; white-space: pre-wrap",
            onbeforeinput: move |e| {
                let d = e.data();
                log.push(format!(
                    "beforeinput {} {:?} composing={}",
                    d.input_type(),
                    d.data(),
                    d.is_composing()
                ));
                if cancel() {
                    e.prevent_default();
                }
            },
            oninput: move |_| log.push("input".into()),
            onkeydown: move |e| log.push(format!("keydown {}", e.key())),
            oncompositionstart: move |e| log.push(format!("compositionstart {:?}", e.data().data())),
            oncompositionupdate: move |e| log.push(format!("compositionupdate {:?}", e.data().data())),
            oncompositionend: move |e| log.push(format!("compositionend {:?}", e.data().data())),
            onpaste: move |e| {
                let text = e.data().data_transfer().get_as_text();
                log.push(format!("paste {text:?}"));
                if cancel() {
                    e.prevent_default();
                }
            },
            onselectionchange: move |_| log.push("element selectionchange".into()),
        }
        pre { id: "log", {log.read().join("\n")} }
    }
}

/// `#source` drives one `CodeBlock`; `#rust` switches its grammar on, off is the control.
#[component]
fn HighlightProbePage() -> Element {
    let mut source = use_signal(String::new);
    let mut rust = use_signal(|| true);

    rsx! {
        label {
            input {
                id: "rust",
                r#type: "checkbox",
                checked: rust(),
                onchange: move |e| rust.set(e.checked()),
            }
            "Rust grammar"
        }
        textarea { id: "source", aria_label: "Source", oninput: move |e| source.set(e.value()) }
        div { id: "block",
            if rust() {
                CodeBlock { language: "rust", copyable: false, source: source() }
            } else {
                CodeBlock { copyable: false, source: source() }
            }
        }
    }
}
