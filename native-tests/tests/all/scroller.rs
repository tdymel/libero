//! `Scroller` natively (todo 659): Blitz sends no `resize` and no `scroll` to
//! the strip, so its edges come from a laid-out measure and `ScrollApi`.

use std::time::Duration;

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

#[test]
fn an_overflowing_strip_offers_its_forward_control() {
    let mut page = mount(strip);
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, FORWARD), "false", "{}", page.tree());
    assert_eq!(disabled(&page, BACK), "true");
}

#[test]
fn a_step_enables_the_back_control() {
    let mut page = mount(strip);
    page.wait(Duration::from_millis(50));
    page.click(FORWARD);
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, BACK), "false", "{}", page.tree());
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
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, FORWARD), "true");
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
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, FORWARD), "true");

    page.click("#more");
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, FORWARD), "false", "{}", page.tree());
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
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, FORWARD), "true");

    page.click("#narrow");
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, FORWARD), "false", "{}", page.tree());
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

/// Pins a gap: a change inside a child component renders nothing of the
/// strip's, and Blitz reports neither a resize nor a mutation.
#[test]
#[ignore = "needs Blitz: no resize event"]
fn a_strip_that_grows_inside_a_child_offers_its_forward_control() {
    let mut page = mount(growing_inside);
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, FORWARD), "true");

    page.click("#more");
    page.wait(Duration::from_millis(50));
    assert_eq!(disabled(&page, FORWARD), "false", "{}", page.tree());
}
