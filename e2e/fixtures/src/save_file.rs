//! `save_file` (2016): buttons saving a small CSV and a few binary bytes, the
//! outcome printed below.
//! `html[data-covered]` marks frames stopping for half a second: Android's share
//! sheet covers the activity, which draws no frame and applies no edit until it closes.

use dioxus::prelude::*;
use libero::components::{Button, Text};
use libero::platform::{SaveOutcome, save_file};

use crate::Routes;

pub const ROUTES: Routes = &[("/save-file", || rsx! { SaveFilePage {} })];

/// Sets `html[data-covered]` once frames stop for half a second.
pub(crate) fn use_covered_mark() {
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
}

#[component]
fn SaveFilePage() -> Element {
    let mut outcome = use_signal(|| None::<SaveOutcome>);
    use_covered_mark();
    rsx! {
        Button {
            id: "save",
            onclick: move |_| async move {
                let saved = save_file("rows.csv", "text/csv", b"a,b\r\n1,2\r\n".to_vec()).await;
                outcome.set(Some(saved));
            },
            "Save rows.csv"
        }
        // Not UTF-8, so Android shares it as a file (2078).
        Button {
            id: "save-binary",
            onclick: move |_| async move {
                let saved = save_file("bytes.bin", "application/octet-stream", vec![0xff, 0, 0xfe]).await;
                outcome.set(Some(saved));
            },
            "Save bytes.bin"
        }
        Text { id: "outcome",
            if let Some(outcome) = outcome() {
                "{outcome:?}"
            }
        }
    }
}
