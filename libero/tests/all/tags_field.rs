//! `TagsField`'s rendered contract: the input is the draft and the labelled
//! control, the tags are chips whose x a keyboard can reach, and the value
//! posts through hidden inputs because the visible one cannot carry it.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{SelectionArgs, TagsField},
};

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
/// stop, the way `MultiSelect`'s chips are, and Backspace
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

/// The live region mounts empty: tags already held when the field first
/// renders are not news, so only a later change is announced (todo 70).
#[test]
fn the_tags_held_at_mount_are_not_announced() {
    let html = body(&render(tagged));

    assert!(html.contains("rust"), "the fixture holds no tag:\n{html}");
    assert!(
        html.contains(r#"role="status"></span>"#) && !html.contains("Added"),
        "the live region is missing, or spoke at mount:\n{html}"
    );
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {

    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{SelectionArgs, TagsField},
    };

    #[test]
    fn pressing_a_tag_x_keeps_the_focus_on_the_input() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    TagsField {
                        label: "Topics",
                        value: vec!["rust".to_string(), "dioxus".to_string()],
                        onchange: move |_: Vec<String>| {},
                    }
                }
            }
        }

        // The default chip's guard, and its wrapper's.
        assert_every_press_keeps_the_focus(app, 4);
    }

    /// Todo 70 (c): a caller's own `tag` draws its x without any guard, and the
    /// field's wrapper supplies it - two chips, two guards, both cancelling.
    #[test]
    fn a_custom_tag_gets_the_guard_it_did_not_draw() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    TagsField {
                        label: "Topics",
                        value: vec!["rust".to_string(), "dioxus".to_string()],
                        onchange: move |_: Vec<String>| {},
                        tag: move |args: SelectionArgs<String>| rsx! {
                            span { "{args.value}"
                                button { tabindex: "-1", onclick: move |_| args.remove.call(()), "x" }
                            }
                        },
                    }
                }
            }
        }

        assert_every_press_keeps_the_focus(app, 2);
    }

    #[test]
    fn clear_empties_a_tags_field_once() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    TagsField {
                        label: "Topics",
                        clearable: true,
                        value: vec!["rust".to_string(), "dioxus".to_string()],
                        onchange: move |tags: Vec<String>| CLEARED.with_borrow_mut(|seen| seen.push(tags.len())),
                    }
                }
            }
        }

        assert_eq!(click_clear(app), Some(vec![0]));
    }

    #[test]
    fn an_empty_field_draws_no_clear_button() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    TagsField {
                        label: "Topics",
                        clearable: true,
                        value: Vec::<String>::new(),
                        onchange: move |_: Vec<String>| {},
                    }
                }
            }
        }

        assert_eq!(click_clear(app), None);
    }
}

/// Todo 1888: a caller's tag hears what the default chip does, so it can drop
/// or disable its remove control.
#[test]
fn a_custom_tag_hears_disabled_and_readonly() {
    fn app() -> Element {
        let tag = move |args: SelectionArgs<String>| {
            rsx! {
                span { "{args.value}:{args.disabled}:{args.readonly}" }
            }
        };
        rsx! {
            LiberoProvider {
                TagsField { label: "A", value: vec!["a".to_string()], onchange: move |_: Vec<String>| {}, tag }
                TagsField { label: "B", value: vec!["b".to_string()], onchange: move |_: Vec<String>| {}, disabled: true, tag }
                TagsField { label: "C", value: vec!["c".to_string()], onchange: move |_: Vec<String>| {}, readonly: true, tag }
            }
        }
    }
    let html = body(&render(app));

    for drawn in ["a:false:false", "b:true:false", "c:false:true"] {
        assert!(html.contains(drawn), "no {drawn:?} in\n{html}");
    }
}
