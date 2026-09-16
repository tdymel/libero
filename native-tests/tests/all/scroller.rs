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
