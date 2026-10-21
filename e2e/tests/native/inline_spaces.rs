//! Inline components in running text natively. Blitz (parley 0.11) trims the
//! whitespace an inline element starts or ends with (todo 931).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::{
    components::{Anchor, Badge, Code, Kbd, Mark, Text},
    sx::sx,
};

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
        // The no-break spaces are `Code`'s side spacers natively (todo 3877).
        ("#code", "Call \u{202f}run()\u{202f} once."),
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

/// Todos 3698, 3877: natively a no-break space at each end of a `Code` stands in for its side
/// padding, short or long, and a long one still wraps across lines.
#[test]
fn a_code_keeps_its_side_padding_and_a_long_one_wraps() {
    const LONG: &str = "a_long_identifier_past_twenty";
    fn app() -> Element {
        let bare = sx().selector("&::before, &::after", sx().display("none"));
        rsx! {
            div { id: "short", width: "fit-content", Code { source: "run()" } }
            div { id: "short-bare", width: "fit-content",
                Code { source: "run()", sx: bare.clone() }
            }
            div { id: "long", width: "fit-content", Code { source: LONG } }
            div { id: "long-bare", width: "fit-content", Code { source: LONG, sx: bare } }
            div { id: "wrap", width: "120px", Code { source: "one two three four five six" } }
        }
    }
    let mut page = mount(app);
    page.settle();
    assert_eq!(page.computed("#short code", "padding-left"), "0px");
    let width = |id: &str| page.rect(id).2;
    for id in ["#short", "#long"] {
        let gained = width(id) - width(&format!("{id}-bare"));
        assert!((11.0..=13.0).contains(&gained), "{id}: {gained}");
    }
    let line = page.rect("#short").3;
    assert!(
        page.rect("#wrap").3 > 1.5 * line,
        "a long code did not wrap"
    );
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
