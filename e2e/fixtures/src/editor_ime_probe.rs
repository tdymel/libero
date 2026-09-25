//! Probe 0b of the editor epic (1159): what a `contenteditable` hands Rust on each input,
//! in the order Rust sees it. Not a component; the Android note reads its numbers.

use dioxus::html::{BeforeInputEvent, InputType};
use dioxus::prelude::*;

use crate::Routes;

pub const ROUTES: Routes = &[
    (
        "/editor-ime-probe/observe",
        || rsx! { Probe { mode: Mode::Observe } },
    ),
    (
        "/editor-ime-probe/cancel",
        || rsx! { Probe { mode: Mode::Cancel } },
    ),
    (
        "/editor-ime-probe/model",
        || rsx! { Probe { mode: Mode::Model } },
    ),
];

/// Observe: the browser edits, Rust logs. Cancel: Rust vetoes `x` and Enter.
/// Model: Rust cancels every cancellable edit and renders its own string.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Observe,
    Cancel,
    Model,
}

#[component]
fn Probe(mode: Mode) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut model = use_signal(String::new);
    let mut selection = use_signal(String::new);
    let pulls = use_signal(Vec::<u128>::new);

    // Selection push: each `selectionchange` crosses the eval channel stamped with its send
    // time; an older mount's listener removes itself (the document outlives a route).
    use_effect(move || {
        let mut eval = document::eval(
            r#"
            const gen = (window.__probeSelGen = (window.__probeSelGen ?? 0) + 1);
            let seq = 0;
            const on = () => {
                if (window.__probeSelGen !== gen) return document.removeEventListener('selectionchange', on);
                const s = getSelection();
                seq += 1;
                dioxus.send([seq, s.anchorOffset, s.focusOffset, performance.now()]);
            };
            document.addEventListener('selectionchange', on);
            await new Promise(() => {});
            "#,
        );
        spawn(async move {
            while let Ok((seq, anchor, focus, sent)) = eval.recv::<(u32, u32, u32, f64)>().await {
                selection.set(format!("{seq}:{anchor}:{focus}:{sent}"));
            }
        });
    });

    let before = move |e: BeforeInputEvent| {
        let kind = e.input_type();
        let data = e.data().data().unwrap_or_default();
        log.write()
            .push(format!("beforeinput|{kind}|{data}|{}", e.is_composing()));
        match mode {
            Mode::Observe => {}
            Mode::Cancel => {
                if data == "x"
                    || matches!(
                        kind,
                        InputType::InsertParagraph | InputType::InsertLineBreak
                    )
                {
                    e.prevent_default();
                }
            }
            Mode::Model => {
                if e.is_composing() || kind == InputType::InsertCompositionText {
                    return;
                }
                match kind {
                    InputType::InsertText | InputType::InsertReplacementText => {
                        model.write().push_str(&data)
                    }
                    InputType::InsertParagraph | InputType::InsertLineBreak => {
                        model.write().push('\n')
                    }
                    InputType::DeleteContentBackward => {
                        model.write().pop();
                    }
                    _ => return,
                }
                e.prevent_default();
            }
        }
    };

    // Selection pull: one eval per key release, timed from Rust (no `Instant` on wasm32).
    let pull = move |_| {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let read =
                document::eval("const s = getSelection(); return [s.anchorOffset, s.focusOffset];");
            let started = std::time::Instant::now();
            let mut pulls = pulls;
            spawn(async move {
                if read.join::<(u32, u32)>().await.is_ok() {
                    pulls.write().push(started.elapsed().as_micros());
                }
            });
        }
    };

    rsx! {
        div {
            id: "editor",
            contenteditable: "true",
            role: "textbox",
            "aria-multiline": "true",
            "aria-label": "Probe",
            style: "white-space: pre-wrap; min-height: 4em; max-width: 320px; border: 1px solid; padding: 8px;",
            onbeforeinput: before,
            oninput: move |e: FormEvent| {
                log.write().push(format!("input|{}", e.value()));
                if mode == Mode::Model {
                    model.set(e.value());
                }
            },
            oncompositionstart: move |e: CompositionEvent| log.write().push(format!("compositionstart|{}", e.data().data())),
            oncompositionupdate: move |e: CompositionEvent| log.write().push(format!("compositionupdate|{}", e.data().data())),
            oncompositionend: move |e: CompositionEvent| log.write().push(format!("compositionend|{}", e.data().data())),
            onkeydown: move |e: KeyboardEvent| log.write().push(format!("keydown|{}", e.key())),
            onkeyup: pull,
            if mode == Mode::Model { "{model}" }
        }
        pre { id: "log", "{log.read().join(\"\\n\")}" }
        output { id: "selection", "{selection}" }
        output { id: "pulls", "{pulls.read().iter().map(u128::to_string).collect::<Vec<_>>().join(\",\")}" }
    }
}
