//! `Notifications`: shown without taking focus, placed in the viewport corner,
//! closed by its button with focus handed on (todo 423), and closed by its own
//! timer.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::{Button, NotificationOptions, Notifications, use_notifications},
    theme::AutoClose,
};
use native_tests::{Key, Page, VIEWPORT, mount};

const TRIGGER: &str = "#notify";
const TIMED: &str = "#notify-timed";
const ITEM: &str = "[aria-live] li";
const CLOSE: &str = "[aria-live] li [data-slot=close]";

fn app() -> Element {
    let notify = use_notifications();
    rsx! {
        Notifications {}
        Button {
            id: "notify",
            onclick: move |_| {
                notify.show_with(
                    "Saved to your library",
                    NotificationOptions { auto_close: Some(AutoClose::Never), ..Default::default() },
                );
            },
            "Notify"
        }
        Button {
            id: "notify-timed",
            onclick: move |_| {
                notify.show_with(
                    "Draft saved",
                    NotificationOptions { auto_close: Some(AutoClose::After(50)), ..Default::default() },
                );
            },
            "Notify for a while"
        }
    }
}

/// Ends the enter or exit animation and libero's timer behind it.
fn finish(page: &mut Page) {
    page.advance(1.0);
    page.wait(Duration::from_millis(400));
}

#[test]
fn a_click_shows_one_without_taking_focus() {
    let mut page = mount(app);
    page.focus(TRIGGER);
    page.press(Key::Enter);
    finish(&mut page);
    assert_eq!(page.query_all(ITEM).len(), 1, "{}", page.tree());
    assert!(page.text(ITEM).contains("Saved to your library"));
    assert!(
        page.is_focused(TRIGGER),
        "focus moved to {}",
        page.focus_owner()
    );
}

/// The stack's own offset is a translate, which the rect leaves out: this
/// checks the untranslated box, which sits flush in the corner.
#[test]
fn it_sits_inside_the_viewport() {
    let mut page = mount(app);
    page.click(TRIGGER);
    finish(&mut page);
    let (x, y, width, height) = page.rect(ITEM);
    let (vw, vh) = (f64::from(VIEWPORT.0), f64::from(VIEWPORT.1));
    assert!(
        width > 0.0 && x >= 0.0 && y >= 0.0 && x + width <= vw && y + height <= vh,
        "item at ({x}, {y}) {width}x{height}"
    );
}

#[test]
fn enter_on_its_close_button_closes_it() {
    let mut page = mount(app);
    page.click(TRIGGER);
    finish(&mut page);
    page.focus(CLOSE);
    page.press(Key::Enter);
    finish(&mut page);
    assert!(!page.exists(ITEM), "{}", page.tree());
}

/// See `hit::a_translated_box_reports_where_it_is_drawn`.
#[test]
#[ignore = "needs Blitz: getBoundingClientRect leaves out the stack's translate, so the click misses"]
fn a_click_on_its_close_button_closes_it() {
    let mut page = mount(app);
    page.click(TRIGGER);
    finish(&mut page);
    page.click(CLOSE);
    finish(&mut page);
    assert!(!page.exists(ITEM), "{}", page.tree());
}

/// The store learns focus is inside from `focusin`, which neither a direct
/// focus nor Tab fires on Blitz.
#[test]
#[ignore = "no focusin on a direct focus or Tab natively: todo 468 N6 (Olaf-26)"]
fn closing_a_focused_one_hands_focus_on_and_back_out() {
    let mut page = mount(app);
    page.focus(TRIGGER);
    for _ in 0..3 {
        page.press(Key::Enter);
    }
    finish(&mut page);
    let closes = page.query_all(CLOSE);
    assert_eq!(closes.len(), 3, "{}", page.tree());

    // Entered from the trigger, as a Tab would.
    page.focus(CLOSE);
    let first = page.focused();
    page.press(Key::Enter);
    finish(&mut page);
    assert_eq!(page.query_all(CLOSE).len(), 2);
    assert!(
        page.focused()
            .is_some_and(|id| id != first.unwrap() && page.query_all(CLOSE).contains(&id)),
        "closing one left focus on {}",
        page.focus_owner()
    );

    page.press(Key::Enter);
    finish(&mut page);
    page.press(Key::Enter);
    finish(&mut page);
    assert!(!page.exists(ITEM), "{}", page.tree());
    assert!(
        page.is_focused(TRIGGER),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn a_timed_one_closes_itself() {
    let mut page = mount(app);
    page.click(TIMED);
    // The hide timer, then the exit's.
    finish(&mut page);
    finish(&mut page);
    assert!(
        !page.exists(ITEM),
        "it did not close itself:\n{}",
        page.tree()
    );
}
