//! A sticky `Header` natively: Blitz lays `position: sticky` out as `relative`,
//! so the platform moves it to its scroller's top edge (todo 907).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{components::Header, sx::sx};

fn page_banner() -> Element {
    rsx! {
        p { id: "intro", height: "40px", margin: "0", "Intro" }
        Header { id: "banner", sx: sx().background("rgb(255, 0, 0)"), "Libero" }
        div { id: "content", height: "3000px", "Content" }
    }
}

fn top(page: &Page, selector: &str) -> f64 {
    page.rect(selector).1
}

#[test]
fn a_page_scroll_holds_the_banner_at_the_top() {
    let mut page = mount(page_banner);
    assert_eq!(top(&page, "#banner"), 40.0);
    page.hover("#content");
    page.wheel("#content", 300.0);
    page.wait_for(|page| top(page, "#banner") == 0.0);
    assert_eq!(top(&page, "#banner"), 0.0, "{}", page.tree());
    let (_, _, width, height) = page.rect("#banner");
    let (x, y) = ((width / 2.0) as u32, (height / 2.0) as u32);
    assert_eq!(page.painted_pixel(x, y), "rgb(255, 0, 0)");

    page.wheel("#content", -300.0);
    page.wait_for(|page| top(page, "#banner") == 40.0);
    assert_eq!(top(&page, "#banner"), 40.0, "it stayed stuck");
}

fn framed() -> Element {
    rsx! {
        div { height: "60px" }
        div { id: "frame", height: "200px", overflow: "auto",
            Header { id: "banner", "Libero" }
            div { id: "content", height: "1000px", "Content" }
        }
    }
}

#[test]
fn a_scrolling_frame_holds_its_banner_at_its_top() {
    let mut page = mount(framed);
    page.hover("#content");
    page.wheel("#content", 150.0);
    page.wait_for(|page| top(page, "#banner") == top(page, "#frame"));
    assert_eq!(
        top(&page, "#banner"),
        top(&page, "#frame"),
        "{}",
        page.tree()
    );
}

fn published() -> Element {
    rsx! {
        Header { id: "banner", publish_height: true, "Libero" }
        div { id: "box", height: "200px", overflow_y: "auto",
            for index in 0..10 {
                a { key: "{index}", id: "item-{index}", href: "#", display: "block", height: "40px", "Item {index}" }
            }
        }
        for index in 0..40 {
            a { key: "{index}", id: "link-{index}", href: "#", display: "block", height: "40px", "Link {index}" }
        }
    }
}

fn bottom(page: &Page, selector: &str) -> f64 {
    let (_, y, _, height) = page.rect(selector);
    y + height
}

/// Todo 1521: a published banner's padding reaches Blitz's focus scroll, so Shift+Tab
/// onto a link under the stuck banner scrolls it clear.
#[test]
fn a_shift_tab_under_a_published_banner_scrolls_clear_of_it() {
    let mut page = mount(published);
    page.hover("#link-20");
    page.wheel("#link-20", 1000.0);
    page.wait_for(|page| top(page, "#banner") == 0.0);
    // The link just above the banner's bottom edge, under it.
    let link = |index: usize| format!("#link-{index}");
    let index = (0..39)
        .find(|&index| {
            let y = top(&page, &link(index));
            y >= 0.0 && y < bottom(&page, "#banner")
        })
        .expect("a link under the banner");
    let under = link(index);
    page.focus(&link(index + 1));
    page.shift_tab();
    page.settle();
    assert!(page.is_focused(&under), "{}", page.focus_owner());
    assert!(
        top(&page, &under) >= bottom(&page, "#banner") - 0.5,
        "{under} at {} under the banner's bottom {}",
        top(&page, &under),
        bottom(&page, "#banner")
    );
}

/// Todo 1521: the root's padding var is inherited, not a nested scroller's own: a shown
/// item at its box's top edge stays put.
#[test]
fn a_nested_scroller_keeps_none_of_the_banners_padding() {
    let mut page = mount(published);
    page.hover("#box");
    page.wheel("#box", 200.0);
    page.wait_for(|page| page.scroll_top("#box") == 200.0);
    page.focus("#item-6");
    page.shift_tab();
    page.settle();
    assert!(page.is_focused("#item-5"), "{}", page.focus_owner());
    assert_eq!(
        page.scroll_top("#box"),
        200.0,
        "the box took the banner's padding"
    );
}

fn sections() -> Element {
    rsx! {
        section { id: "first", height: "300px",
            Header { id: "banner", "First" }
        }
        div { id: "content", height: "3000px", "Content" }
    }
}

/// The banner sticks inside its parent only: that leaving takes it along.
#[test]
fn a_banner_leaves_with_its_parent() {
    let mut page = mount(sections);
    page.hover("#content");
    page.wheel("#content", 500.0);
    // The scroll lands first, the sticky move after it: wait for both.
    page.wait_for(|page| {
        top(page, "#first") < -100.0 && bottom(page, "#banner") == bottom(page, "#first")
    });
    assert_eq!(
        bottom(&page, "#banner"),
        bottom(&page, "#first"),
        "{}",
        page.tree()
    );
}
