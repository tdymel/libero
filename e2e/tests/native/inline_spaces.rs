//! Inline components in running text natively. Blitz (parley 0.11) trims the
//! whitespace an inline element starts or ends with (todo 931).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Anchor, Badge, Code, Kbd, Mark, Text};

fn app() -> Element {
    rsx! {
        Text { id: "mark", "Highlights " Mark { "a chunk" } " of text." }
        Text { id: "combo",
            "Press "
            span { Kbd { "Ctrl" } " + " Kbd { "K" } }
            " to search."
        }
        Text { id: "badge", "Status " Badge { "new" } " today." }
        Text { id: "code", "Call " Code { source: "run()" } " once." }
        Text { id: "anchor", "Read " Anchor { to: "#", "the guide" } " first." }
        Text { id: "nested", "A " Mark { "chunk with " Kbd { "Esc" } " inside" } " here." }
        Text { id: "edge", "x" Mark { " spaced " } "y" }
    }
}

/// An inline-block (`Kbd`, `Badge`) lays its text out apart, hence the gaps.
#[test]
fn inline_components_keep_the_spaces_around_them() {
    let mut page = mount(app);
    page.settle();
    for (id, text) in [
        ("#mark", "Highlights a chunk of text."),
        ("#combo", "Press  +  to search."),
        ("#badge", "Status  today."),
        ("#code", "Call run() once."),
        ("#anchor", "Read the guide first."),
        ("#nested", "A chunk with  inside here."),
    ] {
        assert_eq!(page.laid_out_text(id), text, "{id}");
    }
}

/// A caller's own edge spaces inside an inline element survive, no parley trim.
#[test]
fn a_callers_edge_spaces_survive() {
    let mut page = mount(app);
    page.settle();
    assert_eq!(page.laid_out_text("#edge"), "x spaced y");
}

/// Blitz adds no width for a plain inline element's horizontal margin or padding, so libero
/// sets `inline-block` where it needs the gap (`Table`'s aggregate name).
#[test]
fn an_inline_box_takes_no_horizontal_margin_but_an_inline_block_does() {
    fn app() -> Element {
        rsx! {
            div { id: "plain", style: "width:fit-content", span { "ab" } "cd" }
            div { id: "margin", style: "width:fit-content",
                span { style: "margin-inline-end:6px", "ab" }
                "cd"
            }
            div { id: "padding", style: "width:fit-content",
                span { style: "padding-right:6px", "ab" }
                "cd"
            }
            div { id: "block", style: "width:fit-content",
                span { style: "display:inline-block;margin-inline-end:6px", "ab" }
                "cd"
            }
        }
    }
    let mut page = mount(app);
    page.settle();
    let width = |id: &str| page.rect(id).2;
    let plain = width("#plain");
    assert_eq!(width("#margin"), plain, "an inline margin gained width");
    assert_eq!(width("#padding"), plain, "an inline padding gained width");
    // An inline-block rounds its text width up, a pixel off on a font with fractional advances.
    let gained = width("#block") - plain;
    assert!(
        (6.0..=7.0).contains(&gained),
        "an inline-block margin: {gained}"
    );
}
