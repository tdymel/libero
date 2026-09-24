//! A link item in a `Menu` with a mask mark before it (the docs' TLDR menu), in Blitz.

use dioxus::prelude::*;
use e2e::native::{Key, Page, mount};
use libero::components::{Button, Icon, Menu, MenuEntry, MenuItem, use_menu};

/// Only the left half is inked.
const HALF: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><rect width='12' height='24'/></svg>";

fn app() -> Element {
    let menu = use_menu();
    let items: Vec<MenuEntry> = vec![
        MenuItem::new("Claude")
            .href("https://claude.ai/new?q=x")
            .leading(rsx! { Icon { src: HALF, variant: "standard", color: "red", size: "xl" } })
            .into(),
    ];
    rsx! {
        Menu { state: menu, items,
            Button { attributes: menu.a11y_attributes(), "TLDR" }
        }
    }
}

fn opened() -> Page {
    let mut page = mount(app);
    page.click("[aria-haspopup=menu]");
    page.advance(0.5);
    page
}

#[test]
fn a_link_item_is_an_anchor_with_its_target() {
    let page = opened();
    let link = "[role=menu] a[role=menuitem]";
    assert_eq!(
        page.attr(link, "href").as_deref(),
        Some("https://claude.ai/new?q=x"),
        "{}",
        page.tree()
    );
    assert_eq!(page.attr(link, "target").as_deref(), Some("_blank"));
}

const URL: &str = "https://claude.ai/new?q=x";

/// dioxus-native opens what the document navigates to in the browser.
#[test]
fn a_click_on_a_link_item_opens_its_url() {
    let mut page = opened();
    assert!(page.navigations().is_empty());
    page.click("[role=menu] a[role=menuitem]");
    assert_eq!(page.navigations(), [URL], "{}", page.tree());
}

#[test]
fn enter_on_a_link_item_opens_its_url() {
    let mut page = opened();
    assert!(page.navigations().is_empty());
    page.focus("[role=menu] a[role=menuitem]");
    page.press(Key::Enter);
    assert_eq!(page.navigations(), [URL], "{}", page.tree());
}

/// The mask keeps the inked half in the icon's color and leaves the rest of the
/// row as it was: the docs' marks rely on it.
#[test]
fn a_mask_mark_paints_its_shape_in_the_icon_color() {
    let page = opened();
    let (x, y, width, height) = page.rect("[data-slot=leading] span span");
    assert!(width > 0.0, "the mask has no box");
    let at = |fx: f64| page.painted_pixel((x + width * fx) as u32, (y + height / 2.0) as u32);
    let (inked, empty) = (at(0.25), at(0.75));
    assert_ne!(inked, empty, "a solid block: {inked}");
    assert!(inked.starts_with("rgb(2"), "not the icon's red: {inked}");
}
