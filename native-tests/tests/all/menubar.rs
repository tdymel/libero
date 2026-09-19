//! `Menubar`: one tab stop that roves with Left, Right, Home and End, a
//! disabled menu included; ArrowDown opens a menu, Escape closes it back onto
//! its trigger, and the pointer switches between open menus.

use dioxus::prelude::*;
use libero::components::{MenuEntry, MenuItem, Menubar, MenubarMenu};
use native_tests::{Key, Page, mount};

const MENU: &str = "[role=menu]";

fn trigger(index: usize) -> String {
    format!("[role=menubar] [data-menubar-index=\"{index}\"]")
}

fn app() -> Element {
    let item = |name: &'static str| -> MenuEntry { MenuItem::new(name).onselect(|_| {}).into() };
    rsx! {
        Menubar {
            aria_label: "Editor",
            menus: vec![
                MenubarMenu::new("File", vec![item("New"), item("Open")]),
                MenubarMenu::new("Edit", vec![item("Undo"), item("Redo")]),
                MenubarMenu::new("View", vec![item("Zoom in")]).disabled(true),
                MenubarMenu::new("Help", vec![item("About")]),
            ],
        }
    }
}

fn expanded(page: &Page, index: usize) -> bool {
    page.attr(&trigger(index), "aria-expanded")
        .is_some_and(|v| v == "true")
}

#[test]
fn the_arrows_rove_along_the_bar_and_wrap() {
    let mut page = mount(app);
    page.focus(&trigger(0));
    for (key, to) in [
        (Key::ArrowRight, 1),
        (Key::ArrowRight, 2),
        (Key::ArrowRight, 3),
        (Key::ArrowRight, 0),
        (Key::ArrowLeft, 3),
        (Key::Home, 0),
        (Key::End, 3),
    ] {
        page.press(key.clone());
        assert!(
            page.is_focused(&trigger(to)),
            "{key:?}: focus is on {}",
            page.focus_owner()
        );
    }
    assert!(!page.exists(MENU), "roving opened a menu");
}

#[test]
fn arrow_down_opens_a_menu_and_escape_hands_focus_back() {
    let mut page = mount(app);
    page.focus(&trigger(1));
    page.press(Key::ArrowDown);
    assert!(
        expanded(&page, 1),
        "ArrowDown did not open it:\n{}",
        page.tree()
    );
    assert!(
        page.is_focused("[role=menuitem]"),
        "focus is on {}",
        page.focus_owner()
    );

    page.press(Key::Escape);
    assert!(!page.exists(MENU), "Escape left it open");
    assert!(
        page.is_focused(&trigger(1)),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn right_in_an_open_menu_opens_the_next_one() {
    let mut page = mount(app);
    page.focus(&trigger(0));
    page.press(Key::ArrowDown);
    page.press(Key::ArrowRight);
    assert!(expanded(&page, 1) && !expanded(&page, 0), "{}", page.tree());
    assert_eq!(page.query_all(MENU).len(), 1);
}

/// The docs demo: a column under a centring flex box.
fn centred() -> Element {
    rsx! {
        div { style: "display: flex; justify-content: safe center; align-items: center; width: 600px; padding: 24px;",
            div { style: "display: flex; flex-direction: column; gap: 16px;", {app()} }
        }
    }
}

/// Opening a menu re-laid the bar, and Blitz kept the other labels' min-content
/// text layout: "Edit" painted per glyph in its one-line box (todo 886).
#[test]
fn opening_a_menu_leaves_the_other_labels_on_one_line() {
    let mut page = mount(centred);
    page.click(&trigger(0));
    page.advance(0.5);
    page.hover(&trigger(1));
    page.advance(0.5);
    assert!(expanded(&page, 1), "{}", page.tree());
    assert_eq!(page.wrapped_text("[role=menubar]"), Vec::<String>::new());
}

#[test]
fn hovering_another_trigger_switches_the_open_menu() {
    let mut page = mount(app);
    page.click(&trigger(0));
    assert!(
        expanded(&page, 0),
        "a click did not open it:\n{}",
        page.tree()
    );
    for index in [1, 0, 3] {
        page.hover(&trigger(index));
        assert!(
            expanded(&page, index),
            "hovering {index} left it closed:\n{}",
            page.tree()
        );
        assert_eq!(page.query_all(MENU).len(), 1);
    }
}
