//! `DirectionToggle` turns the document root's `dir` on Blitz, where the root
//! is reached through the provider's anchor rather than a script.

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::DirectionToggle;

const BUTTON: &str = "#direction";

fn app() -> Element {
    rsx! {
        DirectionToggle { id: "direction" }
    }
}

#[test]
fn a_click_and_enter_turn_the_root_and_rename_the_button() {
    let mut page = mount(app);
    assert_eq!(
        page.attr("html", "dir"),
        None,
        "the root was turned at rest"
    );
    assert_eq!(
        page.attr(BUTTON, "aria-label").as_deref(),
        Some("Switch to right-to-left text")
    );

    page.click(BUTTON);
    assert_eq!(
        page.attr("html", "dir").as_deref(),
        Some("rtl"),
        "{}",
        page.tree()
    );
    assert_eq!(
        page.attr(BUTTON, "aria-label").as_deref(),
        Some("Switch to left-to-right text")
    );

    page.focus(BUTTON);
    page.press(Key::Enter);
    assert_eq!(page.attr("html", "dir").as_deref(), Some("ltr"));
    assert_eq!(
        page.attr(BUTTON, "aria-label").as_deref(),
        Some("Switch to right-to-left text")
    );
}
