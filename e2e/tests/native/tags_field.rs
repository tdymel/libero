//! macOS Blitz runs its delete binding after a Backspace the draft input already handled (todo 3625).

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::{TagsField, Text};

const INPUT: &str = "[role=combobox]";

fn app() -> Element {
    let mut tags = use_signal(|| vec!["alpha".to_string(), "beta".to_string()]);
    rsx! {
        TagsField {
            label: "Topics",
            value: tags(),
            onchange: move |next: Vec<String>| tags.set(next),
            suggestions: vec!["gamma".to_string(), "delta".to_string()],
        }
        Text { id: "echo", "{tags().join(\",\")}" }
    }
}

#[test]
fn a_backspace_binding_does_not_open_the_suggestions_after_it_drops_a_tag() {
    let mut page = mount(app);
    page.focus(INPUT);
    page.press_with_binding(Key::Named(NamedKey::Backspace));
    assert_eq!(page.text("#echo"), "alpha");
    assert_eq!(page.attr(INPUT, "aria-expanded").as_deref(), Some("false"));
}
