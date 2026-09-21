//! Blitz paints text at its last min-content measure when taffy serves the layout from cache;
//! libero re-lays it (todo 888). Real soft wraps and preserved white space stay.

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::mount;

fn real_wraps() -> Element {
    rsx! {
        div { id: "narrow", style: "width: 60px;", "one two three four" }
        div { id: "pre", style: "white-space: pre-wrap; width: 400px;", "one\ntwo" }
    }
}

#[test]
fn real_wraps_and_preserved_breaks_stay() {
    let mut page = mount(real_wraps);
    page.wait(Duration::from_millis(50));
    assert_eq!(
        page.wrapped_text("#narrow").len(),
        1,
        "the narrow box's wrap"
    );
    assert_eq!(page.wrapped_text("#pre").len(), 1, "the preserved break");
}
