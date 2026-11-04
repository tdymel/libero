//! `hidden_from` / `visible_from` under Blitz (todo 5647): the lazy rule beats Flex's own
//! `display`, and a field's wrapper and an Anchor hide too.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Anchor, Badge, Flex, TextField};

fn app() -> Element {
    rsx! {
        // First, so the attribute selector below finds its wrapper.
        TextField { hidden_from: "md", label: "Field" }
        Flex { id: "vis-flex", hidden_from: "md", "Flex" }
        Anchor { id: "vis-anchor", hidden_from: "md", to: "https://example.com", "Anchor" }
        Badge { id: "vis-badge", visible_from: "md", "Wide" }
    }
}

fn look(page: &Page) -> [String; 4] {
    [
        page.computed("#vis-flex", "display"),
        page.computed("#vis-anchor", "display"),
        page.computed("[data-state~=\"hidden-from-md\"]", "display"),
        page.computed("#vis-badge", "display"),
    ]
}

#[test]
fn hidden_from_and_visible_from_follow_the_window_width() {
    let mut page = mount(app);
    page.resize(1280, 800);
    let [flex, anchor, field, badge] = look(&page);
    assert_eq!(
        [flex.as_str(), anchor.as_str(), field.as_str()],
        ["none"; 3]
    );
    assert_ne!(badge, "none");

    page.resize(600, 800);
    let [flex, anchor, field, badge] = look(&page);
    assert_eq!(flex, "flex");
    assert_ne!(anchor, "none");
    assert_ne!(field, "none");
    assert_eq!(badge, "none");
}
