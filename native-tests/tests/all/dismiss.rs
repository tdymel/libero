//! `use_dismiss` natively, through `Menu` (todo 46): a click opens it, and
//! focus moving from the trigger into the list keeps it open. The outside
//! click is in `menu.rs`.

use dioxus::prelude::*;
use libero::components::{Button, Menu, MenuItem, use_menu};
use native_tests::mount;

const TRIGGER: &str = "[aria-haspopup]";

fn app() -> Element {
    let menu = use_menu();
    rsx! {
        Menu {
            state: menu,
            items: vec![
                MenuItem::new("Copy").onselect(|_| {}).into(),
                MenuItem::new("Paste").onselect(|_| {}).into(),
            ],
            Button { attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}

#[test]
fn a_click_opens_it_and_focus_moving_into_the_list_keeps_it_open() {
    let mut page = mount(app);
    page.click(TRIGGER);
    assert_eq!(
        page.attr(TRIGGER, "aria-expanded").as_deref(),
        Some("true"),
        "the click did not open it:\n{}",
        page.tree()
    );
    assert!(page.exists("[role=menu]"));
    assert!(
        page.is_focused("[role=menuitem]"),
        "focus is on {}, not an item",
        page.focus_owner()
    );
}
