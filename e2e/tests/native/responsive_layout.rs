//! Responsive layout and typography props under Blitz (todo 5651): the range copies and the
//! user-layer breakpoints follow the window.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{Container, Divider, Text},
    theme::{Size, responsive},
};

fn app() -> Element {
    rsx! {
        Text { id: "rl-text", size: responsive(Size::Xs).md(Size::Xl), "Text" }
        Text { id: "rl-text-narrow", size: Size::Xs, "Text" }
        Text { id: "rl-text-wide", size: Size::Xl, "Text" }
        Container { id: "rl-container", size: responsive(Size::Xs).md(Size::Sm), "Content" }
        Container { id: "rl-container-narrow", size: Size::Xs, "Content" }
        Container { id: "rl-container-wide", size: Size::Sm, "Content" }
        Divider { id: "rl-divider", size: responsive(Size::Xs).md(Size::Xl) }
        Divider { id: "rl-divider-narrow", size: Size::Xs }
        Divider { id: "rl-divider-wide", size: Size::Xl }
    }
}

/// Which elements draw unlike their `mode` twin.
fn off(page: &Page, mode: &str) -> Vec<&'static str> {
    [
        ("rl-text", "font-size"),
        ("rl-container", "max-width"),
        ("rl-divider", "border-bottom-width"),
    ]
    .into_iter()
    .filter(|(id, property)| {
        page.computed(&format!("#{id}"), property)
            != page.computed(&format!("#{id}-{mode}"), property)
    })
    .map(|(id, _)| id)
    .collect()
}

#[test]
fn layout_and_type_props_follow_the_window_width() {
    let mut page = mount(app);
    page.resize(1280, 800);
    assert_eq!(off(&page, "wide"), Vec::<&str>::new());

    page.resize(600, 800);
    assert_eq!(off(&page, "narrow"), Vec::<&str>::new());
}
