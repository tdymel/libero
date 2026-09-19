//! Todo 884: Blitz fires no `error` on an `<img>` that fails to fetch or
//! decode, so `Avatar` keeps its fallback under the picture.

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::Avatar;

/// Blue all over.
const BLUE_SQUARE: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10' width='10' height='10'><rect width='10' height='10' fill='blue'/></svg>";

fn app() -> Element {
    rsx! {
        Avatar { id: "undecodable", name: "Ada Lovelace", initials: "AL", src: "data:image/png,notapng" }
        Avatar { id: "unfetched", name: "Grace Hopper", initials: "GH", src: "/assets/missing.png" }
        Avatar { id: "loaded", name: "Radia Perlman", initials: "RP", src: BLUE_SQUARE, size: "xl" }
    }
}

fn loaded() -> Page {
    let mut page = mount(app);
    page.wait(Duration::from_millis(20));
    page
}

/// A picture that never arrives leaves the initials, not an empty circle.
#[test]
fn a_failed_picture_shows_the_initials() {
    let page = loaded();
    assert_eq!(page.text("#undecodable"), "AL");
    assert_eq!(page.text("#unfetched"), "GH");
}

/// A picture that loads covers the initials under it.
#[test]
fn a_loaded_picture_covers_the_initials() {
    let page = loaded();
    let (x, y, width, height) = page.rect("#loaded");
    let centre = page.painted_pixel((x + width / 2.0) as u32, (y + height / 2.0) as u32);
    assert_eq!(centre, "rgb(0, 0, 255)");
}
