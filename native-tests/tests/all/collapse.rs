//! `Collapse`: opening grows it to its content, and closing with
//! `keep_mounted: false` unmounts the content once the exit ends.

use std::time::Duration;

use dioxus::prelude::*;
use libero::components::{Button, Collapse, Text};
use native_tests::{Page, mount};

const TOGGLE: &str = "#toggle-details";
const ROOT: &str = "#details";
const CONTENT: &str = "#details-text";

fn app() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Button { id: "toggle-details", onclick: move |_| open.toggle(), "Shipping details" }
        Collapse { id: "details", open: open(), keep_mounted: false,
            Text { id: "details-text", "Shipping is calculated at checkout." }
        }
    }
}

/// Ends the transition and `use_presence`'s fallback timer.
fn finish(page: &mut Page) {
    page.advance(1.0);
    page.wait(Duration::from_millis(400));
}

#[test]
fn it_opens_to_its_content_and_unmounts_it_once_closed() {
    let mut page = mount(app);
    assert!(!page.exists(CONTENT), "closed, it rendered its content");

    page.click(TOGGLE);
    finish(&mut page);
    let (_, _, _, content) = page.rect(CONTENT);
    let (_, _, _, root) = page.rect(ROOT);
    assert!(
        content > 0.0 && close(root, content),
        "open at {root}px for {content}px of content"
    );

    page.click(TOGGLE);
    finish(&mut page);
    assert!(
        !page.exists(CONTENT),
        "closing left its content mounted:\n{}",
        page.tree()
    );
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 1.0
}
