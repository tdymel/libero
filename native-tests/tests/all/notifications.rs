//! `Notifications`: shown without taking focus, placed in the viewport corner,
//! closed by its button with focus handed on (todo 423), and closed by its own
//! timer.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::{
        Button, NotificationOptions, NotificationScope, Notifications, use_notifications,
        use_notifications_with,
    },
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
fn a_click_on_its_close_button_closes_it() {
    let mut page = mount(app);
    page.click(TRIGGER);
    finish(&mut page);
    page.click(CLOSE);
    finish(&mut page);
    assert!(!page.exists(ITEM), "{}", page.tree());
}

/// The store learns focus is inside from `focusin`, which Tab does not fire
/// on Blitz; the silent-focus check reports it (N6). A harness `focus` is
/// heard by neither, so the test tabs in. Handing on works; the way back out
/// needs where focus came from, which a silent move does not carry (todo 734).
#[test]
#[ignore = "silent focus moves carry no entered_from, so the trigger is not remembered"]
fn closing_a_focused_one_hands_focus_on_and_back_out() {
    let mut page = mount(app);
    page.focus(TRIGGER);
    for _ in 0..3 {
        page.press(Key::Enter);
    }
    finish(&mut page);
    let closes = page.query_all(CLOSE);
    assert_eq!(closes.len(), 3, "{}", page.tree());

    for _ in 0..10 {
        if page.focused().is_some_and(|id| closes.contains(&id)) {
            break;
        }
        page.tab();
    }
    let first = page.focused();
    assert!(
        first.is_some_and(|id| closes.contains(&id)),
        "Tab never reached a close button"
    );
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

fn contained() -> Element {
    let mut mounted = use_context_provider(|| Signal::new(true));
    rsx! {
        Button { id: "before", "Before" }
        if mounted() {
            Notifications { contained: true,
                Inside {}
            }
        }
        Button { id: "drop", onclick: move |_| mounted.set(false), "Drop" }
    }
}

#[component]
fn Inside() -> Element {
    let notify = use_notifications_with(|_: NotificationScope<String>| {
        let mut mounted = use_context::<Signal<bool>>();
        rsx! {
            Button { class: "drop-host", onclick: move |_| mounted.set(false), "Drop host" }
        }
    });
    rsx! {
        Button {
            id: "notify",
            onclick: move |_| {
                notify.show_with(
                    "Inside".to_string(),
                    NotificationOptions { auto_close: Some(AutoClose::Never), ..Default::default() },
                );
            },
            "Notify"
        }
    }
}

/// A contained host unmounting with focus inside and its opener inside too:
/// focus moves to the focusable before the host (todo 589).
#[test]
fn a_contained_host_unmounting_focuses_the_control_before_it() {
    let mut page = mount(contained);
    page.focus(TRIGGER);
    page.press(Key::Enter);
    finish(&mut page);
    page.tab();
    assert!(
        page.is_focused(".drop-host"),
        "Tab went to {}",
        page.focus_owner()
    );
    page.press(Key::Enter);
    finish(&mut page);
    assert!(!page.exists(".drop-host"), "{}", page.tree());
    assert!(
        page.is_focused("#before"),
        "focus is on {}",
        page.focus_owner()
    );
}

const OUTLET: &str = "[style^='position:absolute;width:100vw']";

fn tall() -> Element {
    rsx! {
        {app()}
        div { id: "tall", height: "3000px" }
    }
}

/// An empty stack keeps its portal entry mounted: a wheel leaves the outlet
/// alone until a notification is drawn, which puts it back on the viewport
/// (todo 639).
#[test]
fn a_wheel_realigns_the_outlet_only_once_one_is_drawn() {
    let mut page = mount(tall);
    let resting = page.attr(OUTLET, "style");
    page.hover(TRIGGER);
    page.wheel(TRIGGER, 250.0);
    assert!(page.rect(TRIGGER).1 < 0.0, "the root did not scroll");
    assert_eq!(page.attr(OUTLET, "style"), resting, "realigned for nothing");

    page.focus(TRIGGER);
    page.press(Key::Enter);
    finish(&mut page);
    let (x, y, ..) = page.rect(OUTLET);
    assert!(x.abs() < 0.5 && y.abs() < 0.5, "outlet at ({x}, {y})");

    // Drawn now, so the next wheel realigns at once.
    page.hover("#tall");
    page.wheel("#tall", -100.0);
    let (x, y, ..) = page.rect(OUTLET);
    assert!(
        x.abs() < 0.5 && y.abs() < 0.5,
        "outlet at ({x}, {y}) after a wheel"
    );
}
