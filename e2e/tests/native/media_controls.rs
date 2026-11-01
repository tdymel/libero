//! The fallback `Video` and `Audio` draw where Blitz plays no media (todos 5200, 5201).
//! The browser twin is `all/media_controls.rs`.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Audio, Video};

const BOX: &str = "[data-slot=message]";
const LINK: &str = "[data-slot=message] a";
/// Blue all over.
const POSTER: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 9' width='16' height='9'><rect width='16' height='9' fill='blue'/></svg>";

fn posterised() -> Element {
    rsx! { Video { src: "https://example.com/clip.webm", label: "Clip", poster: POSTER } }
}

fn plain() -> Element {
    rsx! { Video { src: "https://example.com/clip.webm", label: "Clip" } }
}

fn audio() -> Element {
    rsx! { Audio { src: "https://example.com/clip.ogg", label: "Clip" } }
}

/// Todo 5200: an unsupported Video hides the empty black media box and draws its poster with
/// the sentence and link over the picture's foot.
#[test]
fn an_unsupported_video_draws_its_poster_with_the_link_over_it() {
    let mut page = mount(posterised);
    assert!(
        page.exists("[data-slot=message][data-poster]"),
        "no poster box\n{}",
        page.tree()
    );
    assert_eq!(page.computed("video", "display"), "none", "{}", page.tree());
    let (left, top, width, height) = page.rect("[data-slot=message] > img");
    let painted = page.wait_for(|page| {
        let (x, y) = ((left + width / 2.0) as u32, (top + height / 8.0) as u32);
        page.painted_pixel(x, y) == "rgb(0, 0, 255)"
    });
    assert!(painted, "the poster does not paint in the box");
    let (link_left, link_top, link_width, link_height) = page.rect(LINK);
    assert!(
        link_left >= left
            && link_left + link_width <= left + width
            && link_top >= top
            && link_top + link_height <= top + height,
        "the link ({link_left}, {link_top}) is not over the poster ({left}, {top}, {width}, {height})"
    );
}

#[test]
fn an_unsupported_video_without_a_poster_draws_the_plain_message() {
    let page = mount(plain);
    assert!(page.exists(BOX), "{}", page.tree());
    assert!(!page.exists("[data-poster]"), "{}", page.tree());
    assert!(!page.exists("[data-slot=message] > img"), "{}", page.tree());
    assert!(page.exists(LINK), "no download link\n{}", page.tree());
}

/// Todo 5201: the download link is an `Anchor` to the file, so the router never sees its path.
#[test]
fn the_audio_download_link_is_an_anchor_to_the_file() {
    let page = mount(audio);
    assert_eq!(
        page.attr(LINK, "href").as_deref(),
        Some("https://example.com/clip.ogg"),
        "{}",
        page.tree()
    );
    let state = page.attr(LINK, "data-state").unwrap_or_default();
    assert!(
        state.contains("underline-"),
        "not an Anchor: {state:?}\n{}",
        page.tree()
    );
}
