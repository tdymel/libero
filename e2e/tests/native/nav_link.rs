//! `NavLink`'s painted active bar. The nested links' toggle is e2e's shared
//! scenario (`nav_link::`).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::NavLink;

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

fn closed_panel() -> Element {
    rsx! {
        div { width: "240px", padding: "16px",
            NavLink { id: "docs", to: "/docs", nested: rsx! {
                NavLink { id: "install", to: "/install", "Install" }
            }, "Docs" }
        }
    }
}

/// Blitz's Tab walk enters a link under `visibility: hidden`, so a closed disclosure's
/// nested links stay in the tab order (every `Collapse`).
#[test]
#[ignore = "Blitz's Tab traversal does not skip visibility: hidden; a filed todo"]
fn a_closed_panels_links_are_out_of_the_tab_order() {
    let mut page = mount(closed_panel);
    page.focus("#docs + button");
    page.tab();
    assert!(
        !page.is_focused("#install"),
        "Tab entered the closed panel: {}",
        page.focus_owner()
    );
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
