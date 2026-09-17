//! `NavLink` with nested links: a click on the toggle beside the parent shows
//! them and flips `aria-expanded` (todo 580).

use std::time::Duration;

use dioxus::prelude::*;
use libero::components::NavLink;
use native_tests::{Page, mount};

const TOGGLE: &str = "#docs + button";

fn app() -> Element {
    rsx! {
        NavLink { id: "docs", to: "/docs",
            nested: rsx! { NavLink { id: "install", to: "/docs/install", "Install" } },
            "Docs"
        }
    }
}

/// Ends `Collapse`'s transition and its fallback timer.
fn finish(page: &mut Page) {
    page.advance(1.0);
    page.wait(Duration::from_millis(400));
}

#[test]
fn a_click_on_the_toggle_shows_the_nested_links() {
    let mut page = mount(app);
    assert_eq!(page.attr(TOGGLE, "aria-expanded").as_deref(), Some("false"));

    page.click(TOGGLE);
    finish(&mut page);
    assert_eq!(page.attr(TOGGLE, "aria-expanded").as_deref(), Some("true"));
    let (_, _, _, height) = page.rect("#install");
    assert!(
        height > 0.0,
        "the nested link is {height}px tall:\n{}",
        page.tree()
    );
}

fn active(dir: &'static str) -> Element {
    rsx! {
        div { dir, width: "240px", padding: "16px",
            NavLink { id: "here", to: "/here", active: true, "Here" }
        }
    }
}

/// `[start edge, end edge, middle]` pixels of the active link's row.
fn edges(page: &Page) -> Vec<[u8; 4]> {
    let (x, y, w, h) = page.rect("#here");
    let row = (y + h / 2.0) as u32;
    page.painted_pixels(&[
        ((x + 1.0) as u32, row),
        ((x + w - 1.0) as u32, row),
        ((x + w / 2.0) as u32, (y + 2.0) as u32),
    ])
}

/// The start bar sits at the start edge: the left one here (todo 764: in the bar colour).
#[test]
fn the_active_bar_sits_at_the_start_edge() {
    let px = edges(&mount(|| active("ltr")));
    assert_ne!(px[0], px[2], "no bar at the left edge: {px:?}");
    assert_eq!(px[1], px[2], "a bar at the right edge: {px:?}");
}

/// Under `dir="rtl"` the bar moves to the right (todo 735): stylo does not
/// match `:dir(rtl)`, so an `[dir=rtl]` ancestor selector carries it.
#[test]
fn an_rtl_active_bar_sits_at_the_right_edge() {
    let px = edges(&mount(|| active("rtl")));
    assert_ne!(px[1], px[2], "no bar at the right edge: {px:?}");
    assert_eq!(px[0], px[2], "a bar at the left edge: {px:?}");
}
