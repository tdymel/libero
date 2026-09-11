//! `Carousel`'s track keys natively: an arrow in a field inside a slide stays
//! the field's (`typing_target`, `arrow_target`).

use dioxus::prelude::*;
use libero::components::Carousel;
use native_tests::{Key, mount};

const CURRENT: &str = "[data-current=true]";

fn app() -> Element {
    rsx! {
        Carousel {
            aria_label: "Photos",
            slides: vec![
                rsx! { input { id: "text", r#type: "text" } },
                rsx! { input { id: "range", r#type: "range" } },
                rsx! { div { "slide 2" } },
            ],
        }
    }
}

#[test]
fn an_arrow_on_the_track_moves_to_the_next_slide() {
    let mut page = mount(app);
    let first = page.node(CURRENT);
    page.focus("[aria-roledescription=carousel] [tabindex='0']");
    page.press(Key::ArrowRight);
    assert_ne!(
        page.node(CURRENT),
        first,
        "ArrowRight on {} did not move:\n{}",
        page.focus_owner(),
        page.tree()
    );
}

#[test]
fn an_arrow_in_a_text_field_stays_in_the_field() {
    let mut page = mount(app);
    let first = page.node(CURRENT);
    page.focus("#text");
    page.press(Key::ArrowRight);
    assert_eq!(
        page.node(CURRENT),
        first,
        "the carousel took the caret's arrow"
    );
}

#[test]
fn an_arrow_on_a_range_stays_on_the_range() {
    let mut page = mount(app);
    page.focus("#range");
    let first = page.node(CURRENT);
    page.press(Key::ArrowRight);
    assert_eq!(
        page.node(CURRENT),
        first,
        "the carousel took the range's arrow"
    );
}
