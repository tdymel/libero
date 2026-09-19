//! Portaled and `position: fixed` overlays: a `Menu` lands under its anchor, a
//! `Modal` is centred in the viewport, and a pointer reaches both. The outlet
//! must not take the hits meant for the app underneath.

use dioxus::prelude::*;
use e2e::native::{Key, Page, VIEWPORT, mount};
use libero::{
    components::{Button, Menu, MenuItem, use_menu},
    hooks::{ModalScope, use_modal},
};

const TRIGGER: &str = "[aria-haspopup]";
const MENU: &str = "[role=menu]";
const DIALOG: &str = "[role=dialog]";

fn menu() -> Element {
    let menu = use_menu();
    let mut picked = use_signal(|| "none");
    rsx! {
        div { height: "200px", "Above" }
        Menu {
            state: menu,
            items: vec![
                MenuItem::new("Copy").onselect(move |_| picked.set("copy")).into(),
                MenuItem::new("Paste").onselect(move |_| picked.set("paste")).into(),
            ],
            Button { attributes: menu.a11y_attributes(), "Actions" }
        }
        span { id: "picked", "{picked}" }
    }
}

fn modal() -> Element {
    let mut clicks = use_signal(|| 0);
    let dialog = use_modal(move |_: ModalScope<()>| {
        rsx! {
            div {
                role: "dialog",
                width: "200px",
                height: "100px",
                pointer_events: "auto",
                button { id: "inside", onclick: move |_| clicks += 1, "Inside" }
            }
        }
    });
    rsx! {
        div { height: "300px", "Above" }
        Button { id: "open", onclick: move |_| { dialog.open(); }, "Open" }
        span { id: "clicks", "{clicks}" }
    }
}

fn open_modal() -> Page {
    let mut page = mount(modal);
    page.click("#open");
    assert!(
        page.exists(DIALOG),
        "the click did not open it:\n{}",
        page.tree()
    );
    page
}

fn open_menu(page: &mut Page) {
    page.click(TRIGGER);
    assert!(
        page.exists(MENU),
        "the click did not open it:\n{}",
        page.tree()
    );
}

#[test]
fn a_menu_opens_right_under_its_trigger() {
    let mut page = mount(menu);
    open_menu(&mut page);
    let (tx, ty, _, th) = page.rect(TRIGGER);
    let (mx, my, _, _) = page.rect(MENU);
    let gap = my - (ty + th);
    assert!(
        (0.0..=16.0).contains(&gap) && (mx - tx).abs() <= 16.0,
        "trigger at ({tx}, {ty}) h {th}, menu at ({mx}, {my})"
    );
}

#[test]
fn a_click_on_a_menu_item_selects_it() {
    let mut page = mount(menu);
    open_menu(&mut page);
    assert!(page.hits("[role=menuitem]"), "no hit on the item");
    page.click("[role=menuitem]");
    assert_eq!(page.text("#picked"), "copy", "{}", page.tree());
}

#[test]
fn a_modal_is_centred_in_the_viewport() {
    let page = open_modal();
    let (x, y, width, height) = page.rect(DIALOG);
    let (cx, cy) = (x + width / 2.0, y + height / 2.0);
    let (vw, vh) = (f64::from(VIEWPORT.0), f64::from(VIEWPORT.1));
    assert!(
        (cx - vw / 2.0).abs() <= 1.0 && (cy - vh / 2.0).abs() <= 1.0,
        "dialog at ({x}, {y}) {width}x{height}"
    );
}

/// The outlet follows a scrolled root back onto the viewport.
#[test]
fn a_modal_is_centred_in_a_scrolled_document() {
    fn tall() -> Element {
        rsx! {
            {modal()}
            div { height: "3000px" }
        }
    }
    let mut page = mount(tall);
    page.hover("#open");
    page.wheel("#open", 250.0);
    assert!(page.rect("#open").1 < 100.0, "the root did not scroll");
    // The harness's pointer has no page offset, so a key opens it.
    page.focus("#open");
    page.press(Key::Enter);
    page.wait(std::time::Duration::from_millis(50));
    let (x, y, width, height) = page.rect(DIALOG);
    let (cx, cy) = (x + width / 2.0, y + height / 2.0);
    let (vw, vh) = (f64::from(VIEWPORT.0), f64::from(VIEWPORT.1));
    assert!(
        (cx - vw / 2.0).abs() <= 1.0 && (cy - vh / 2.0).abs() <= 1.0,
        "dialog at ({x}, {y}) {width}x{height}"
    );
}

#[test]
fn a_click_reaches_a_button_in_a_modal() {
    let mut page = open_modal();
    assert!(page.hits("#inside"), "no hit on the button");
    page.click("#inside");
    assert_eq!(page.text("#clicks"), "1", "{}", page.tree());
}

fn menu_under_header() -> Element {
    let menu = use_menu();
    let mut picked = use_signal(|| "none");
    rsx! {
        Menu {
            state: menu,
            items: vec![MenuItem::new("Copy").onselect(move |_| picked.set("copy")).into()],
            Button { attributes: menu.a11y_attributes(), "Actions" }
        }
        div {
            position: "relative",
            z_index: "10",
            height: "400px",
            background: "white",
            span { id: "picked", "{picked}" }
        }
    }
}

#[test]
fn a_menu_takes_the_hit_over_a_raised_sibling() {
    let mut page = mount(menu_under_header);
    open_menu(&mut page);
    assert!(
        page.hits("[role=menuitem]"),
        "no hit on the item:\n{}",
        page.tree()
    );
}
