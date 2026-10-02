//! `save_file` (2016): a button saving a small CSV, its outcome printed below.
//! `html[data-covered]` marks frames stopping for half a second: Android's share
//! sheet covers the activity, which draws no frame and applies no edit until it closes.

use dioxus::prelude::*;
use libero::components::{Button, Text};
use libero::platform::{SaveOutcome, save_file};

use crate::Routes;

pub const ROUTES: Routes = &[("/save-file", || rsx! { SaveFilePage {} })];

#[component]
fn SaveFilePage() -> Element {
    let mut outcome = use_signal(|| None::<SaveOutcome>);
    use_effect(|| {
        document::eval(
            "let drawn = performance.now();
            const frame = () => { drawn = performance.now(); requestAnimationFrame(frame); };
            requestAnimationFrame(frame);
            setInterval(() => {
                if (performance.now() - drawn > 500) document.documentElement.dataset.covered = '';
            }, 100);",
        );
    });
    rsx! {
        Button {
            id: "save",
            onclick: move |_| async move {
                let saved = save_file("rows.csv", "text/csv", b"a,b\r\n1,2\r\n".to_vec()).await;
                outcome.set(Some(saved));
            },
            "Save rows.csv"
        }
        Text { id: "outcome",
            if let Some(outcome) = outcome() {
                "{outcome:?}"
            }
        }
    }
}
