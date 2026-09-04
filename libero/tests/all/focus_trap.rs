use crate::common::{body, classes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{FocusTrap, FocusTrapInitialFocus},
};

#[test]
fn focus_trap_renders_its_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FocusTrap { "trapped" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("trapped"));
}

/// `FocusTrap` is `display: contents`, so nothing can host an absolute
/// initial-focus span; a fixed one never makes the document scroll to it.
#[test]
fn focus_trap_initial_focus_is_fixed() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FocusTrap {
                    FocusTrapInitialFocus {}
                }
            }
        }
    }

    let html = render(app);
    let span = classes_of(&html, "span");
    assert!(
        span.iter()
            .any(|class| html.contains(&format!(".{class}{{position:fixed"))),
        "{span:?}"
    );
}
