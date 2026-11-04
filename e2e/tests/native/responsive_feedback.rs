//! Responsive feedback, navigation and overlay props under Blitz (todo 5655): the range copies
//! and the user-layer breakpoints follow the window.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{Loader, ProgressBar, Skeleton},
    sx::ThemeAwareValue,
    theme::{Size, responsive},
};

fn app() -> Element {
    let css = ThemeAwareValue::from;
    rsx! {
        Loader { id: "rf-loader", size: responsive(Size::Xs).md(Size::Xl) }
        Loader { id: "rf-loader-narrow", size: Size::Xs }
        Loader { id: "rf-loader-wide", size: Size::Xl }
        ProgressBar { id: "rf-progress", aria_label: "Upload", value: 40.0,
            radius: responsive(ThemeAwareValue::from(Size::Xs)).md(css("0")),
        }
        ProgressBar { id: "rf-progress-narrow", aria_label: "Upload", value: 40.0, radius: Size::Xs }
        ProgressBar { id: "rf-progress-wide", aria_label: "Upload", value: 40.0, radius: "0" }
        Skeleton { id: "rf-skeleton", height: responsive(css("20px")).md(css("60px")) }
        Skeleton { id: "rf-skeleton-narrow", height: "20px" }
        Skeleton { id: "rf-skeleton-wide", height: "60px" }
    }
}

/// Which elements draw unlike their `mode` twin.
fn off(page: &Page, mode: &str) -> Vec<&'static str> {
    [
        ("rf-loader", "width"),
        ("rf-progress", "border-top-left-radius"),
        ("rf-skeleton", "height"),
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
fn feedback_props_follow_the_window_width() {
    let mut page = mount(app);
    page.resize(1280, 800);
    assert_eq!(off(&page, "wide"), Vec::<&str>::new());

    page.resize(600, 800);
    assert_eq!(off(&page, "narrow"), Vec::<&str>::new());
}
