//! Blitz's editor ignores `maxlength`; libero cuts a paste to it (todo 950),
//! where it was inserted, before a caller hears it.

use dioxus::prelude::*;
use e2e::native::{Key, MOD, Page, mount};
use libero::components::{Text, TextField, Textarea};

fn app() -> Element {
    let mut code = use_signal(|| "ab".to_string());
    rsx! {
        TextField { id: "code", maxlength: "5", value: code(), oninput: move |next| code.set(next) }
        Text { id: "echo", "{code}" }
        Textarea { id: "note", counter: true, maxlength: "6" }
        input { id: "raw", maxlength: "4" }
    }
}

fn paste(page: &mut Page, text: &str) {
    page.set_clipboard(text);
    page.press_with(Key::Character("v".into()), MOD);
}

#[test]
fn a_paste_into_a_controlled_field_is_cut_where_it_lands() {
    let mut page = mount(app);
    page.focus("#code");
    page.press(Key::Named(NamedKey::Home));
    paste(&mut page, "XYZW");
    assert_eq!(page.text("#echo"), "XYZab");
    assert_eq!(page.editor_text("#code"), "XYZab");
}

#[test]
fn a_paste_over_a_selection_at_the_limit_is_cut() {
    let mut page = mount(app);
    page.focus("#raw");
    for key in ["a", "b", "c", "d"] {
        page.press(Key::Character(key.into()));
    }
    page.press(Key::Named(NamedKey::Home));
    page.press(Key::Named(NamedKey::ArrowRight));
    page.press_with(Key::Named(NamedKey::ArrowRight), Modifiers::SHIFT);
    paste(&mut page, "xyz");
    assert_eq!(page.editor_text("#raw"), "axcd");
}

#[test]
fn a_paste_into_an_uncontrolled_textarea_or_a_raw_input_is_cut() {
    let mut page = mount(app);
    page.focus("#note");
    paste(&mut page, "123456789");
    assert_eq!(page.editor_text("#note"), "123456");
    assert_eq!(page.text("[data-slot=counter]"), "6/6");

    page.focus("#raw");
    paste(&mut page, "abcdef");
    assert_eq!(page.editor_text("#raw"), "abcd");
}
