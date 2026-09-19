//! `Menubar` in Blitz's layout: opening a menu re-lays the bar. Roving,
//! ArrowDown, Escape and hover switching are e2e's shared scenarios
//! (`menubar::`).

use dioxus::prelude::*;
use libero::components::{MenuEntry, MenuItem, Menubar, MenubarMenu};
use native_tests::{Page, mount};

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
