//! `Tldr`'s trigger names: a blank label falls back to the localized words.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Tldr};

#[test]
fn a_blank_label_falls_back_to_the_localized_text() {
    fn app() -> Element {
        rsx! { LiberoProvider { Tldr { url: "https://libero-ui.dev/md/menu.md", label: "  " } } }
    }

    let html = body(&render(app));
    assert!(html.contains("TLDR"), "{html}");
}

#[test]
fn a_blank_aria_label_names_the_icon_only_trigger_in_the_localized_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tldr { url: "https://libero-ui.dev/md/menu.md", icon_only: true, aria_label: "  " }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains(r#"aria-label="Summarize with AI""#), "{html}");
}
