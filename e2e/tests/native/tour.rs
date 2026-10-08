//! `use_tour` in Blitz: the hole lands on the target, found by selector too, the keys step and
//! close, and the mask takes a press on the highlighted element unless the step is interactive.

use dioxus::prelude::*;
use std::time::{Duration, Instant};

use e2e::native::{Key, Page, WAIT_LIMIT, mount};
use libero::{
    components::{Button, TourOptions, TourStep, use_tour},
    hooks::use_element,
};

const HIGHLIGHT: &str = "[data-lsx-tour] > [data-slot=highlight]";
const MASK: &str = "[data-lsx-tour] > [data-slot=mask]";
const CARD: &str = "[data-lsx-tour] [data-slot=card]";
/// `TourDefaults::padding`.
const PADDING: f64 = 6.0;

fn app() -> Element {
    let first = use_element();
    let second = use_element();
    let mut presses = use_signal(|| 0);
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("first").target(first).title("First"),
            TourStep::new("second").target(second).title("Second"),
        ],
        ..Default::default()
    });
    rsx! {
        div { padding: "40px", display: "flex", gap: "40px",
            Button { id: "start", onclick: move |_| tour.start(), "Start" }
            Button {
                id: "first",
                onmounted: first.mount(),
                onclick: move |_| presses += 1,
                "First"
            }
            Button { id: "second", onmounted: second.mount(), "Second" }
            span { id: "presses", "{presses}" }
        }
    }
}

/// Polls `done`, moving the clock on: the hole's move is a CSS transition.
fn until(page: &mut Page, done: impl Fn(&Page) -> bool) -> bool {
    let deadline = Instant::now() + WAIT_LIMIT;
    while !done(page) {
        if Instant::now() >= deadline {
            return false;
        }
        page.wait(Duration::from_millis(5));
        page.advance(0.05);
    }
    true
}

fn close_to(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

/// The hole is `target` grown by the padding on every side.
fn hole_on(page: &Page, target: &str) -> bool {
    if !page.exists(HIGHLIGHT) {
        return false;
    }
    let (x, y, w, h) = page.rect(target);
    let (hx, hy, hw, hh) = page.rect(HIGHLIGHT);
    close_to(hx, x - PADDING)
        && close_to(hy, y - PADDING)
        && close_to(hw, w + 2.0 * PADDING)
        && close_to(hh, h + 2.0 * PADDING)
}

fn started() -> Page {
    let mut page = mount(app);
    page.click("#start");
    assert!(
        until(&mut page, |page| hole_on(page, "#first")),
        "the hole {:?} ({:?}) is not around the first target {:?}",
        page.rect(HIGHLIGHT),
        page.attr(HIGHLIGHT, "style"),
        page.rect("#first")
    );
    page
}

#[test]
fn the_hole_pads_the_target_and_the_card_takes_focus() {
    let mut page = started();
    assert!(
        until(&mut page, |page| page.is_focused(CARD)),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn arrow_right_moves_the_hole_and_escape_returns_focus() {
    let mut page = started();
    until(&mut page, |page| page.is_focused(CARD));
    page.press(Key::ArrowRight);
    assert!(
        until(&mut page, |page| hole_on(page, "#second")),
        "ArrowRight did not move the hole:\n{}",
        page.tree()
    );
    page.press(Key::Escape);
    assert!(
        until(&mut page, |page| !page.exists(CARD)),
        "Escape left the tour open"
    );
    assert!(
        until(&mut page, |page| page.is_focused("#start")),
        "focus is on {}",
        page.focus_owner()
    );
}

/// A selector's step (2220), then an interactive one (2221).
fn more_app() -> Element {
    let pressable = use_element();
    let mut presses = use_signal(|| 0);
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("picked")
                .target_selector("#picked")
                .title("Picked"),
            TourStep::new("pressable")
                .target(pressable)
                .interactive(true)
                .title("Pressable"),
        ],
        ..Default::default()
    });
    rsx! {
        div { padding: "40px", display: "flex", gap: "40px",
            Button { id: "start", onclick: move |_| tour.start(), "Start" }
            Button { id: "picked", "Picked" }
            Button {
                id: "pressable",
                onmounted: pressable.mount(),
                onclick: move |_| presses += 1,
                "Pressable"
            }
            span { id: "presses", "{presses}" }
        }
    }
}

#[test]
fn a_selector_target_takes_the_hole() {
    let mut page = mount(more_app);
    page.click("#start");
    assert!(
        until(&mut page, |page| hole_on(page, "#picked")),
        "the hole {:?} is not around #picked {:?}",
        page.rect(HIGHLIGHT),
        page.rect("#picked")
    );
}

#[test]
fn an_interactive_target_takes_presses_and_tab() {
    let mut page = mount(more_app);
    page.click("#start");
    until(&mut page, |page| page.is_focused(CARD));
    page.press(Key::ArrowRight);
    assert!(
        until(&mut page, |page| hole_on(page, "#pressable")),
        "ArrowRight did not move the hole:\n{}",
        page.tree()
    );
    until(&mut page, |page| page.is_focused(CARD));

    // Tab off the card's last control reaches the target; Tab from it, the card's first.
    let mut reached = false;
    for _ in 0..6 {
        page.press(Key::Tab);
        until(&mut page, |page| !page.is_focused(CARD));
        if page.is_focused("#pressable") {
            reached = true;
            break;
        }
    }
    assert!(
        reached,
        "Tab never reached the target: on {}",
        page.focus_owner()
    );
    page.press(Key::Tab);
    assert!(
        until(&mut page, |page| page
            .is_focused("[data-lsx-tour] [data-slot=close]")),
        "Tab from the target went to {}",
        page.focus_owner()
    );
    page.press_with(Key::Tab, Modifiers::SHIFT);
    assert!(
        until(&mut page, |page| page.is_focused("#pressable")),
        "Shift+Tab from the card's first control went to {}",
        page.focus_owner()
    );

    let (left, top, width, height) = page.rect("#pressable");
    page.click_at((left + width / 2.0) as f32, (top + height / 2.0) as f32);
    assert!(
        until(&mut page, |page| page.text("#presses") == "1"),
        "the press did not reach the target"
    );
    assert!(page.exists(CARD), "a press on the target closed the tour");
}

#[test]
fn the_mask_takes_a_press_on_the_target() {
    let mut page = started();
    let (left, top, width, height) = page.rect("#first");
    let (x, y) = ((left + width / 2.0) as f32, (top + height / 2.0) as f32);
    assert!(
        page.hits_at(MASK, x, y),
        "the press does not reach the mask"
    );
    page.click_at(x, y);
    assert_eq!(page.text("#presses"), "0");
    assert!(page.exists(CARD), "a press on the mask closed the tour");
}
