//! Any `position: sticky` box natively, found by its computed style: Blitz lays
//! one out as `relative`, so the platform moves it (todo 927).

use dioxus::prelude::*;
use e2e::native::{Page, mount};

fn top(page: &Page, selector: &str) -> f64 {
    page.rect(selector).1
}

fn scroll_page(page: &mut Page, by: f64) {
    page.hover("#content");
    page.wheel("#content", by);
}

fn plain() -> Element {
    rsx! {
        p { height: "40px", margin: "0", "Intro" }
        div { id: "bar", position: "sticky", top: "0", height: "20px", "Bar" }
        div { id: "offset", position: "sticky", top: "30px", height: "20px", "Offset" }
        div { id: "content", height: "3000px", "Content" }
    }
}

#[test]
fn a_plain_sticky_box_holds_at_its_top() {
    let mut page = mount(plain);
    assert_eq!(top(&page, "#bar"), 40.0);
    scroll_page(&mut page, 300.0);
    page.wait_for(|page| top(page, "#bar") == 0.0);
    assert_eq!(top(&page, "#bar"), 0.0, "{}", page.tree());
    assert_eq!(top(&page, "#offset"), 30.0, "{}", page.tree());
}

fn toggled() -> Element {
    let mut sticky = use_signal(|| true);
    rsx! {
        button { id: "toggle", onclick: move |_| sticky.toggle(), "Toggle" }
        div {
            id: "bar",
            position: if sticky() { "sticky" } else { "static" },
            top: "0",
            height: "20px",
            "Bar"
        }
        div { id: "content", height: "3000px", "Content" }
    }
}

/// A box that stops being sticky (a breakpoint, a class) goes back in flow.
#[test]
fn a_box_no_longer_sticky_is_moved_back() {
    let mut page = mount(toggled);
    let place = top(&page, "#bar");
    scroll_page(&mut page, 300.0);
    page.wait_for(|page| top(page, "#bar") == 0.0);
    page.click("#toggle");
    page.wait_for(|page| top(page, "#bar") == place - 300.0);
    assert_eq!(top(&page, "#bar"), place - 300.0, "{}", page.tree());
}

/// The docs sidebar from `Sm` up: sticky, as tall as its row, so it has no
/// room to move and stays put.
fn sidebar() -> Element {
    rsx! {
        div { height: "40px" }
        div { display: "flex", height: "300px",
            nav { id: "nav", position: "sticky", top: "0", height: "100%", width: "100px", "Nav" }
            div { flex: "1" }
        }
        div { id: "content", height: "3000px", "Content" }
    }
}

#[test]
fn a_box_as_tall_as_its_parent_stays_put() {
    let mut page = mount(sidebar);
    scroll_page(&mut page, 200.0);
    page.wait_for(|page| top(page, "#nav") == -160.0);
    assert_eq!(top(&page, "#nav"), -160.0, "{}", page.tree());
}
