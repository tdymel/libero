//! `Textarea { counter }` with `maxlength="20"`: one controlled, one owning its
//! own text (todo 584), and one in a `Form` that resets (todo 679). `/textarea/scroller`
//! puts one in a scrolling ancestor (1201).

use dioxus::prelude::*;
use libero::components::{Button, Flex, Form, Text, TextField, Textarea, use_form};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/textarea/counter", || rsx! { CounterPage {} }),
    ("/textarea/reset", || rsx! { ResetPage {} }),
    ("/textarea/raw-reset", || rsx! { RawResetPage {} }),
    ("/textarea/echo", || rsx! { EchoPage {} }),
    ("/textarea/scroller", || rsx! { ScrollerPage {} }),
];

/// Todo 1201: a three-line textarea between spacers in a short scroller, which
/// an arrow on the first or last line must not scroll.
#[component]
fn ScrollerPage() -> Element {
    rsx! {
        div { id: "scroller", style: "height: 160px; max-width: 320px; overflow-y: auto",
            div { style: "height: 240px" }
            Textarea { label: "Long note", initial_value: "one\ntwo\nthree" }
            div { style: "height: 240px" }
        }
    }
}

/// The typed value echoed in `#echo`, for the shared web/native scenarios.
#[component]
fn EchoPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Textarea { label: "Note", value: value(), oninput: move |next| value.set(next) }
            Text { id: "echo", "{value}" }
        }
    }
}

/// Todo 685: a raw `<form>`, no libero `Form`, reset by its own button.
#[component]
fn RawResetPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            form {
                Textarea {
                    label: "Remark",
                    counter: true,
                    maxlength: "20",
                    initial_value: "hi",
                }
                button { r#type: "reset", "Clear" }
            }
        }
    }
}

#[component]
fn CounterPage() -> Element {
    let mut note = use_signal(String::new);
    let mut code = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Textarea {
                label: "Note",
                counter: true,
                maxlength: "20",
                value: note(),
                oninput: move |next| note.set(next),
            }
            Textarea { label: "Remark", counter: true, maxlength: "20" }
            // Todo 942: Blitz's editor ignores `maxlength` on an input too.
            TextField { label: "Code", maxlength: "4", value: code(), oninput: move |next| code.set(next) }
            Text { id: "code", "{code}" }
        }
    }
}

#[component]
fn ResetPage() -> Element {
    let value = use_store(String::new);
    let form = use_form();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Form { value, form,
                Textarea {
                    label: "Remark",
                    counter: true,
                    maxlength: "20",
                    initial_value: "hi",
                }
                button { r#type: "reset", "Clear" }
            }
            Button { onclick: move |_| form.reset(), "Reset" }
        }
    }
}
