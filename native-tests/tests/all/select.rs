//! `NativeSelect` is a real `<select>` on the web; natively, where that opens
//! no picker, libero's listbox. `Select`'s keys and clicks are e2e's shared
//! scenarios (`select::`).

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

// As Spotlight's (627): a long list, and "g" keeps the first two rows in place.
fn narrowing() -> Element {
    let mut value = use_signal(|| None::<String>);
    let options = use_hook(|| {
        let mut all: Vec<String> = ["Getting Started", "Theming", "Select", "MultiSelect"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        all.extend((0..60).map(|n| format!("Item {n}")));
        all
    });
    rsx! {
        Select {
            label: "Page",
            options,
            searchable: true,
            value: value(),
            onchange: move |next| value.set(next),
        }
    }
}

#[test]
fn a_narrowed_select_keeps_each_label_on_one_line() {
    let mut page = mount(narrowing);
    page.click("[role=combobox]");
    assert!(
        page.query_all("[role=option]").len() > 60,
        "{}",
        page.tree()
    );
    page.press_before_layout(Key::Character("g".into()));
    assert_eq!(page.query_all("[role=option]").len(), 2, "{}", page.tree());
    assert_eq!(page.wrapped_text("[role=option]"), Vec::<String>::new());
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
