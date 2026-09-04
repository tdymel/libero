//! `Menu`'s markup: the trigger's wiring, the roving `tabindex`, one index
//! sequence across groups, and a submenu that is announced but not drawn until
//! it opens. Focus movement needs a real renderer - `ElementApi` is
//! `Unsupported` here - and is verified in the browser instead.

use std::cell::{Cell, RefCell};

use crate::common::{attributes_of, body, render};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, Menu, MenuEntry, MenuItem, use_menu},
};

thread_local! {
    static OPEN: Cell<bool> = const { Cell::new(false) };
    static DISABLED: Cell<bool> = const { Cell::new(false) };
    /// The state's id, read while the dom is alive - its signal is gone once
    /// `render` returns.
    static ID: RefCell<String> = const { RefCell::new(String::new()) };
}

fn items() -> Vec<MenuEntry> {
    vec![
        MenuEntry::Group {
            label: "Edit".into(),
            items: vec![
                MenuItem::new("Copy").on_select(|_| {}).into(),
                MenuItem::new("Paste")
                    .disabled(true)
                    .on_select(|_| {})
                    .into(),
            ],
        },
        MenuEntry::Separator,
        MenuItem::new("Export")
            .submenu(vec![MenuItem::new("PNG").on_select(|_| {}).into()])
            .into(),
        MenuItem::new("Delete").on_select(|_| {}).into(),
    ]
}

fn app() -> Element {
    let menu = use_menu();
    use_hook(|| {
        if OPEN.get() {
            menu.open();
        }
    });
    ID.with(|id| *id.borrow_mut() = menu.id());

    rsx! {
        LiberoProvider {
            Menu {
                state: menu,
                items: items(),
                disabled: DISABLED.get(),
                Button { attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

fn rendered(open: bool, disabled: bool) -> String {
    OPEN.set(open);
    DISABLED.set(disabled);
    body(&render(app))
}

fn id() -> String {
    ID.with(|id| id.borrow().clone())
}

/// Every opening tag carrying `needle`, in document order.
fn tags_with<'a>(html: &'a str, needle: &str) -> Vec<&'a str> {
    html.match_indices('<')
        .map(|(at, _)| &html[at..at + html[at..].find('>').unwrap()])
        .filter(|tag| tag.contains(needle))
        .collect()
}

#[test]
fn a_closed_menu_wires_its_trigger_and_draws_no_menu() {
    let html = rendered(false, false);
    let id = id();
    let trigger = attributes_of(&html, "button");

    assert_eq!(trigger["id"], format!("{id}-trigger"));
    assert_eq!(trigger["aria-haspopup"], "menu");
    assert_eq!(trigger["aria-expanded"], "false");
    // It would name an element that is not in the document.
    assert!(!trigger.contains_key("aria-controls"));
    assert!(!html.contains(r#"role="menu""#));
}

#[test]
fn an_open_menu_is_labelled_by_its_trigger_and_controlled_by_it() {
    let html = rendered(true, false);
    let id = id();
    let trigger = attributes_of(&html, "button");
    assert_eq!(trigger["aria-expanded"], "true");
    assert_eq!(trigger["aria-controls"], format!("{id}-menu"));

    let menu = tags_with(&html, r#"role="menu""#);
    assert_eq!(menu.len(), 1, "one menu box, and no submenu yet");
    assert!(menu[0].contains(&format!(r#"id="{id}-menu""#)));
    assert!(menu[0].contains(&format!(r#"aria-labelledby="{id}-trigger""#)));
    assert!(menu[0].contains(r#"tabindex="-1""#));
}

#[test]
fn exactly_one_item_is_tabbable_and_indices_run_through_groups() {
    let html = rendered(true, false);
    let items = tags_with(&html, r#"role="menuitem""#);
    assert_eq!(items.len(), 4, "the submenu's own item is not drawn");

    for (index, item) in items.iter().enumerate() {
        assert!(
            item.contains(&format!(r#"data-menu-index="{index}""#)),
            "{item}"
        );
        let tabindex = if index == 0 { "0" } else { "-1" };
        assert!(
            item.contains(&format!(r#"tabindex="{tabindex}""#)),
            "{item}"
        );
    }
}

#[test]
fn a_group_is_named_by_its_label() {
    let html = rendered(true, false);
    let id = id();
    let group = tags_with(&html, r#"role="group""#);
    assert_eq!(group.len(), 1);
    let label_id = format!("{id}-menu-group-0");
    assert!(group[0].contains(&format!(r#"aria-labelledby="{label_id}""#)));
    assert!(html.contains(&format!(r#"id="{label_id}""#)));
    assert!(html.contains(">Edit<"));
}

#[test]
fn a_separator_and_a_disabled_item_say_so() {
    let html = rendered(true, false);
    assert_eq!(tags_with(&html, r#"role="separator""#).len(), 1);

    let paste = tags_with(&html, r#"data-menu-index="1""#);
    assert!(paste[0].contains(r#"aria-disabled="true""#));
    // Not the `disabled` attribute: that would take it out of the arrow order.
    assert!(!paste[0].contains(" disabled"));
}

#[test]
fn a_submenu_item_announces_a_closed_menu() {
    let html = rendered(true, false);
    let export = tags_with(&html, r#"data-menu-index="2""#);
    assert!(export[0].contains(r#"aria-haspopup="menu""#));
    assert!(export[0].contains(r#"aria-expanded="false""#));
    assert!(!export[0].contains("aria-controls"));

    let delete = tags_with(&html, r#"data-menu-index="3""#);
    assert!(!delete[0].contains("aria-haspopup"));
    assert!(!delete[0].contains("aria-expanded"));
}

#[test]
fn a_disabled_menu_is_closed_not_hidden() {
    let html = rendered(true, true);
    assert!(!html.contains(r#"role="menu""#));
    // The trigger must not announce a menu nobody can see.
    let trigger = attributes_of(&html, "button");
    assert_eq!(trigger["aria-expanded"], "false");
    assert!(!trigger.contains_key("aria-controls"));
}
