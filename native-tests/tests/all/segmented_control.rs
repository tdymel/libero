//! `SegmentedControl` is hidden radios beside their `<label>`s: a click on a
//! label, and the arrows on a focused radio, move the selection.

use dioxus::prelude::*;
use libero::components::{Form, Options, SegmentedControl};
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

#[test]
fn the_arrows_move_focus_with_the_selection_and_wrap() {
    let mut page = mount(app);
    page.focus("input[type=radio][checked]");
    page.press(Key::ArrowLeft);
    assert_eq!(selected(&page).as_deref(), Some("Left"), "{}", page.tree());
    assert!(
        page.is_focused("input[aria-label=Left]"),
        "focus is on {}",
        page.focus_owner()
    );
    page.press(Key::ArrowLeft);
    assert_eq!(selected(&page).as_deref(), Some("Right"), "{}", page.tree());
    assert!(
        page.is_focused("input[aria-label=Right]"),
        "focus is on {}",
        page.focus_owner()
    );
    page.press(Key::ArrowDown);
    assert_eq!(selected(&page).as_deref(), Some("Left"), "{}", page.tree());
}

/// Outside a `Form`, Enter picks the focused segment as Space does (todo 648).
#[test]
fn enter_picks_the_focused_segment() {
    let mut page = mount(app);
    page.focus("input[aria-label=Left]");
    page.press(Key::Enter);
    assert_eq!(selected(&page).as_deref(), Some("Left"), "{}", page.tree());
}

/// Inside a `Form`, Enter submits it as for a native radio and picks nothing
/// (todo 508).
#[test]
fn enter_in_a_form_submits_and_picks_nothing() {
    fn app() -> Element {
        let mut alignment = use_signal(|| Alignment::Center);
        let mut submits = use_signal(|| 0u32);
        rsx! {
            Form::<()> { onsubmit: move |_| submits += 1,
                SegmentedControl {
                    label: "Alignment",
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                }
                button { r#type: "submit", "Save" }
            }
            span { id: "submits", "{submits}" }
        }
    }
    let mut page = mount(app);
    page.focus("input[aria-label=Left]");
    page.press(Key::Enter);
    assert_eq!(
        selected(&page).as_deref(),
        Some("Center"),
        "{}",
        page.tree()
    );
    assert_eq!(page.text("#submits"), "1", "{}", page.tree());
}

/// A radio group is one tab stop, the checked radio: Tab in lands there and
/// the next Tab leaves the strip.
#[test]
fn tab_enters_on_the_checked_segment_and_leaves_with_the_next_tab() {
    fn app() -> Element {
        let mut alignment = use_signal(|| Alignment::Center);
        rsx! {
            button { id: "before", "Before" }
            SegmentedControl {
                label: "Alignment",
                value: alignment(),
                onchange: move |next| alignment.set(next),
            }
            button { id: "after", "After" }
        }
    }
    let mut page = mount(app);
    page.focus("#before");
    page.tab();
    assert!(
        page.is_focused("input[aria-label=Center]"),
        "Tab landed on {}",
        page.focus_owner()
    );
    page.tab();
    assert!(
        page.is_focused("#after"),
        "the second Tab landed on {}",
        page.focus_owner()
    );
    page.shift_tab();
    assert!(
        page.is_focused("input[aria-label=Center]"),
        "Shift+Tab landed on {}",
        page.focus_owner()
    );
}
