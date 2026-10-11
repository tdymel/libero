//! `Anchor`: the opt-in new-tab icon after a `_blank` link's text.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::Anchor;

fn app() -> Element {
    rsx! {
        div { Anchor { id: "hinted", to: "https://dioxuslabs.com", target: "_blank", new_tab_icon: true, "Read the docs" } }
        div { Anchor { id: "quiet", to: "https://dioxuslabs.com", target: "_blank", "Read the docs" } }
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
    let (ix, ..) = page.rect("#hinted [data-slot='new-tab']");
    let gap = ix - (tx + tw);
    assert!(gap >= 2.0, "icon {gap}px after the text");
}

/// Todo 3324: the icon is opt-in; the hidden hint stays.
#[test]
fn the_new_tab_icon_is_off_by_default() {
    let page = mount(app);
    assert!(!page.exists("#quiet [data-slot='new-tab']"));
    assert!(page.text("#quiet").contains("(opens in a new tab)"));
}
