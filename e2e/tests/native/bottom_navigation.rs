//! A docked `BottomNavigation` natively (todo 1356): sticky and fixed hold
//! through the platform's sticky shim, `env()` falls back to 0px.

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

fn crowded() -> Element {
    rsx! {
        div { width: "320px",
            BottomNavigation { id: "bar", "aria-label": "Main",
                BottomNavigationItem { id: "long", onclick: |_| {}, icon: rsx! { "N" },
                    "Notifications and messages"
                }
                for label in ["Home", "Search", "Inbox", "Profile"] {
                    BottomNavigationItem { key: "{label}", onclick: |_| {}, icon: rsx! { "i" }, "{label}" }
                }
            }
        }
    }
}

fn pane() -> Element {
    rsx! {
        div {
            id: "pane",
            width: "320px",
            height: "300px",
            overflow_y: "auto",
            style: "--lsx-scroll-padding-bottom: var(--lsx-bottom-navigation-height);",
            for index in 0..20 {
                a { key: "{index}", id: "row-{index}", href: "#row-{index}", display: "block", padding: "12px", "Row {index}" }
            }
            BottomNavigation { id: "sticky", "aria-label": "Main", position: "sticky",
                BottomNavigationItem { selected: true, onclick: |_| {}, icon: rsx! { "H" }, "Home" }
                BottomNavigationItem { onclick: |_| {}, icon: rsx! { "S" }, "Search" }
            }
        }
    }
}

/// A link tabbed to in the pane a sticky bar closes ends up clear of the bar.
#[test]
fn a_tabbed_link_in_a_pane_is_clear_of_a_sticky_bar() {
    let mut page = mount(pane);
    page.focus("#row-0");
    for _ in 0..10 {
        page.tab();
    }
    page.settle();
    let (_, y, _, height) = page.rect("#row-10");
    let bar = page.rect("#sticky").1;
    assert!(page.is_focused("#row-10"), "{}", page.focus_owner());
    assert!(
        y + height <= bar + 1.0,
        "row bottom {} under bar top {bar}",
        y + height
    );
}

/// Blitz has no `-webkit-line-clamp`: a height cap stops a long label at two lines, without the ellipsis.
#[test]
fn a_long_label_stops_at_two_lines() {
    let page = mount(crowded);
    let (_, _, _, height) = page.rect("#long [data-slot=label]");
    let line = 12.0 * 1.25;
    assert!(height <= 2.0 * line + 1.0, "the label is {height}px tall");
}

/// Blitz lays `fixed` out as `absolute`, at the document's end: the shim docks
/// it to the window, through a scroll too (todo 1409).
#[test]
fn a_fixed_bar_docks_to_the_window() {
    let mut page = mount(fixed);
    let window = page.window_size().1 as f64;
    page.wait_for(|page| bottom(page, "#bar") == window);
    assert_eq!(bottom(&page, "#bar"), window, "{}", page.tree());

    page.hover("#content");
    page.wheel("#content", 500.0);
    page.wait_for(|page| page.rect("#content").1 < -100.0 && bottom(page, "#bar") == window);
    assert_eq!(bottom(&page, "#bar"), window, "{}", page.tree());
}
