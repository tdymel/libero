//! Responsive data_display props under Blitz (todo 5654): an axis copy and a user-layer var
//! follow the window.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{Flex, Icon, Indicator, Timeline, TimelineEvent},
    sx::ThemeAwareValue,
    theme::{Size, responsive},
};

fn app() -> Element {
    rsx! {
        Flex { direction: "column", align: "flex-start",
            Indicator { id: "rsp-indicator", size: responsive(Size::Xs).md(Size::Xl) }
            Icon { id: "rsp-icon", size: responsive(ThemeAwareValue::from("16px")).md("48px".into()), "x" }
            Timeline {
                id: "rsp-timeline",
                gap: responsive(ThemeAwareValue::from("0")).md("40px".into()),
                items: vec![TimelineEvent::new("Ordered"), TimelineEvent::new("Shipped")],
            }
        }
    }
}

/// The indicator's and the icon's heights, then the timeline's row gap.
fn look(page: &Page) -> (f64, f64, String) {
    (
        page.rect("#rsp-indicator").3,
        page.rect("#rsp-icon").3,
        page.computed("#rsp-timeline", "row-gap"),
    )
}

#[test]
fn data_display_props_follow_the_window_width() {
    let mut page = mount(app);
    page.resize(1280, 800);
    let wide = look(&page);
    assert_eq!((wide.1, wide.2.as_str()), (48.0, "40px"), "{wide:?}");

    page.resize(600, 800);
    let narrow = look(&page);
    assert_eq!((narrow.1, narrow.2.as_str()), (16.0, "0px"), "{narrow:?}");
    assert!(narrow.0 < wide.0, "{narrow:?} {wide:?}");
}
