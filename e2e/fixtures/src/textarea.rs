//! `Textarea { counter }` with `maxlength="20"`: one controlled, one owning its
//! own text (todo 584), and one in a `Form` that resets (todo 679).

use dioxus::prelude::*;
use libero::components::{Button, Flex, Form, Textarea, use_form};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/textarea/counter", || rsx! { CounterPage {} }),
    ("/textarea/reset", || rsx! { ResetPage {} }),
];

#[component]
fn CounterPage() -> Element {
    let mut note = use_signal(String::new);

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
