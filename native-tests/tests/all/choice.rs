//! `Checkbox` and `RadioGroup`: a click on the drawn box, Space on the hidden
//! input, the arrows in a group. Links in labels are in `pointer.rs`.

use dioxus::prelude::*;
use libero::components::{Checkbox, Options, RadioGroup};
use native_tests::{Key, Page, mount};

const CHECKBOX: &str = "input[type=checkbox]";
const RADIOS: &str = "[role=radiogroup] input[type=radio]";

fn checkbox() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Checkbox { checked: on(), onchange: move |v| on.set(v), label: "Remember me" }
    }
}

fn checked(page: &Page) -> bool {
    page.attr(CHECKBOX, "checked").is_some_and(|v| v == "true")
}

#[test]
fn a_click_on_the_box_toggles_a_checkbox() {
    let mut page = mount(checkbox);
    page.click(&format!("{CHECKBOX} + [aria-hidden=true]"));
    assert!(checked(&page), "{}", page.tree());
    page.click(&format!("{CHECKBOX} + [aria-hidden=true]"));
    assert!(!checked(&page));
}

#[test]
fn space_toggles_a_focused_checkbox() {
    let mut page = mount(checkbox);
    page.tab();
    assert!(page.is_focused(CHECKBOX), "{}", page.focus_owner());
    page.press(Key::Character(" ".into()));
    assert!(checked(&page), "{}", page.tree());
    page.press(Key::Character(" ".into()));
    assert!(!checked(&page));
}

/// Blitz's svg parser reads only `currentColor` as the current colour; the
/// lowercase keyword left the checked box without its mark.
#[test]
fn a_checked_box_paints_its_mark() {
    fn app() -> Element {
        rsx! {
            Checkbox { checked: true, onchange: |_| {}, label: "Remember me" }
        }
    }
    let page = mount(app);
    let mark = format!("{CHECKBOX} + [aria-hidden=true]");
    assert_eq!(
        page.painted_stroke(&format!("{mark} > svg")),
        page.computed(&mark, "color")
    );
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
    Team,
}

fn group() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));
    rsx! {
        RadioGroup { label: "Plan", value: plan(), onchange: move |next| plan.set(Some(next)) }
    }
}

/// The index of the checked radio.
fn picked(page: &Page) -> Option<usize> {
    page.query_all(RADIOS)
        .into_iter()
        .position(|id| page.attr_of(id, "checked").is_some_and(|v| v == "true"))
}

#[test]
fn tab_enters_a_radio_group_at_the_checked_radio() {
    let mut page = mount(group);
    page.tab();
    let radios = page.query_all(RADIOS);
    assert!(
        page.focused() == Some(radios[1]),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn the_arrows_move_and_select_in_a_radio_group() {
    let mut page = mount(group);
    page.focus(&format!("{RADIOS}[checked=true]"));
    page.press(Key::ArrowDown);
    assert_eq!(picked(&page), Some(2), "{}", page.tree());
    assert!(page.focused() == Some(page.query_all(RADIOS)[2]));
    // APG: the arrows wrap.
    page.press(Key::ArrowDown);
    assert_eq!(picked(&page), Some(0), "{}", page.tree());
    page.press(Key::ArrowUp);
    assert_eq!(picked(&page), Some(2));
}

#[test]
fn a_click_on_an_option_label_selects_it() {
    let mut page = mount(group);
    page.click("[role=radiogroup] > div:first-of-type label");
    assert_eq!(picked(&page), Some(0), "{}", page.tree());
}
