//! `FloatingWindow`: a non-modal window. Enter opens it with focus inside and
//! Escape hands focus back; the title bar moves it and the corner separator
//! resizes it, each by the theme's step, and both report the drawn rect.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::{Button, FloatingWindowOptions, Text, WindowRect},
    hooks::use_floating_window,
    sx::sx,
};
use native_tests::{Key, Page, mount};

const TRIGGER: &str = "#open-window";
const DIALOG: &str = "[role=dialog]";
const HANDLE: &str = "[role=dialog] [data-window-handle]";
const SEPARATOR: &str = "[role=dialog] [role=separator]";
const MOVE_REPORT: &str = "#move-report";
const RESIZE_REPORT: &str = "#resize-report";
const MIN: (f64, f64) = (240.0, 120.0);
const MAX: (f64, f64) = (480.0, 360.0);
const STEP: f64 = 10.0;

fn app() -> Element {
    let mut moved = use_signal(String::new);
    let mut resized = use_signal(String::new);
    let record_move = use_callback(move |rect: WindowRect| moved.set(rect_text(rect)));
    let record_resize = use_callback(move |rect: WindowRect| resized.set(rect_text(rect)));
    let inspector = use_floating_window(
        FloatingWindowOptions {
            title: Some("Inspector".into()),
            resizable: true,
            sx: sx()
                .min_width("240px")
                .min_height("120px")
                .max_width("480px")
                .max_height("360px")
                .into(),
            onmove: Some(record_move),
            onresize: Some(record_resize),
            ..Default::default()
        },
        |window| {
            rsx! {
                Text { "Use the arrow keys on the title bar." }
                Button { id: "window-done", onclick: move |_| window.close(), "Done" }
            }
        },
    );
    rsx! {
        Button { id: "open-window", onclick: move |_| inspector.open(), "Inspector" }
        div { id: "move-report", "{moved}" }
        div { id: "resize-report", "{resized}" }
    }
}

fn rect_text(rect: WindowRect) -> String {
    format!(
        "{} {} {} {}",
        rect.x.round(),
        rect.y.round(),
        rect.width.round(),
        rect.height.round()
    )
}

fn open(page: &mut Page) {
    page.focus(TRIGGER);
    page.press(Key::Enter);
    settle(page);
    assert!(
        page.exists(DIALOG),
        "Enter did not open it:\n{}",
        page.tree()
    );
}

/// The reports are owed to an effect.
fn settle(page: &mut Page) {
    page.wait(Duration::from_millis(20));
}

fn assert_reported(page: &Page, report: &str, what: &str) {
    let (x, y, width, height) = page.rect(DIALOG);
    let drawn = format!(
        "{} {} {} {}",
        x.round(),
        y.round(),
        width.round(),
        height.round()
    );
    assert_eq!(
        page.text(report),
        drawn,
        "{what} reported another rect than the drawn one"
    );
}

fn close_to(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

#[test]
fn enter_opens_it_with_focus_inside_and_escape_hands_it_back() {
    let mut page = mount(app);
    open(&mut page);
    assert!(
        page.is_focused(&format!("{DIALOG}, {DIALOG} *")),
        "focus is on {}",
        page.focus_owner()
    );
    page.press(Key::Escape);
    settle(&mut page);
    assert!(!page.exists(DIALOG), "Escape left it open");
    assert!(
        page.is_focused(TRIGGER),
        "focus is on {}",
        page.focus_owner()
    );
}

/// F6 from inside the window returns to the page, and back to the window
/// (todo 671): the window's `:focus` check has to see focus inside it.
#[test]
fn f6_moves_focus_between_the_window_and_the_page() {
    let mut page = mount(app);
    open(&mut page);
    for inside in [DIALOG, "#window-done", HANDLE] {
        page.focus(inside);
        page.press(Key::F6);
        settle(&mut page);
        assert!(
            page.is_focused(TRIGGER),
            "F6 from {inside} left focus on {}",
            page.focus_owner()
        );
        page.press(Key::F6);
        settle(&mut page);
        assert!(
            page.is_focused(DIALOG),
            "F6 from the page left focus on {}",
            page.focus_owner()
        );
    }
}

/// `Float` places the window by a translate, which Blitz's client rect leaves
/// out: the move starts from the untranslated box, so a centred window jumps.
#[test]
#[ignore = "needs Blitz: getBoundingClientRect leaves out a transform"]
fn an_arrow_on_the_title_bar_moves_it_by_a_step() {
    let mut page = mount(app);
    open(&mut page);
    page.focus(HANDLE);
    let before = page.rect(DIALOG);
    page.press(Key::ArrowRight);
    settle(&mut page);
    page.press(Key::ArrowDown);
    settle(&mut page);
    let after = page.rect(DIALOG);
    assert!(
        close_to(after.0, before.0 + STEP) && close_to(after.1, before.1 + STEP),
        "moved from {before:?} to {after:?}"
    );
    assert!(
        close_to(after.2, before.2) && close_to(after.3, before.3),
        "the move resized it"
    );
    assert_reported(&page, MOVE_REPORT, "the move");
}

#[test]
fn the_separator_resizes_it_and_clamps_to_the_callers_bounds() {
    let mut page = mount(app);
    open(&mut page);
    page.focus(SEPARATOR);
    let before = page.rect(DIALOG);
    page.press(Key::ArrowRight);
    settle(&mut page);
    let stepped = page.rect(DIALOG);
    assert!(
        close_to(stepped.2, before.2 + STEP),
        "{before:?} became {stepped:?}"
    );
    assert_reported(&page, RESIZE_REPORT, "the step");

    for (key, (width, height)) in [(Key::Home, MIN), (Key::End, MAX)] {
        page.press(key.clone());
        settle(&mut page);
        let (_, _, w, h) = page.rect(DIALOG);
        assert!(
            close_to(w, width) && close_to(h, height),
            "{key:?}: {w}x{h}"
        );
        assert_reported(&page, RESIZE_REPORT, &format!("{key:?}"));
        assert_eq!(
            page.attr(SEPARATOR, "aria-valuenow"),
            Some(width.to_string())
        );
        assert_eq!(
            page.attr(SEPARATOR, "aria-valuetext"),
            Some(format!("{width} by {height} pixels"))
        );
    }
    assert_eq!(
        page.attr(SEPARATOR, "aria-valuemin"),
        Some(MIN.0.to_string())
    );
    assert_eq!(
        page.attr(SEPARATOR, "aria-valuemax"),
        Some(MAX.0.to_string())
    );
}

/// The press lands where the untranslated rect says, beside the handle.
#[test]
#[ignore = "needs Blitz: getBoundingClientRect leaves out a transform"]
fn a_drag_leaves_its_handle_focused() {
    let mut page = mount(app);
    open(&mut page);
    for handle in [SEPARATOR, HANDLE] {
        page.drag(handle, 20.0, 20.0);
        settle(&mut page);
        assert!(
            page.is_focused(handle),
            "after dragging {handle} focus is on {}",
            page.focus_owner()
        );
    }
}
