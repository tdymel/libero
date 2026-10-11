//! `Blockquote`, left to right and right to left.

use dioxus::prelude::*;
use libero::components::Blockquote;

use crate::Routes;

pub const ROUTES: Routes = &[("/blockquote", || rsx! { BlockquotePage {} })];

#[component]
fn BlockquotePage() -> Element {
    rsx! {
        div { style: "width: 320px; display: flex; flex-direction: column; gap: 16px",
            div { id: "ltr",
                Blockquote { radius: "md", "A quote that starts on the left." }
            }
            div { id: "rtl", dir: "rtl",
                Blockquote { radius: "md", "A quote that starts on the right." }
            }
        }
    }
}
