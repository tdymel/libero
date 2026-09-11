//! `Modal` and `Drawer`: the overlay contract natively. Opening moves focus in,
//! Tab and Shift+Tab stay inside, a backdrop click closes, focus returns.

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog},
    hooks::{DrawerOptions, ModalScope, use_drawer, use_modal},
};
use native_tests::{Page, VIEWPORT, mount};

const DIALOG: &str = "[role=dialog]";

fn modal() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog {
                title: "Unsaved changes",
                size: "sm",
                Button { id: "keep", onclick: move |_| s.close(), "Keep editing" }
                Button { id: "discard", onclick: move |_| s.close(), "Discard" }
            }
        }
    });
    rsx! {
        Button { id: "before", "Before" }
        Button {
            id: "open",
            onclick: move |_| {
                prompt.open();
            },
            "Close editor"
        }
    }
}

fn drawer() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            anchor: "right".into(),
            size: "sm".into(),
            aria_label: Some("Navigation".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                Button { id: "keep", "Home" }
                Button { id: "discard", onclick: move |_| s.close(), "Close" }
            }
        },
    );
    rsx! {
        Button { id: "before", "Before" }
        Button {
            id: "open",
            onclick: move |_| {
                nav.open();
            },
            "Open navigation"
        }
    }
}

fn open(app: fn() -> Element) -> Page {
    let mut page = mount(app);
    page.click("#open");
    assert!(
        page.exists(DIALOG),
        "the click did not open it:\n{}",
        page.tree()
    );
    page
}

fn assert_inside(page: &Page, step: &str) {
    assert!(
        page.is_focused(&format!("{DIALOG} *")) || page.is_focused(DIALOG),
        "{step}: focus left the dialog for {}",
        page.focus_owner()
    );
}

fn tab_stays_inside(app: fn() -> Element) {
    let mut page = open(app);
    assert_inside(&page, "on open");
    for step in 0..4 {
        page.tab();
        assert_inside(&page, &format!("Tab {step}"));
    }
    for step in 0..4 {
        page.shift_tab();
        assert_inside(&page, &format!("Shift+Tab {step}"));
    }
}

#[test]
fn a_modal_moves_focus_in_and_traps_tab() {
    tab_stays_inside(modal);
}

#[test]
fn a_drawer_moves_focus_in_and_traps_tab() {
    tab_stays_inside(drawer);
}

#[test]
fn tab_cycles_through_every_button_in_a_modal() {
    let mut page = open(modal);
    page.focus("#keep");
    page.tab();
    assert!(
        page.is_focused("#discard"),
        "Tab went to {}",
        page.focus_owner()
    );
    page.tab();
    assert!(!page.is_focused("#discard"), "Tab did not wrap");
    assert_inside(&page, "after the wrap");
}

fn click_backdrop_closes(app: fn() -> Element) {
    let mut page = open(app);
    // The top left corner: backdrop for a centred dialog and a right drawer.
    page.click_at(4.0, 4.0);
    assert!(!page.exists(DIALOG), "the backdrop click left it open");
    assert!(
        page.is_focused("#open"),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn a_backdrop_click_closes_a_modal() {
    click_backdrop_closes(modal);
}

#[test]
fn a_backdrop_click_closes_a_drawer() {
    click_backdrop_closes(drawer);
}

#[test]
fn a_button_in_a_drawer_closes_it_and_focus_returns() {
    let mut page = open(drawer);
    page.click("#discard");
    assert!(!page.exists(DIALOG), "the close button left it open");
    assert!(
        page.is_focused("#open"),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn a_right_drawer_hugs_the_right_edge() {
    let page = open(drawer);
    let (x, y, width, height) = page.rect(DIALOG);
    let (vw, vh) = (f64::from(VIEWPORT.0), f64::from(VIEWPORT.1));
    assert!(
        (x + width - vw).abs() <= 1.0 && y.abs() <= 1.0 && (height - vh).abs() <= 1.0,
        "drawer at ({x}, {y}) {width}x{height}"
    );
}
