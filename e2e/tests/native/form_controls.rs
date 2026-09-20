//! Raw form controls follow the theme natively (todo 955): Blitz's UA sheet
//! paints `input`/`textarea` white whatever the scheme.

use dioxus::prelude::*;
use e2e::native::{ColorScheme, Page, mount_in};
use libero::components::TextField;

fn app() -> Element {
    rsx! {
        div { id: "paper", style: "background-color: var(--lsx-paper-background)" }
        input { id: "text" }
        textarea { id: "area" }
        select { id: "pick", option { "One" } }
        input { id: "check", r#type: "checkbox" }
        div { id: "field", TextField { aria_label: "Name" } }
    }
}

fn background(page: &Page, selector: &str) -> String {
    page.computed(selector, "background-color")
}

#[test]
fn raw_form_controls_take_the_dark_paper() {
    let page = mount_in(app, ColorScheme::Dark);
    let paper = background(&page, "#paper");
    for control in ["#text", "#area", "#pick"] {
        assert_eq!(background(&page, control), paper, "{control}");
        // Blitz's UA sheet sets no text colour, so the page's light ink reads.
        assert_eq!(
            page.computed(control, "color"),
            page.computed("body", "color"),
            "{control}"
        );
    }
}

/// The rule sits in the base layer, so a component's own control keeps its
/// transparent background, and a checkbox keeps the UA look.
#[test]
fn styled_and_toggle_controls_are_left_alone() {
    let page = mount_in(app, ColorScheme::Dark);
    let paper = background(&page, "#paper");
    assert_ne!(background(&page, "#field input"), paper);
    assert_ne!(background(&page, "#check"), paper);
}
