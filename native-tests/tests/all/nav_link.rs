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
