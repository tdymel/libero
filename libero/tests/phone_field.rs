//! `PhoneField`'s rendered contract: the `tel` input is the labelled control
//! and holds the *text*, a hidden input posts the E.164, and the picker is a
//! real button that only exists while `country_select` is on.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::PhoneField};

fn typed() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                country: "US",
                value: "+12133734253",
                oninput: move |_: String| {},
            }
        }
    }
}

fn posting() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                country: "US",
                name: "phone",
                value: "+12133734253",
                oninput: move |_: String| {},
            }
        }
    }
}

fn pinned() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                country: "DE",
                country_select: false,
                oninput: move |_: String| {},
            }
        }
    }
}

fn elsewhere() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                country: "US",
                value: "+4930123456",
                oninput: move |_: String| {},
            }
        }
    }
}

/// The control is an `<input>`, which `<label for>` can name outright - so this
/// field takes `use_field()` plain rather than `labelled_by()`.
#[test]
fn the_tel_input_is_the_labelled_control() {
    let html = body(&render(typed));

    let input = attributes_of(&html, "input");
    let id = input.get("id").expect("the input carries the field's id");
    assert_eq!(input.get("type").map(String::as_str), Some("tel"));
    assert_eq!(input.get("inputmode").map(String::as_str), Some("tel"));
    assert!(
        html.contains(&format!("for=\"{id}\"")),
        "the label does not name the input:\n{html}"
    );
    assert!(
        !html.contains("aria-labelledby"),
        "a labelable control should not be named the other way round:\n{html}"
    );
}

/// The visible input holds the national text, so it must not carry the `name`:
/// it would post what is on screen instead of the value.
#[test]
fn the_hidden_input_posts_the_e164_and_the_visible_one_holds_the_text() {
    let html = body(&render(posting));

    let visible = attributes_of(&html, "input");
    assert_eq!(
        visible.get("value").map(String::as_str),
        Some("213 373 4253")
    );
    assert!(
        !visible.contains_key("name"),
        "the visible input posts the display text:\n{html}"
    );
    assert!(
        html.contains(r#"type="hidden" name="phone" value="+12133734253""#),
        "no hidden input carrying the E.164:\n{html}"
    );
}

/// An existing number renders under its own country, whatever the field was
/// asked to start on - `country` is the initial pick, not an override.
#[test]
fn a_value_from_another_country_renders_under_that_country() {
    let html = body(&render(elsewhere));

    assert_eq!(
        attributes_of(&html, "input")
            .get("value")
            .map(String::as_str),
        Some("30123456")
    );
    assert!(html.contains(">DE<"), "the picker still says US:\n{html}");
    assert!(
        html.contains(">+49<"),
        "the dial code did not follow:\n{html}"
    );
}

/// The picker is a second tab stop, and should be - it is the only way to
/// reach what it does. `type="button"` because a button in a form submits it
/// otherwise.
#[test]
fn the_picker_is_a_button_that_says_a_listbox_hangs_off_it() {
    let html = body(&render(typed));

    let button = attributes_of(&html, "button");
    assert_eq!(button.get("type").map(String::as_str), Some("button"));
    assert_eq!(
        button.get("aria-haspopup").map(String::as_str),
        Some("listbox")
    );
    assert_eq!(
        button.get("aria-expanded").map(String::as_str),
        Some("false")
    );
    // The content reads `US +1`, which names a code and not a country.
    assert_eq!(
        button.get("aria-label").map(String::as_str),
        Some("Country: United States")
    );
    // Closed, there is no list to point at and nothing to announce.
    assert!(
        !html.contains("role=\"listbox\""),
        "a closed picker rendered its list:\n{html}"
    );
}

/// `country_select: false` pins the country and is one tab stop fewer: the
/// leading slot becomes a plain `+49` with nothing to open.
#[test]
fn a_pinned_country_renders_no_picker_at_all() {
    let html = body(&render(pinned));

    assert!(
        !html.contains("<button"),
        "the pinned field still has a picker:\n{html}"
    );
    assert!(html.contains("+49"), "the dial code is not drawn:\n{html}");
}
