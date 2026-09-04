use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::VisuallyHidden};

#[test]
fn visually_hidden_stays_in_the_accessibility_tree() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                VisuallyHidden { "screen reader only" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("screen reader only"));
}
