//! `Menu`, APG's menu button: Enter on the trigger opens it, Escape closes it
//! and hands focus back to the trigger. A click outside closes it too.

use dioxus::prelude::*;
use libero::components::{Button, Menu, MenuItem, use_menu};
use native_tests::{Key, Page, mount};

const TRIGGER: &str = "[aria-haspopup]";
const MENU: &str = "[role=menu]";

fn app() -> Element {
    let menu = use_menu();
    rsx! {
        Button { id: "elsewhere", "Elsewhere" }
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

fn expanded(page: &Page) -> bool {
    page.attr(TRIGGER, "aria-expanded")
        .is_some_and(|v| v == "true")
}

#[test]
fn enter_opens_it_and_escape_closes_it_with_focus_back_on_the_trigger() {
    let mut page = mount(app);
    page.focus(TRIGGER);
    assert!(!expanded(&page));

    page.press(Key::Enter);
    // The item takes focus once the box is placed, a timer later.
    page.wait(std::time::Duration::from_millis(50));
    assert!(expanded(&page), "Enter did not open it:\n{}", page.tree());
    assert!(page.exists(MENU));
    assert!(
        page.is_focused("[role=menuitem]"),
        "focus is on {}, not an item",
        page.focus_owner()
    );

    page.press(Key::Escape);
    assert!(
        !expanded(&page),
        "Escape did not close it:\n{}",
        page.tree()
    );
    assert!(
        page.is_focused(TRIGGER),
        "focus is on {}, not the trigger",
        page.focus_owner()
    );
}

#[test]
fn a_click_outside_closes_it_and_leaves_focus_where_it_went() {
    let mut page = mount(app);
    page.focus(TRIGGER);
    page.press(Key::Enter);
    assert!(expanded(&page));

    page.click("#elsewhere");
    assert!(
        !expanded(&page),
        "a click outside did not close it:\n{}",
        page.tree()
    );
    assert!(
        page.is_focused("#elsewhere"),
        "focus is on {}, not where the click went",
        page.focus_owner()
    );
}
