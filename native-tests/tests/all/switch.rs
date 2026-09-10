//! `Switch` is a visually hidden checkbox with `role="switch"`: the track takes
//! the pointer, the input the keyboard.

use dioxus::prelude::*;
use libero::components::Switch;
use native_tests::{Key, Page, mount};

const INPUT: &str = "[role=switch]";
const TRACK: &str = "[role=switch] + [aria-hidden=true]";

fn app() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Switch { checked: on(), onchange: move |v| on.set(v), label: "Wifi" }
    }
}

fn checked(page: &Page) -> bool {
    page.attr(INPUT, "checked").is_some_and(|v| v == "true")
}

#[test]
fn a_click_on_the_track_toggles_it() {
    let mut page = mount(app);
    assert!(!checked(&page));
    page.click(TRACK);
    assert!(checked(&page), "{}", page.tree());
    page.click(TRACK);
    assert!(!checked(&page));
}

#[test]
fn a_click_on_the_label_toggles_it() {
    let mut page = mount(app);
    page.click("label");
    assert!(checked(&page), "{}", page.tree());
}

#[test]
fn the_thumb_moves_once_its_transition_ends() {
    const THUMB: &str = "[role=switch] + [aria-hidden=true] > span";
    let mut page = mount(app);
    let off = page.computed(THUMB, "transform");
    page.click(TRACK);
    page.advance(1.0);
    let on = page.computed(THUMB, "transform");
    assert_ne!(off, on, "the thumb stayed at {off}");
}

#[test]
fn space_and_enter_toggle_the_focused_switch() {
    let mut page = mount(app);
    page.focus(INPUT);
    page.press(Key::Character(" ".into()));
    assert!(checked(&page), "Space");
    page.press(Key::Enter);
    assert!(!checked(&page), "Enter");
    assert!(page.is_focused(INPUT));
}
