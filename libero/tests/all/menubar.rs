//! `Menubar`'s markup: the bar's role and name, one tab stop across the row,
//! and each trigger wired to its own menu. Focus, the arrow keys and switching
//! need a real renderer - `ElementApi` is `Unsupported` here - and are
//! verified in the browser instead.

use std::cell::Cell;

use crate::common::{attributes_of, body, render};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{MenuItem, Menubar, MenubarMenu},
};

thread_local! {
    static FIRST_DISABLED: Cell<bool> = const { Cell::new(false) };
}

fn app() -> Element {
    let item = |label: &str| MenuItem::new(label).on_select(|_| {}).into();
    rsx! {
        LiberoProvider {
            Menubar {
                aria_label: "Main",
                menus: vec![
                    MenubarMenu::new("File", vec![item("New"), item("Open")])
                        .disabled(FIRST_DISABLED.get()),
                    MenubarMenu::new("Edit", vec![item("Copy")]),
                    MenubarMenu::new("View", vec![item("Zoom")]).disabled(true),
                ],
            }
        }
    }
}

fn rendered(first_disabled: bool) -> String {
    FIRST_DISABLED.set(first_disabled);
    body(&render(app))
}

/// Every opening tag carrying `needle`, in document order.
fn tags_with<'a>(html: &'a str, needle: &str) -> Vec<&'a str> {
    html.match_indices('<')
        .map(|(at, _)| &html[at..at + html[at..].find('>').unwrap()])
        .filter(|tag| tag.contains(needle))
        .collect()
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    Some(&tag[start..start + tag[start..].find('"')?])
}

#[test]
fn the_bar_is_a_named_horizontal_menubar() {
    let html = rendered(false);
    let bar = tags_with(&html, r#"role="menubar""#);
    assert_eq!(bar.len(), 1);
    assert!(bar[0].contains(r#"aria-label="Main""#));
    assert!(bar[0].contains(r#"aria-orientation="horizontal""#));
}

#[test]
fn every_trigger_is_a_menuitem_wired_to_its_own_closed_menu() {
    let html = rendered(false);
    let triggers = tags_with(&html, "data-menubar-index");
    assert_eq!(triggers.len(), 3);

    let mut ids = Vec::new();
    for (index, trigger) in triggers.iter().enumerate() {
        assert!(trigger.contains(r#"role="menuitem""#), "{trigger}");
        assert!(trigger.contains(r#"aria-haspopup="menu""#), "{trigger}");
        assert!(trigger.contains(r#"aria-expanded="false""#), "{trigger}");
        assert!(
            trigger.contains(&format!(r#"data-menubar-index="{index}""#)),
            "{trigger}"
        );
        ids.push(attribute(trigger, "id").expect("a trigger id").to_string());
    }
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3, "each menu has its own state and ids");
    assert!(
        !html.contains(r#"role="menu""#),
        "nothing open, nothing drawn"
    );
}

#[test]
fn the_bar_is_one_tab_stop_on_the_first_enabled_trigger() {
    let tabbable = |html: &str| -> Vec<usize> {
        tags_with(html, "data-menubar-index")
            .iter()
            .enumerate()
            .filter(|(_, tag)| tag.contains(r#"tabindex="0""#))
            .map(|(index, _)| index)
            .collect()
    };
    assert_eq!(tabbable(&rendered(false)), vec![0]);
    assert_eq!(
        tabbable(&rendered(true)),
        vec![1],
        "a disabled one is skipped"
    );
}

#[test]
fn a_disabled_trigger_says_so_and_stays_in_the_row() {
    let html = rendered(false);
    let view = tags_with(&html, r#"data-menubar-index="2""#);
    assert!(view[0].contains(r#"aria-disabled="true""#));
    // Not the `disabled` attribute: the element stays where the row has it.
    assert!(!view[0].contains(" disabled"));
    let file = attributes_of(&html, "button");
    assert!(!file.contains_key("aria-disabled"));
}
