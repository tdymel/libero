//! `Textarea { counter }` with `maxlength="20"`: one controlled, one owning its
//! own text (todo 584).

use dioxus::prelude::*;
use libero::components::{Flex, Textarea};

use crate::Routes;

pub const ROUTES: Routes = &[("/textarea/counter", || rsx! { CounterPage {} })];

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
