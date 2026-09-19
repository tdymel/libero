//! `NavLink`'s painted active bar. The nested links' toggle is e2e's shared
//! scenario (`nav_link::`).

use dioxus::prelude::*;
use libero::components::NavLink;
use native_tests::{Page, mount};

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
