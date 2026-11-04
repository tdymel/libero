//! Responsive `orientation` under Blitz (todo 5656): the toolbar's column and keys follow the window.

use dioxus::prelude::*;
use e2e::native::{Key, Page, mount};
use libero::{
    components::{ActionIcon, Orientation, Toolbar},
    theme::responsive,
};

fn app() -> Element {
    rsx! {
        Toolbar {
            id: "rso-toolbar",
            "aria-label": "Formatting",
            orientation: responsive(Orientation::Vertical).md(Orientation::Horizontal),
            ActionIcon { id: "rso-bold", aria_label: "Bold", "B" }
            ActionIcon { id: "rso-italic", aria_label: "Italic", "I" }
            ActionIcon { id: "rso-under", aria_label: "Underline", "U" }
        }
    }
}

fn axis_is(page: &mut Page, direction: &str, focused: &str) {
    let reached = page.wait_for(|page| {
        page.computed("#rso-toolbar", "flex-direction") == direction && page.is_focused(focused)
    });
    assert!(
        reached,
        "{} with {}, want {direction} with {focused}",
        page.computed("#rso-toolbar", "flex-direction"),
        page.focus_owner()
    );
}

#[test]
fn a_responsive_toolbar_orientation_follows_the_window() {
    let mut page = mount(app);
    page.resize(1280, 800);
    page.click("#rso-bold");
    page.press(Key::Named(NamedKey::ArrowRight));
    axis_is(&mut page, "row", "#rso-italic");
    page.resize(600, 800);
    page.press(Key::Named(NamedKey::ArrowDown));
    axis_is(&mut page, "column", "#rso-under");
}
