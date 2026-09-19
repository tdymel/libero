//! Todo 935: Blitz fires no `error` on an `<img>` that fails to fetch or
//! decode, so `Image` paints its `fallback_src` under the picture.

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::Image;

/// Blue all over.
const BLUE_SQUARE: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10' width='10' height='10'><rect width='10' height='10' fill='blue'/></svg>";
/// Red all over.
const RED_SQUARE: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10' width='10' height='10'><rect width='10' height='10' fill='red'/></svg>";

fn app() -> Element {
    rsx! {
        for (id, src) in [
            ("undecodable", "data:image/png,notapng"),
            ("unfetched", "/assets/missing.png"),
            ("unfetched-svg", "/assets/missing.svg"),
            ("loaded", BLUE_SQUARE),
        ] {
            div { key: "{id}", id, style: "width: 40px; height: 40px;",
                Image { src, fallback_src: RED_SQUARE.to_string(), alt: id }
            }
        }
    }
}

fn centre(page: &Page, selector: &str) -> String {
    let (x, y, width, height) = page.rect(selector);
    page.painted_pixel((x + width / 2.0) as u32, (y + height / 2.0) as u32)
}

/// A picture that never arrives shows the fallback; one that loads covers it.
#[test]
fn a_failed_picture_shows_the_fallback() {
    let mut page = mount(app);
    page.wait(Duration::from_millis(20));
    for id in ["#undecodable", "#unfetched", "#unfetched-svg"] {
        assert_eq!(centre(&page, id), "rgb(255, 0, 0)", "{id}");
    }
    assert_eq!(centre(&page, "#loaded"), "rgb(0, 0, 255)");
}
