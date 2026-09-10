//! `SegmentedControl` is hidden radios beside their `<label>`s: a click on a
//! label, and the arrows on a focused radio, move the selection.

use dioxus::prelude::*;
use libero::components::{Options, SegmentedControl};
use native_tests::{Key, Page, mount};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

fn app() -> Element {
    let mut alignment = use_signal(|| Alignment::Center);
    rsx! {
        SegmentedControl {
            label: "Alignment",
            value: alignment(),
            onchange: move |next| alignment.set(next),
        }
    }
}

/// The `aria-label` of the checked radio.
fn selected(page: &Page) -> Option<String> {
    page.query_all("input[type=radio]")
        .into_iter()
        .find(|id| page.attr_of(*id, "checked").is_some_and(|v| v == "true"))
        .and_then(|id| {
            page.attr_of(id, "aria-label")
                .or_else(|| page.attr_of(id, "value"))
        })
}

#[test]
fn a_click_on_a_label_selects_it() {
    let mut page = mount(app);
    assert_eq!(
        selected(&page).as_deref(),
        Some("Center"),
        "{}",
        page.tree()
    );
    page.click("input[type=radio] + label");
    assert_eq!(selected(&page).as_deref(), Some("Left"), "{}", page.tree());
}

#[test]
fn arrow_right_selects_the_next_segment() {
    let mut page = mount(app);
    page.focus("input[type=radio][checked]");
    page.press(Key::ArrowRight);
    assert_eq!(selected(&page).as_deref(), Some("Right"), "{}", page.tree());
}
