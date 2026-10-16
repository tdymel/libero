//! macOS Blitz runs its delete binding after a Backspace the cell already handled (todo 3342).

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::{PinField, Text};

fn app() -> Element {
    let mut pin = use_signal(String::new);
    rsx! {
        PinField { label: "Code", length: 4usize, value: pin(), oninput: move |next: String| pin.set(next) }
        Text { id: "echo", "{pin}" }
    }
}

fn focused_cell(page: &e2e::native::Page) -> String {
    let owner = page.focus_owner().to_string();
    owner.split("data-pin-index=\"").nth(1).map_or(String::new(), |rest| rest[..1].to_string())
}

#[test]
fn a_backspace_binding_does_not_undo_the_cells_own_edit() {
    let mut page = mount(app);
    page.click("[data-pin-index='0']");
    for digit in ["1", "2"] {
        page.press(Key::Character(digit.into()));
    }
    page.press_with_binding(Key::Backspace);
    assert_eq!((page.text("#echo").as_str(), focused_cell(&page).as_str()), ("12", "1"));
    page.press_with_binding(Key::Backspace);
    assert_eq!((page.text("#echo").as_str(), focused_cell(&page).as_str()), ("1", "0"));
    page.press_with_binding(Key::Backspace);
    assert_eq!((page.text("#echo").as_str(), focused_cell(&page).as_str()), ("", "0"));
}
