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

/// A caller's own edge spaces inside an inline element still go: parley's
/// trim, not a component's ("xspacedy").
#[test]
#[ignore = "Blitz-only: parley 0.11 trims an inline element's edge whitespace"]
fn a_callers_edge_spaces_survive() {
    let mut page = mount(app);
    page.settle();
    assert_eq!(page.laid_out_text("#edge"), "x spaced y");
}
