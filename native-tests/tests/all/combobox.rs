//! The combobox family: `Autocomplete`, `MultiSelect`, `TagsField`,
//! `Cascader`. Keys on the trigger, a click on a portaled option.

use dioxus::prelude::*;
use libero::components::{Autocomplete, Cascader, CascaderOption, MultiSelect, Options, TagsField};
use native_tests::{Key, Page, mount};

const TRIGGER: &str = "[role=combobox]";
const LISTBOX: &str = "[role=listbox]";

fn typing(page: &mut Page, text: &str) {
    for c in text.chars() {
        page.press(Key::Character(c.to_string()));
    }
}

fn expanded(page: &Page) -> bool {
    page.attr(TRIGGER, "aria-expanded")
        .is_some_and(|v| v == "true")
}

// As Spotlight's (627): a long list, and "G" keeps the first two rows in place.
fn narrowing() -> Element {
    let mut value = use_signal(String::new);
    let options = use_hook(|| {
        let mut all: Vec<String> = ["Getting Started", "Theming", "Select", "MultiSelect"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        all.extend((0..60).map(|n| format!("Item {n}")));
        all
    });
    rsx! {
        Autocomplete {
            label: "Page",
            options,
            value: value(),
            oninput: move |next| value.set(next),
        }
    }
}

#[test]
fn a_narrowed_autocomplete_keeps_each_label_on_one_line() {
    let mut page = mount(narrowing);
    page.focus(TRIGGER);
    page.press(Key::ArrowDown);
    assert!(
        page.query_all("[role=option]").len() > 60,
        "{}",
        page.tree()
    );
    page.press_before_layout(Key::Character("G".into()));
    assert_eq!(page.query_all("[role=option]").len(), 2, "{}", page.tree());
    assert_eq!(page.wrapped_text("[role=option]"), Vec::<String>::new());
}

fn autocomplete() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        Autocomplete {
            label: "City",
            options: ["Amsterdam", "Berlin", "Copenhagen", "Dublin"]
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>(),
            value: value(),
            oninput: move |next| value.set(next),
        }
        span { id: "echo", "{value}" }
    }
}

#[test]
fn typing_filters_an_autocomplete_and_enter_picks_the_option() {
    let mut page = mount(autocomplete);
    page.click(TRIGGER);
    typing(&mut page, "be");
    assert!(expanded(&page), "{}", page.tree());
    assert_eq!(page.query_all("[role=option]").len(), 1, "{}", page.tree());
    page.press(Key::ArrowDown);
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "Berlin", "{}", page.tree());
    assert!(!expanded(&page));
}

#[test]
fn escape_closes_an_open_autocomplete() {
    let mut page = mount(autocomplete);
    page.focus(TRIGGER);
    page.press(Key::ArrowDown);
    assert!(page.exists(LISTBOX), "{}", page.tree());
    page.press(Key::Escape);
    assert!(!expanded(&page), "{}", page.tree());
    assert!(page.is_focused(TRIGGER), "{}", page.focus_owner());
}

#[test]
fn a_click_on_an_autocomplete_option_picks_it() {
    let mut page = mount(autocomplete);
    page.focus(TRIGGER);
    page.press(Key::ArrowDown);
    page.click("[role=option]:nth-child(3)");
    assert_eq!(page.text("#echo"), "Copenhagen", "{}", page.tree());
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

fn multi() -> Element {
    let mut fruits = use_signal(|| vec![Fruit::Cherry]);
    rsx! {
        MultiSelect { label: "Fruits", value: fruits(), onchange: move |next| fruits.set(next) }
        span { id: "echo", "{fruits:?}" }
    }
}

#[test]
fn the_keyboard_adds_a_multi_select_option() {
    let mut page = mount(multi);
    page.focus(TRIGGER);
    // Opens on the held Cherry: Enter drops it, the list stays open.
    page.press(Key::ArrowDown);
    assert!(page.exists(LISTBOX), "{}", page.tree());
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "[]", "{}", page.tree());
    assert!(expanded(&page), "{}", page.tree());
    page.press(Key::ArrowUp);
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "[Banana]", "{}", page.tree());
}

