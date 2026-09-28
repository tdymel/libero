//! A docked `BottomNavigation` natively (todo 1356): sticky holds through the
//! platform's sticky shim, `env()` falls back to 0px, `fixed` is not docked yet.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{BottomNavigation, BottomNavigationItem};

fn items() -> Element {
    rsx! {
        BottomNavigationItem { selected: true, onclick: |_| {}, icon: rsx! { "H" }, "Home" }
        BottomNavigationItem { onclick: |_| {}, icon: rsx! { "S" }, "Search" }
    }
}

fn sticky() -> Element {
    rsx! {
        div { id: "content", height: "3000px", padding_bottom: "var(--lsx-bottom-navigation-height)", "Content" }
        BottomNavigation { id: "bar", "aria-label": "Main", position: "sticky", {items()} }
    }
}

fn fixed() -> Element {
    rsx! {
        div { id: "content", height: "3000px", "Content" }
        BottomNavigation { id: "bar", "aria-label": "Main", position: "fixed", {items()} }
    }
}

fn bottom(page: &Page, selector: &str) -> f64 {
    let (_, y, _, height) = page.rect(selector);
    y + height
}

/// The bar holds at the window's bottom edge through a scroll, its safe-area
/// padding resolves to 0px, and the page is padded by its measured height.
#[test]
fn a_sticky_bar_holds_at_the_bottom_and_publishes_its_height() {
    let mut page = mount(sticky);
    let window = page.window_size().1 as f64;
    page.wait_for(|page| bottom(page, "#bar") == window);
    assert_eq!(bottom(&page, "#bar"), window, "{}", page.tree());
    assert_eq!(page.computed("#bar", "padding-bottom"), "0px");
    let height = page.rect("#bar").3;
    page.wait_for(|page| page.computed("#content", "padding-bottom") == format!("{height}px"));
    assert_eq!(
        page.computed("#content", "padding-bottom"),
        format!("{height}px")
    );

    page.hover("#content");
    page.wheel("#content", 500.0);
    page.wait_for(|page| page.rect("#content").1 < -100.0 && bottom(page, "#bar") == window);
    assert_eq!(bottom(&page, "#bar"), window, "{}", page.tree());
}

/// Blitz lays `fixed` out as `absolute`: the bar sits at the document's end.
#[test]
#[ignore = "Blitz lays position: fixed out as absolute; a platform shim is a filed todo"]
fn a_fixed_bar_docks_to_the_window() {
    let mut page = mount(fixed);
    let window = page.window_size().1 as f64;
    page.wait_for(|page| bottom(page, "#bar") == window);
    assert_eq!(bottom(&page, "#bar"), window, "{}", page.tree());
}
