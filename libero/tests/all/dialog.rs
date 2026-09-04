use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Dialog};

/// `Dialog` is a `Paper`, so the surface has to reach it - and the dialog's
/// own chrome has to survive the composition.
#[test]
fn a_dialog_renders_the_paper_surface_under_its_own_chrome() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Dialog { aria_label: "Inline", "content" }
            }
        }
    }

    let html = render(app);
    let dialog = attributes_of(&body(&html), "div");

    assert_eq!(dialog["role"], "dialog");
    assert!(
        html.contains("background:var(--lsx-paper-background);"),
        "{html}"
    );
    assert!(
        html.contains("border-radius:var(--lsx-dialog-radius, var(--lsx-paper-radius));"),
        "{html}"
    );
    assert!(html.contains("box-shadow:var(--lsx-shadow-xl);"), "{html}");
}
