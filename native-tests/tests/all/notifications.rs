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
const SLOW: &str = "#notify-slow";
const ITEM: &str = "[aria-live] li";
const CLOSE: &str = "[aria-live] li [data-slot=close]";
const ELSEWHERE: &str = "#elsewhere";

fn app() -> Element {
    let notify = use_notifications();
    rsx! {
        Notifications {}
        // Before the triggers, so a Tab from the last one enters the stack.
        Button { id: "elsewhere", "Elsewhere" }
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
        Button {
            id: "notify-slow",
            onclick: move |_| {
                notify.show_with(
                    "Draft synced",
                    NotificationOptions { auto_close: Some(AutoClose::After(300)), ..Default::default() },
                );
            },
            "Notify for longer"
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

/// Shown from the keyboard, focus stays on the trigger: nothing pauses it
/// (todo 471).
#[test]
fn one_shown_from_the_keyboard_closes_itself() {
    let mut page = mount(app);
    page.focus(TIMED);
    page.press(Key::Enter);
    finish(&mut page);
    finish(&mut page);
    assert!(
        !page.exists(ITEM),
        "it stayed with focus on {}:\n{}",
        page.focus_owner(),
        page.tree()
    );
}

/// Tab fires no `focusin`/`focusout` natively: the silent focus move pauses
/// and resumes it (todo 471).
#[test]
fn tabbing_into_one_pauses_it_until_focus_leaves() {
    let mut page = mount(app);
    page.focus(SLOW);
    page.press(Key::Enter);
    page.tab();
    assert!(page.is_focused(CLOSE), "Tab went to {}", page.focus_owner());
    finish(&mut page);
    finish(&mut page);
    assert!(page.exists(ITEM), "it closed under focus");

    page.shift_tab();
    finish(&mut page);
    finish(&mut page);
    assert!(
        !page.exists(ITEM),
        "it stayed after focus left for {}:\n{}",
        page.focus_owner(),
        page.tree()
    );
}

/// Tabbed in, then clicked out: Blitz fires that click's `focusout`, so the
/// pause ends there (todo 471).
#[test]
fn a_click_out_after_tabbing_in_resumes_it() {
    for (target, out) in [(Some(ELSEWHERE), "a button"), (None, "nothing")] {
        let mut page = mount(app);
        page.focus(SLOW);
        page.press(Key::Enter);
        page.tab();
        assert!(page.is_focused(CLOSE), "Tab went to {}", page.focus_owner());
        match target {
            Some(target) => page.click(target),
            None => page.click_at(500.0, 400.0),
        }
        finish(&mut page);
        finish(&mut page);
        assert!(
            !page.exists(ITEM),
            "a click on {out} left it paused, focus on {}",
            page.focus_owner()
        );
    }
}

#[test]
fn the_pointer_on_one_pauses_it_until_it_leaves() {
    let mut page = mount(app);
    page.click(SLOW);
    page.hover(ITEM);
    finish(&mut page);
    finish(&mut page);
    assert!(page.exists(ITEM), "it closed under the pointer");

    page.hover(TRIGGER);
    finish(&mut page);
    finish(&mut page);
    assert!(!page.exists(ITEM), "it stayed:\n{}", page.tree());
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
