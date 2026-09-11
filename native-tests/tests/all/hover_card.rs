//! `HoverCard` opened by the pointer while focus sits elsewhere: Escape has to
//! close it from wherever focus is (SC 1.4.13).

use std::time::Duration;

use dioxus::prelude::*;
use libero::components::{Button, HoverCard, Options, Select};
use native_tests::{Key, mount};

const CARD: &str = "[role=dialog]";

fn app() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        HoverCard {
            aria_label: "Ada Lovelace",
            open_delay: 10,
            close_delay: 10,
            content: rsx! { Button { id: "inside", "Profile" } },
            Button { id: "trigger", "Ada Lovelace" }
        }
    }
}

#[test]
fn escape_on_the_trigger_closes_a_pointer_opened_card() {
    let mut page = mount(app);
    page.focus("#trigger");
    page.hover("#trigger");
    page.wait(Duration::from_millis(100));
    assert!(
        page.exists(CARD),
        "hovering did not open it:\n{}",
        page.tree()
    );

    page.press(Key::Escape);
    assert!(
        !page.exists(CARD),
        "Escape did not close it:\n{}",
        page.tree()
    );
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Fruit {
    Apple,
    Banana,
}

fn select_in_card() -> Element {
    let mut value = use_signal(|| Some(Fruit::Apple));
    rsx! {
        HoverCard {
            aria_label: "Ada Lovelace",
            open_delay: 10,
            close_delay: 10,
            content: rsx! {
                Select { label: "Fruit", value: value(), onchange: move |next| value.set(next) }
            },
            Button { id: "trigger", "Ada Lovelace" }
        }
    }
}

/// An open field list is its own layer: one Escape closes the list, the next
/// the card (todo 348).
#[test]
fn escape_closes_a_select_list_in_the_card_before_the_card() {
    let mut page = mount(select_in_card);
    page.hover("#trigger");
    page.wait(Duration::from_millis(100));
    page.focus("[role=combobox]");
    page.press(Key::ArrowDown);
    assert!(page.exists("[role=listbox]"), "{}", page.tree());

    page.press(Key::Escape);
    assert!(!page.exists("[role=listbox]"), "Escape left the list open");
    assert!(page.exists(CARD), "Escape closed the card with the list");

    page.press(Key::Escape);
    assert!(
        !page.exists(CARD),
        "the second Escape did not close the card"
    );
}

#[test]
fn escape_closes_a_pointer_opened_card_with_focus_elsewhere() {
    let mut page = mount(app);
    page.focus("#before");
    page.hover("#trigger");
    page.wait(Duration::from_millis(100));
    assert!(
        page.exists(CARD),
        "hovering did not open it:\n{}",
        page.tree()
    );

    page.press(Key::Escape);
    assert!(
        !page.exists(CARD),
        "Escape did not close it:\n{}",
        page.tree()
    );
    assert!(
        page.is_focused("#before"),
        "focus moved to {}",
        page.focus_owner()
    );
}
