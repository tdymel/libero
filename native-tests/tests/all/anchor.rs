//! `Anchor`: the new-tab icon after a `_blank` link's text.

use dioxus::prelude::*;
use libero::components::Anchor;
use native_tests::mount;

fn app() -> Element {
    rsx! {
        div { Anchor { id: "hinted", to: "https://dioxuslabs.com", target: "_blank", "Read the docs" } }
        div {
            Anchor {
                id: "bare",
                to: "https://dioxuslabs.com",
                target: "_blank",
                new_tab_hint: false,
                "Read the docs"
            }
        }
    }
}

/// Blitz drops a no-break space at the start of an inline element, so the
/// icon touched the last word (todo 891).
#[test]
fn a_space_parts_the_new_tab_icon_from_the_text() {
    let page = mount(app);
    let (tx, _, tw, _) = page.rect("#bare");
    let (ix, ..) = page.rect("#hinted [data-anchor-new-tab]");
    let gap = ix - (tx + tw);
    assert!(gap >= 2.0, "icon {gap}px after the text");
}
