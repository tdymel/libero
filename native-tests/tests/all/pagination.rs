//! `Pagination`: a click or Enter changes the page, and an arrow that disables
//! under its own press hands focus to the current page (todo 406).

use std::time::Duration;

use dioxus::prelude::*;
use libero::{components::Pagination, platform::timer};
use native_tests::{Key, Page, mount};

const PREVIOUS: &str = "[aria-label=\"Go to previous page\"]";
const LAST: &str = "[aria-label=\"Go to last page\"]";
const CURRENT: &str = "[aria-current=page]";

fn pages(delay: Option<Duration>) -> Element {
    let mut page = use_signal(|| 2u32);
    // A caller that answers late, as one that fetches first does.
    let mut pending = use_hook(|| CopyValue::new(None));
    rsx! {
        Pagination {
            total: 10,
            page: page(),
            aria_label: "Results pages",
            with_edges: true,
            onchange: move |next: u32| match delay {
                None => page.set(next),
                Some(delay) => pending.set(
                    timer().map(|timer| timer.after(delay, Box::new(move || page.set(next)))),
                ),
            },
        }
        span { id: "page", "{page}" }
    }
}

fn now() -> Element {
    pages(None)
}

fn late() -> Element {
    pages(Some(Duration::from_millis(30)))
}

fn assert_page(page: &mut Page, number: u32) {
    page.wait(Duration::from_millis(80));
    assert_eq!(page.text("#page"), number.to_string());
    assert_eq!(page.text(CURRENT), number.to_string(), "{}", page.tree());
}

fn a_disabling_arrow_hands_focus_on(app: fn() -> Element) {
    let mut page = mount(app);
    page.click("[aria-label=\"Go to page 3\"]");
    assert_page(&mut page, 3);
    assert!(
        page.is_focused(CURRENT),
        "focus is on {}",
        page.focus_owner()
    );

    page.focus(PREVIOUS);
    page.press(Key::Enter);
    assert_page(&mut page, 2);
    page.press(Key::Enter);
    assert_page(&mut page, 1);
    assert!(
        page.attr(PREVIOUS, "disabled").is_some(),
        "previous at page 1"
    );
    assert!(
        page.is_focused(CURRENT),
        "focus is on {}",
        page.focus_owner()
    );

    page.click(LAST);
    assert_page(&mut page, 10);
    assert!(page.attr(LAST, "disabled").is_some(), "last at page 10");
    assert!(
        page.is_focused(CURRENT),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn a_disabling_arrow_hands_focus_to_the_current_page() {
    a_disabling_arrow_hands_focus_on(now);
}

#[test]
fn a_late_answer_hands_focus_to_the_current_page_too() {
    a_disabling_arrow_hands_focus_on(late);
}
