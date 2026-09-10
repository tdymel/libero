//! A selectable `Chip` is a hidden checkbox beside its `<label>`, the shape
//! `Switch` had: Blitz toggles the input and reports `input`, not `click`.

use dioxus::prelude::*;
use libero::components::Chip;
use native_tests::{Key, Page, mount};

const INPUT: &str = "input[type=checkbox]";

fn app() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Chip { checked: on(), onchange: move |v| on.set(v), "Wifi" }
    }
}

fn checked(page: &Page) -> bool {
    page.attr(INPUT, "checked").is_some_and(|v| v == "true")
}

#[test]
fn a_click_on_the_label_toggles_it() {
    let mut page = mount(app);
    page.click("label");
    assert!(checked(&page), "{}", page.tree());
    page.click("label");
    assert!(!checked(&page), "{}", page.tree());
}

#[test]
fn space_toggles_the_focused_chip() {
    let mut page = mount(app);
    page.focus(INPUT);
    page.press(Key::Character(" ".into()));
    assert!(checked(&page), "{}", page.tree());
    page.press(Key::Character(" ".into()));
    assert!(!checked(&page), "{}", page.tree());
}
