//! `NativeSelect` is `onchange` on a real `<select>`; `Select` is a listbox
//! libero draws itself.

use dioxus::prelude::*;
use libero::components::{NativeSelect, Options, Select};
use native_tests::{Key, mount};

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

fn native() -> Element {
    let mut value = use_signal(|| Some(Fruit::Apple));
    rsx! {
        NativeSelect {
            label: "Fruit",
            value: value(),
            onchange: move |next: Fruit| value.set(Some(next)),
        }
        span { id: "picked", "{value:?}" }
    }
}

fn listbox() -> Element {
    let mut value = use_signal(|| Some(Fruit::Apple));
    rsx! {
        Select {
            label: "Fruit",
            value: value(),
            onchange: move |next| value.set(next),
        }
        span { id: "picked", "{value:?}" }
    }
}

#[test]
#[ignore = "Blitz: a <select> opens no picker and ignores the arrows, so nothing fires change"]
fn arrow_down_on_a_native_select_picks_the_next_option() {
    let mut page = mount(native);
    page.focus("select");
    page.press(Key::ArrowDown);
    assert_eq!(page.text("#picked"), "Some(Banana)", "{}", page.tree());
}

#[test]
fn a_click_on_a_select_option_picks_it() {
    let mut page = mount(listbox);
    page.click("[role=combobox]");
    assert!(page.exists("[role=listbox]"), "{}", page.tree());
    page.click("[role=option]:nth-child(3)");
    assert_eq!(page.text("#picked"), "Some(Cherry)", "{}", page.tree());
}

#[test]
fn the_keyboard_picks_a_select_option() {
    let mut page = mount(listbox);
    page.focus("[role=combobox]");
    page.press(Key::ArrowDown);
    assert!(page.exists("[role=listbox]"), "{}", page.tree());
    page.press(Key::ArrowDown);
    page.press(Key::Enter);
    assert_eq!(page.text("#picked"), "Some(Banana)", "{}", page.tree());
}
