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
    let bottom = |page: &Page, selector: &str| {
        let (_, y, _, height) = page.rect(selector);
        y + height
    };
    page.wait_for(|page| top(page, "#first") < -100.0);
    assert_eq!(
        bottom(&page, "#banner"),
        bottom(&page, "#first"),
        "{}",
        page.tree()
    );
}
