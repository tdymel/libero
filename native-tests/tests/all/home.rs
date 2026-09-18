//! The docs home page's booking card, from the docs' own source (todo 773).

#[path = "../../../docs/src/pages/home/booking.rs"]
mod booking;

use dioxus::prelude::*;
use libero::components::Notifications;
use native_tests::{Key, Page, mount};

const DAY: &str = "input[role=combobox]";
const DIALOG: &str = "[role=dialog]";
const BOOK: &str = "button[type=submit]";
const SUMMARY: &str = "[role=alert]";

/// Room above and below, so the page scrolls to the card.
fn app() -> Element {
    rsx! {
        div { height: "500px" }
        booking::BookingCard {}
        div { height: "1000px" }
        Notifications {}
    }
}

fn typing(page: &mut Page, text: &str) {
    for c in text.chars() {
        page.press(Key::Character(c.to_string()));
    }
}

/// Arrow Down enters the picker on the typed day, not on the grid cell the
/// month shown before held. A click, not `page.focus`: its focus change and
/// the stylesheet it adds panicked stylo in one frame (todo 837).
#[test]
fn arrow_down_after_typing_enters_on_the_typed_day() {
    let mut page = mount(app);
    page.click(DAY);
    typing(&mut page, "October 3, 2026");
    page.press(Key::ArrowDown);
    assert!(
        page.is_focused("[data-date='2026-10-03']"),
        "focus is on {}",
        page.focus_owner()
    );
}

/// libero's press hit test took the client point, Blitz's the page point: on
/// a scrolled page the press read as one on nothing, and the click was lost.
#[test]
fn on_a_scrolled_page_a_click_beside_the_open_picker_reaches_book() {
    let mut page = mount(app);
    page.wheel_at(100.0, 100.0, 400.0);
    assert_eq!(page.viewport_scroll(), (0.0, 400.0));
    page.click(DAY);
    typing(&mut page, "October 3, 2026");
    page.press(Key::ArrowDown);
    assert!(page.exists(DIALOG), "{}", page.tree());
    // The right end of Book, clear of the picker.
    let (x, y, width, height) = page.rect(BOOK);
    page.click_at((x + width - 10.0) as f32, (y + height / 2.0) as f32);
    assert!(!page.exists(DIALOG), "{}", page.tree());
    assert!(
        page.exists(SUMMARY),
        "Book did not submit; focus is on {}",
        page.focus_owner()
    );
}
