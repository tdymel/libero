//! `Scroller` natively (todo 659): Blitz sends no `resize` and no `scroll` to
//! the strip, so its edges come from a laid-out measure and `ScrollApi`.

use dioxus::prelude::*;
use libero::components::Scroller;
use native_tests::{Page, mount};

const BACK: &str = "[data-state~=start]";
const FORWARD: &str = "[data-state~=end]";

fn strip() -> Element {
    rsx! {
        div { style: "width: 300px;",
            Scroller { aria_label: "Tags",
                div { style: "display: flex; width: 1200px;",
                    for i in 0..12 {
                        span { style: "width: 100px; flex: none;", "Item {i}" }
                    }
                }
            }
        }
    }
}

fn disabled(page: &Page, control: &str) -> String {
    page.attr(control, "aria-disabled").unwrap_or_default()
}

/// [`disabled`], once it reads `expected` or the wait runs out: the strip's
/// measure is a timer, late on a loaded machine.
fn disabled_once(page: &mut Page, control: &str, expected: &str) -> String {
    page.wait_for(|page| disabled(page, control) == expected);
    disabled(page, control)
}

#[test]
fn an_overflowing_strip_offers_its_forward_control() {
    let mut page = mount(strip);
    assert_eq!(
        disabled_once(&mut page, FORWARD, "false"),
        "false",
        "{}",
        page.tree()
    );
    assert_eq!(disabled(&page, BACK), "true");
}

#[test]
fn a_step_enables_the_back_control() {
    let mut page = mount(strip);
    disabled_once(&mut page, FORWARD, "false");
    page.click(FORWARD);
    assert_eq!(
        disabled_once(&mut page, BACK, "false"),
        "false",
        "{}",
        page.tree()
    );
}

/// Under RTL the strip starts at its right edge (todo 115).
#[test]
fn under_rtl_a_step_enables_the_back_control() {
    let mut page = mount(|| rsx! { div { dir: "rtl", {strip()} } });
    assert_eq!(
        disabled_once(&mut page, FORWARD, "false"),
        "false",
        "{}",
        page.tree()
    );
    page.click(FORWARD);
    assert_eq!(
        disabled_once(&mut page, BACK, "false"),
        "false",
        "{}",
        page.tree()
    );
}

fn fitting() -> Element {
    rsx! {
        div { style: "width: 300px;",
            Scroller { aria_label: "Tags", span { "Short" } }
        }
    }
}

#[test]
fn a_strip_that_fits_offers_neither_control() {
    let mut page = mount(fitting);
    assert_eq!(disabled_once(&mut page, FORWARD, "true"), "true");
    assert_eq!(disabled(&page, BACK), "true");
}

// The button sits outside the strip, so the growth comes from the caller's
// render, not from a scroll.
fn growing() -> Element {
    let mut count = use_signal(|| 1);
    rsx! {
        button { id: "more", onclick: move |_| count.set(12), "More" }
        div { style: "width: 300px;",
            Scroller { aria_label: "Tags",
                div { style: "display: flex;",
                    for i in 0..count() {
                        span { key: "{i}", style: "width: 100px; flex: none;", "Item {i}" }
                    }
                }
            }
        }
    }
}

/// Blitz sends no `resize`: the edges follow content the caller renders into
/// the strip (todo 677).
#[test]
fn a_strip_that_grows_offers_its_forward_control() {
    let mut page = mount(growing);
    assert_eq!(disabled_once(&mut page, FORWARD, "true"), "true");

    page.click("#more");
    assert_eq!(
        disabled_once(&mut page, FORWARD, "false"),
        "false",
        "{}",
        page.tree()
    );
}

fn narrowing() -> Element {
    let mut width = use_signal(|| 1300);
    rsx! {
        button { id: "narrow", onclick: move |_| width.set(300), "Narrow" }
        div { style: "width: {width}px;",
            Scroller { aria_label: "Tags",
                div { style: "width: 1200px;", "Wide" }
            }
        }
    }
}

/// The caller's render that narrows the pane re-renders the strip too.
#[test]
fn a_strip_that_narrows_offers_its_forward_control() {
    let mut page = mount(narrowing);
    assert_eq!(disabled_once(&mut page, FORWARD, "true"), "true");

    page.click("#narrow");
    assert_eq!(
        disabled_once(&mut page, FORWARD, "false"),
        "false",
        "{}",
        page.tree()
    );
}

// Reads the count itself, so a change re-renders it and not the `Scroller`.
#[component]
fn Items() -> Element {
    let count = use_context::<Signal<usize>>();
    rsx! {
        div { style: "display: flex;",
            for i in 0..count() {
                span { key: "{i}", style: "width: 100px; flex: none;", "Item {i}" }
            }
        }
    }
}

fn growing_inside() -> Element {
    let mut count = use_context_provider(|| Signal::new(1usize));
    rsx! {
        button { id: "more", onclick: move |_| count.set(12), "More" }
        div { style: "width: 300px;",
            Scroller { aria_label: "Tags", Items {} }
        }
    }
}

/// A change inside a child component renders nothing of the strip's; the
/// platform's resize watch measures it after the click (todo 788).
#[test]
fn a_strip_that_grows_inside_a_child_offers_its_forward_control() {
    let mut page = mount(growing_inside);
    assert_eq!(disabled_once(&mut page, FORWARD, "true"), "true");

    page.click("#more");
    assert_eq!(
        disabled_once(&mut page, FORWARD, "false"),
        "false",
        "{}",
        page.tree()
    );
}

fn full_width() -> Element {
    rsx! {
        Scroller { aria_label: "Tags",
            div { style: "width: 900px;", "Wide" }
        }
    }
}

/// A window resize renders nothing: the watch's poll sees the strip narrow
/// (todo 788).
#[test]
fn a_narrowed_window_offers_the_forward_control() {
    let mut page = mount(full_width);
    assert_eq!(
        disabled_once(&mut page, FORWARD, "true"),
        "true",
        "{}",
        page.tree()
    );

    page.resize(600, 768);
    assert_eq!(
        disabled_once(&mut page, FORWARD, "false"),
        "false",
        "{}",
        page.tree()
    );
}
