//! Typed fields: `TextField`, `Textarea`, `PinField`, `NumberField`. Blitz
//! keeps a typed value off the attributes, so each app echoes its value.

use dioxus::prelude::*;
use libero::components::{NumberField, PasswordField, PhoneField, PinField, TextField, Textarea};
use native_tests::{Key, Page, mount};

fn typing(page: &mut Page, text: &str) {
    for c in text.chars() {
        page.press(Key::Character(c.to_string()));
    }
}

fn text() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        TextField { label: "Name", value: value(), oninput: move |next| value.set(next) }
        span { id: "echo", "{value}" }
    }
}

#[test]
fn typing_into_a_text_field_reaches_its_value() {
    let mut page = mount(text);
    page.click("input");
    assert!(page.is_focused("input"), "{}", page.focus_owner());
    typing(&mut page, "Ada");
    assert_eq!(page.text("#echo"), "Ada", "{}", page.tree());
    page.press(Key::Backspace);
    assert_eq!(page.text("#echo"), "Ad");
}

fn area() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        Textarea { label: "Note", value: value(), oninput: move |next| value.set(next) }
        span { id: "echo", "{value}" }
    }
}

#[test]
fn typing_into_a_textarea_reaches_its_value() {
    let mut page = mount(area);
    page.click("textarea");
    typing(&mut page, "hi");
    assert_eq!(page.text("#echo"), "hi", "{}", page.tree());
}

fn pin() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        PinField {
            label: "Code",
            length: 4usize,
            value: value(),
            oninput: move |next: String| value.set(next),
        }
        span { id: "echo", "{value}" }
    }
}

#[test]
fn a_pin_field_takes_one_digit_per_box_and_moves_on() {
    let mut page = mount(pin);
    page.click("input");
    typing(&mut page, "12");
    assert_eq!(page.text("#echo"), "12", "{}", page.tree());
    let boxes = page.query_all("input");
    assert!(
        page.focused() == Some(boxes[2]),
        "focus is on {}",
        page.focus_owner()
    );
    // The first press only moves back from the empty third box.
    page.press(Key::Backspace);
    page.press(Key::Backspace);
    assert_eq!(page.text("#echo"), "1", "{}", page.tree());
    assert!(page.focused() == Some(boxes[0]), "{}", page.focus_owner());
}

fn number() -> Element {
    let mut value = use_signal(|| 3i32);
    rsx! {
        NumberField {
            label: "Quantity",
            steppers: true,
            value: value(),
            onchange: move |next| value.set(next),
        }
        span { id: "echo", "{value}" }
    }
}

const SPIN: &str = "input[role=spinbutton]";

#[test]
fn the_steppers_step_a_number_field() {
    let mut page = mount(number);
    page.click("button[aria-label=Increase]");
    page.click("button[aria-label=Increase]");
    assert_eq!(page.text("#echo"), "5", "{}", page.tree());
    assert_eq!(page.attr(SPIN, "aria-valuenow").as_deref(), Some("5"));
    page.click("button[aria-label=Decrease]");
    assert_eq!(page.text("#echo"), "4");
}

#[test]
fn the_arrows_step_a_focused_number_field() {
    let mut page = mount(number);
    page.click(SPIN);
    page.press(Key::ArrowUp);
    assert_eq!(page.text("#echo"), "4", "{}", page.tree());
    page.press(Key::ArrowDown);
    page.press(Key::ArrowDown);
    assert_eq!(page.text("#echo"), "2");
}

fn password() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        PasswordField { label: "Password", value: value(), oninput: move |next| value.set(next) }
        span { id: "echo", "{value}" }
    }
}

#[test]
fn the_reveal_button_shows_a_password() {
    let mut page = mount(password);
    page.click("input");
    typing(&mut page, "pw");
    assert_eq!(page.text("#echo"), "pw", "{}", page.tree());
    assert_eq!(page.attr("input", "type").as_deref(), Some("password"));
    page.click("button[aria-label='Show password']");
    assert_eq!(
        page.attr("input", "type").as_deref(),
        Some("text"),
        "{}",
        page.tree()
    );
    assert!(page.exists("button[aria-label='Hide password']"));
}

fn phone() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        PhoneField {
            label: "Phone",
            country: "DE",
            value: value(),
            oninput: move |next: String| value.set(next),
        }
        span { id: "echo", "{value}" }
    }
}

#[test]
fn typing_a_phone_number_reaches_its_e164_value() {
    let mut page = mount(phone);
    page.click("input[type=tel]");
    typing(&mut page, "301234");
    assert_eq!(page.text("#echo"), "+49301234", "{}", page.tree());
}

#[test]
fn typing_a_number_commits_it() {
    let mut page = mount(number);
    page.click(SPIN);
    page.press(Key::Backspace);
    typing(&mut page, "42");
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "42", "{}", page.tree());
}
