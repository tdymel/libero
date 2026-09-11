//! `Select` is a listbox libero draws itself. `NativeSelect` is a real
//! `<select>` on the web; natively, where that opens no picker, the same listbox.

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

fn native_labelled() -> Element {
    rsx! {
        NativeSelect {
            label: "Fruit",
            value: Some(Fruit::Cherry),
            onchange: |_: Fruit| {},
            option_label: |fruit: Fruit| format!("{fruit:?}!"),
        }
    }
}

#[test]
fn a_native_select_draws_a_listbox_natively() {
    let mut page = mount(native);
    assert!(!page.exists("select"), "{}", page.tree());
    page.click("[role=combobox]");
    page.click("[role=option]:nth-child(3)");
    assert_eq!(page.text("#picked"), "Some(Cherry)", "{}", page.tree());
}

#[test]
fn arrow_down_on_a_native_select_picks_the_next_option() {
    let mut page = mount(native);
    page.focus("[role=combobox]");
    page.press(Key::ArrowDown);
    page.press(Key::ArrowDown);
    page.press(Key::Enter);
    assert_eq!(page.text("#picked"), "Some(Banana)", "{}", page.tree());
}

#[test]
fn a_native_select_keeps_its_option_label() {
    let mut page = mount(native_labelled);
    assert_eq!(page.text("[role=combobox]"), "Cherry!", "{}", page.tree());
    page.click("[role=combobox]");
    assert_eq!(page.text("[role=option]"), "Apple!", "{}", page.tree());
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