#[test]
fn a_click_on_a_multi_select_option_toggles_it() {
    let mut page = mount(multi);
    page.click(TRIGGER);
    assert!(page.exists(LISTBOX), "{}", page.tree());
    page.click("[role=option]:nth-child(2)");
    assert_eq!(page.text("#echo"), "[Cherry, Banana]", "{}", page.tree());
}

/// The rows cancel `mousedown` to keep focus on the trigger, which closes on
/// blur. Blitz moves focus on the press regardless; libero moves it back and
/// the trigger ignores that blur (todo 472).
#[test]
fn a_click_on_a_multi_select_option_keeps_the_list_open() {
    let mut page = mount(multi);
    page.click(TRIGGER);
    page.click("[role=option]:nth-child(2)");
    assert!(expanded(&page), "{}", page.tree());
    assert!(page.is_focused(TRIGGER), "{}", page.focus_owner());
}

#[test]
fn backspace_in_an_empty_multi_select_drops_the_last_pick() {
    let mut page = mount(multi);
    page.focus(TRIGGER);
    page.press(Key::Backspace);
    assert_eq!(page.text("#echo"), "[]", "{}", page.tree());
}

fn tags() -> Element {
    let mut topics = use_signal(|| vec!["rust".to_string()]);
    rsx! {
        TagsField {
            label: "Topics",
            suggestions: ["rust", "dioxus", "wasm"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            value: topics(),
            onchange: move |next| topics.set(next),
        }
        span { id: "echo", "{topics:?}" }
    }
}

#[test]
fn typing_and_enter_add_a_tag() {
    let mut page = mount(tags);
    page.click(TRIGGER);
    typing(&mut page, "css");
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), r#"["rust", "css"]"#, "{}", page.tree());
}

#[test]
fn a_suggestion_picked_by_the_keys_becomes_a_tag() {
    let mut page = mount(tags);
    page.focus(TRIGGER);
    page.press(Key::ArrowDown);
    assert!(page.exists(LISTBOX), "{}", page.tree());
    page.press(Key::Enter);
    assert_eq!(
        page.text("#echo"),
        r#"["rust", "dioxus"]"#,
        "{}",
        page.tree()
    );
}

#[test]
fn backspace_in_an_empty_tags_field_removes_the_last_tag() {
    let mut page = mount(tags);
    page.click(TRIGGER);
    page.press(Key::Backspace);
    page.press(Key::Backspace);
    assert_eq!(page.text("#echo"), "[]", "{}", page.tree());
}

fn cascader() -> Element {
    let mut place = use_signal(|| None::<String>);
    let data = vec![
        CascaderOption::new("europe", "Europe").children(vec![
            CascaderOption::new("france", "France")
                .children(vec![CascaderOption::new("paris", "Paris")]),
            CascaderOption::new("germany", "Germany")
                .children(vec![CascaderOption::new("berlin", "Berlin")]),
        ]),
        CascaderOption::new("oceania", "Oceania").children(vec![
            CascaderOption::new("australia", "Australia")
                .children(vec![CascaderOption::new("sydney", "Sydney")]),
        ]),
    ];
    rsx! {
        Cascader {
            label: "Place",
            data,
            value: place(),
            onchange: move |next: Option<String>| place.set(next),
        }
        span { id: "echo", {place().unwrap_or_default()} }
    }
}

#[test]
fn the_keys_walk_a_cascader_to_a_leaf() {
    let mut page = mount(cascader);
    page.focus(TRIGGER);
    page.press(Key::ArrowDown);
    assert!(expanded(&page), "{}", page.tree());
    // Europe -> France -> Paris.
    page.press(Key::ArrowRight);
    page.press(Key::ArrowRight);
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "paris", "{}", page.tree());
}

/// A click on a branch opens its column. Natively the press blurs the
/// trigger all the same, which must not close the list (todo 472).
#[test]
fn clicks_walk_a_cascader_to_a_leaf() {
    let mut page = mount(cascader);
    page.click(TRIGGER);
    assert!(expanded(&page), "{}", page.tree());
    page.click("[id$=-option-0-1]");
    assert!(expanded(&page), "the click on Oceania closed the list");
    page.click("[id$=-option-1-0]");
    page.click("[id$=-option-2-0]");
    assert_eq!(page.text("#echo"), "sydney", "{}", page.tree());
}
