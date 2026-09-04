use crate::common::{attributes_of, body, classes_of, has_rule_for, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Paper};

#[test]
fn a_paper_is_a_surface_with_no_semantics_of_its_own() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { radius: "lg", bordered: true, "sheet" }
            }
        }
    }

    let html = render(app);
    let paper = attributes_of(&body(&html), "div");

    assert_eq!(paper["data-state"], "radius-lg bordered");
    assert!(
        !paper.contains_key("role"),
        "a surface names nothing: {paper:?}"
    );
    for class in classes_of(&body(&html), "div") {
        assert!(
            has_rule_for(&html, &class),
            "{class} is referenced but never emitted"
        );
    }
    assert!(
        html.contains("background:var(--lsx-paper-background);"),
        "{html}"
    );
}

/// The themed default arrives as a plain declaration, not as a token - which
/// is what lets `Dialog` override it in its own base static.
#[test]
fn a_paper_that_names_no_step_carries_no_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Paper { "sheet" }
            }
        }
    }

    let paper = attributes_of(&body(&render(app)), "div");

    assert!(!paper.contains_key("data-state"), "{paper:?}");
}
