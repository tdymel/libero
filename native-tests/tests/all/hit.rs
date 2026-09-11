//! Blitz hit testing of inline content: a box laid out inline in a block with
//! padding takes no hit, wherever the pointer lands. A block wrapper without
//! padding, or a flex container, avoids it. And the client rect of a
//! translated box.

use dioxus::prelude::*;
use native_tests::mount;

fn app() -> Element {
    rsx! {
        div { padding_left: "200px", button { id: "padded-left", "A" } }
        div { padding_top: "20px", button { id: "padded-top", "B" } }
        div { margin_left: "200px", button { id: "margin", "C" } }
        div { padding_left: "200px", div { button { id: "wrapped", "D" } } }
        div { padding_left: "200px", display: "flex", button { id: "flex", "E" } }
        div { transform: "translate(40px, 40px)", display: "flex", button { id: "moved", "F" } }
    }
}

/// Hit testing follows the translate; the client rect does not, so a click at
/// the reported centre misses (a `Notifications` stack's close button).
#[test]
#[ignore = "needs Blitz: getBoundingClientRect leaves out a transform"]
fn a_translated_box_reports_where_it_is_drawn() {
    let page = mount(app);
    let (x, _, _, _) = page.rect("#moved");
    assert_eq!(x, 40.0, "#moved reports x {x}");
    assert!(page.hits("#moved"));
}

#[test]
fn inline_content_without_padding_around_it_takes_its_hits() {
    let page = mount(app);
    for id in ["#margin", "#wrapped", "#flex"] {
        assert!(page.hits(id), "{id} at {:?} takes no hit", page.rect(id));
    }
}

#[test]
#[ignore = "needs Blitz: inline content in a padded block takes no hit"]
fn inline_content_in_a_padded_block_takes_its_hits() {
    let page = mount(app);
    for id in ["#padded-left", "#padded-top"] {
        assert!(page.hits(id), "{id} at {:?} takes no hit", page.rect(id));
    }
}
