//! Any `position: sticky` or `fixed` box natively, found by its computed style:
//! Blitz lays one out as `relative` or `absolute`, so the platform moves it
//! (todos 927, 1409).

use dioxus::prelude::*;
use e2e::native::{Page, VIEWPORT, mount};

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

fn left(page: &Page, selector: &str) -> f64 {
    page.rect(selector).0
}

/// A 200px-square scroller around `content`, for the other edges (933).
fn scroller(content: Element, row: bool) -> Element {
    rsx! {
        div {
            id: "scroller",
            display: if row { "flex" } else { "block" },
            width: "200px",
            height: "200px",
            overflow: "auto",
            {content}
        }
    }
}

fn relative_tops() -> Element {
    scroller(
        rsx! {
            div { height: "40px" }
            div { id: "percent", position: "sticky", top: "10%", height: "20px", "Percent" }
            div { id: "em", position: "sticky", top: "2em", font_size: "10px", height: "20px", "Em" }
            div { id: "content", height: "1000px", "Content" }
        },
        false,
    )
}

/// `%` refers to the scroller's window (200px), `em` to the box's font size.
#[test]
fn a_percent_or_em_top_holds_there() {
    let mut page = mount(relative_tops);
    let origin = top(&page, "#scroller");
    // The platform's sticky move lands after a timer, at mount too.
    page.wait_for(|page| top(page, "#percent") - origin == 40.0);
    assert_eq!(top(&page, "#percent") - origin, 40.0, "{}", page.tree());
    page.hover("#content");
    page.wheel("#content", 300.0);
    page.wait_for(|page| {
        top(page, "#percent") - origin == 20.0 && top(page, "#em") - origin == 20.0
    });
    assert_eq!(top(&page, "#percent") - origin, 20.0, "{}", page.tree());
    assert_eq!(top(&page, "#em") - origin, 20.0, "{}", page.tree());
}

fn footer() -> Element {
    scroller(
        rsx! {
            div { id: "content", height: "1000px", "Content" }
            div { id: "footer", position: "sticky", bottom: "10px", height: "20px", "Footer" }
            div { height: "100px" }
        },
        false,
    )
}

/// A `bottom` box below the window is held at its bottom edge, and goes back
/// to its place once scrolled to.
#[test]
fn a_bottom_box_holds_at_the_bottom_until_reached() {
    let mut page = mount(footer);
    let origin = top(&page, "#scroller");
    page.wait_for(|page| top(page, "#footer") - origin == 170.0);
    assert_eq!(top(&page, "#footer") - origin, 170.0, "{}", page.tree());
    page.hover("#content");
    page.wheel("#content", 2000.0);
    // Scrolled to the end (920): in flow at 1000.
    page.wait_for(|page| top(page, "#footer") - origin == 80.0);
    assert_eq!(top(&page, "#footer") - origin, 80.0, "{}", page.tree());
}

fn page_footer() -> Element {
    rsx! {
        div { id: "content", height: "3000px", "Content" }
        div { id: "footer", position: "sticky", bottom: "0", height: "20px", "Footer" }
    }
}

/// Without a scroller the viewport holds it.
#[test]
fn a_bottom_box_holds_at_the_viewport_bottom() {
    let mut page = mount(page_footer);
    let bottom = f64::from(VIEWPORT.1) - 20.0;
    page.wait_for(|page| top(page, "#footer") == bottom);
    assert_eq!(top(&page, "#footer"), bottom, "{}", page.tree());
}

/// Blitz reports no window resize: the window size poll re-places it (934).
#[test]
fn a_bottom_box_follows_a_window_resize() {
    let mut page = mount(page_footer);
    let bottom = f64::from(VIEWPORT.1) - 20.0;
    page.wait_for(|page| top(page, "#footer") == bottom);
    assert_eq!(top(&page, "#footer"), bottom, "{}", page.tree());
    page.resize(800, 400);
    page.wait_for(|page| top(page, "#footer") == 380.0);
    assert_eq!(top(&page, "#footer"), 380.0, "{}", page.tree());
}

fn row() -> Element {
    scroller(
        rsx! {
            div { id: "start", position: "sticky", left: "0", width: "20px", flex_shrink: "0", "S" }
            div { id: "content", width: "1000px", flex_shrink: "0", "Content" }
            div { id: "end", position: "sticky", right: "10%", width: "20px", flex_shrink: "0", "E" }
        },
        true,
    )
}

/// `left` and `right` hold a box sideways, the same way.
#[test]
fn left_and_right_boxes_hold_sideways() {
    let mut page = mount(row);
    let origin = left(&page, "#scroller");
    page.wait_for(|page| left(page, "#end") - origin == 160.0);
    assert_eq!(left(&page, "#end") - origin, 160.0, "{}", page.tree());
    page.hover("#content");
    page.wheel_x("#content", 300.0);
    page.wait_for(|page| left(page, "#start") - origin == 0.0);
    assert_eq!(left(&page, "#start") - origin, 0.0, "{}", page.tree());
    assert_eq!(left(&page, "#end") - origin, 160.0, "{}", page.tree());
}

fn fixed() -> Element {
    rsx! {
        div { height: "100px" }
        div { padding: "30px", position: "relative",
            div { id: "corner", position: "fixed", top: "10px", left: "20px", width: "40px", height: "20px", "Corner" }
            div { id: "end", position: "fixed", bottom: "10%", right: "0", width: "40px", height: "20px", "End" }
            div { id: "static", position: "fixed", width: "40px", height: "20px", "Static" }
        }
        div { transform: "translateX(5px)",
            div { id: "held", position: "fixed", top: "0", height: "20px", "Held" }
        }
        div { id: "content", height: "3000px", "Content" }
    }
}

/// Blitz places `fixed` against the parent box: the shim moves it to its insets
/// in the window (`%` of the window), and no scroll moves it (todo 1409).
#[test]
fn a_fixed_box_holds_at_its_insets_in_the_window() {
    let mut page = mount(fixed);
    let (width, height) = (f64::from(VIEWPORT.0), f64::from(VIEWPORT.1));
    let placed = |page: &Page| {
        let (corner, end) = (page.rect("#corner"), page.rect("#end"));
        (corner.0, corner.1, end.0, end.1)
    };
    let expected = (20.0, 10.0, width - 40.0, (0.9 * height - 20.0).round());
    page.wait_for(|page| placed(page) == expected);
    assert_eq!(placed(&page), expected, "{}", page.tree());
    let stays = top(&page, "#static");

    scroll_page(&mut page, 300.0);
    page.wait_for(|page| top(page, "#content") < -100.0 && placed(page) == expected);
    assert_eq!(placed(&page), expected, "{}", page.tree());
    assert_eq!(top(&page, "#static"), stays, "{}", page.tree());
}

/// A transformed ancestor holds a `fixed` box, as on the web: it scrolls along.
#[test]
fn a_fixed_box_in_a_transformed_box_stays_in_it() {
    let mut page = mount(fixed);
    let place = top(&page, "#held");
    scroll_page(&mut page, 300.0);
    page.wait_for(|page| top(page, "#held") == place - 300.0);
    assert_eq!(top(&page, "#held"), place - 300.0, "{}", page.tree());
}
