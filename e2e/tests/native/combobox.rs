//! `Autocomplete`'s narrowed labels as Blitz lays them out (627). The family's
//! keys and clicks are e2e's shared scenarios (`combobox::`).

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::Autocomplete;

const TRIGGER: &str = "[role=combobox]";

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
