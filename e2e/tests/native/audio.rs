//! The download link `Audio` falls back to where Blitz plays no media.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::Audio;

const FALLBACK: &str = "[data-slot=message] a";

fn script() -> Element {
    rsx! { Audio { src: "javascript:alert(1)", label: "Clip" } }
}

fn ordinary() -> Element {
    rsx! { Audio { src: "https://example.com/clip.ogg", label: "Clip" } }
}

/// Todo 3626: a `javascript:` source leaves the download link without an `href`.
#[test]
fn a_script_source_leaves_the_download_link_inert() {
    let page = mount(script);
    assert!(page.exists(FALLBACK), "no fallback link\n{}", page.tree());
    assert_eq!(page.attr(FALLBACK, "href"), None, "{}", page.tree());
}

#[test]
fn an_ordinary_source_is_the_download_link() {
    let page = mount(ordinary);
    assert_eq!(
        page.attr(FALLBACK, "href").as_deref(),
        Some("https://example.com/clip.ogg"),
        "{}",
        page.tree()
    );
}
