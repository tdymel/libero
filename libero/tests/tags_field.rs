//! `TagsField`'s rendered contract: the input is the draft and the labelled
//! control, the tags are chips whose x a keyboard can reach, and the value
//! posts through hidden inputs because the visible one cannot carry it.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::TagsField};

fn tagged() -> Element {
    rsx! {
        LiberoProvider {
            TagsField {
                label: "Topics",
                placeholder: "Add a topic",
                value: vec!["rust".to_string(), "dioxus".to_string()],
                onchange: move |_: Vec<String>| {},
            }
        }
    }
}

fn empty() -> Element {
    rsx! {
        LiberoProvider {
            TagsField {
                label: "Topics",
                placeholder: "Add a topic",
                onchange: move |_: Vec<String>| {},
            }
        }
    }
}

fn suggesting() -> Element {
    rsx! {
        LiberoProvider {
            TagsField {
                label: "Topics",
                suggestions: vec!["rust".to_string()],
                onchange: move |_: Vec<String>| {},
            }
        }
    }
}

fn posting() -> Element {
    rsx! {
        LiberoProvider {
            TagsField {
                label: "Topics",
                name: "topics",
                value: vec!["rust".to_string(), "dioxus".to_string()],
                onchange: move |_: Vec<String>| {},
            }
        }
    }
}

/// The control is an `<input>`, which `<label for>` can name outright - so this
/// field does *not* take `use_field().labelled_by()`, the way `Select`'s `div`
/// trigger has to.
#[test]
fn the_input_is_the_labelled_control() {
    let html = body(&render(tagged));

    let input = attributes_of(&html, "input");
    let id = input.get("id").expect("the input carries the field's id");
    assert!(
        html.contains(&format!("for=\"{id}\"")),
        "the label does not name the input:\n{html}"
    );
    assert!(
        !html.contains("aria-labelledby"),
        "a labelable control should not be named the other way round:\n{html}"
    );
}

/// One wrapper per tag, and its x is **not** a tab stop: the field is one tab
/// stop, the way `MultiSelect`'s chips and Mantine's `Pill` are, and Backspace
/// is how a keyboard takes a tag back.
#[test]
fn every_tag_is_a_chip_whose_remove_button_is_not_a_tab_stop() {
    let html = body(&render(tagged));

    assert_eq!(
        html.matches("data-slot=\"tag\"").count(),
        2,
        "one wrapper per tag:\n{html}"
    );
    assert!(html.contains(">rust<"), "{html}");
    assert!(html.contains(">dioxus<"), "{html}");
    assert!(
        html.contains("aria-label=\"Remove rust\""),
        "an icon-only x needs a name:\n{html}"
    );
    assert_eq!(
        html.matches("tabindex=\"-1\"").count(),
        2,
        "one x per tag, each out of the tab order:\n{html}"
    );
}

/// The placeholder describes an empty field, and a field holding chips is not
/// one - it would otherwise sit beside them asking for the first tag again.
#[test]
fn the_placeholder_stands_down_once_a_tag_is_held() {
    assert!(
        body(&render(empty)).contains("placeholder=\"Add a topic\""),
        "an empty field shows it"
    );
    assert!(
        !body(&render(tagged)).contains("placeholder=\"Add a topic\""),
        "a field with tags does not"
    );
}

/// Without `suggestions` there is no listbox for `aria-controls` to name and
/// nothing for the arrows to move, so the input is what it looks like.
#[test]
fn the_input_is_only_a_combobox_when_there_are_suggestions() {
    let plain = attributes_of(&body(&render(empty)), "input");
    assert!(!plain.contains_key("role"), "{plain:?}");
    assert!(!plain.contains_key("aria-expanded"), "{plain:?}");
    assert!(!plain.contains_key("aria-autocomplete"), "{plain:?}");

    let combobox = attributes_of(&body(&render(suggesting)), "input");
    assert_eq!(combobox.get("role").map(String::as_str), Some("combobox"));
    assert_eq!(
        combobox.get("aria-autocomplete").map(String::as_str),
        Some("list")
    );
}

/// The visible input holds the *draft*, so it cannot be what posts. One hidden
/// input per tag is - the shape `MultiSelect` already uses, and the one a
/// native `<select multiple>` sends.
#[test]
fn the_list_posts_through_one_hidden_input_per_tag() {
    let html = body(&render(posting));

    assert_eq!(
        html.matches("type=\"hidden\"").count(),
        2,
        "one per tag:\n{html}"
    );
    assert!(html.contains("value=\"rust\""), "{html}");
    assert!(html.contains("value=\"dioxus\""), "{html}");
    assert_eq!(
        html.matches("name=\"topics\"").count(),
        2,
        "only the hidden inputs carry the name - the draft must not post:\n{html}"
    );
}
