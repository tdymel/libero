//! `Carousel`'s rendered contract.

mod common;

use common::{body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Carousel};

/// The autoplay button is a toggle: one fixed name, with `aria-pressed` as the
/// state. A name that flipped to "Play" as well would read "Play, pressed".
#[test]
fn the_autoplay_toggle_has_a_fixed_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Offers",
                    autoplay: true,
                    slides: vec![rsx! { "a" }, rsx! { "b" }],
                }
            }
        }
    }

    let html = body(&render(app));
    assert!(
        html.contains(r#"aria-label="Pause slideshow" aria-pressed="false""#),
        "{html}"
    );
    assert!(!html.contains("Play slideshow"), "{html}");
}

fn six() -> Vec<Element> {
    (1..=6).map(|n| rsx! { "{n}" }).collect()
}

/// Mantine counts where the strip can rest, not slides: six three-up rest at
/// four places, so there are four dots and the status is out of four.
#[test]
fn the_status_and_the_dots_count_resting_positions() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel { aria_label: "Offers", per_view: 3.0, indicators: true, slides: six() }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("Slide 1 of 4"), "{html}");
    assert!(html.contains(r#"aria-label="Go to slide 4""#), "{html}");
    assert!(!html.contains(r#"aria-label="Go to slide 5""#), "{html}");
}

/// Every slide in view is one resting place, whichever slide the caller holds
/// - a lightbox's thumbnail strip opened on its last picture read "4 of 6".
#[test]
fn a_strip_whose_slides_all_fit_is_one_position() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Thumbnails",
                    per_view: 6.0,
                    align: "center",
                    index: Some(5),
                    controls: true,
                    indicators: true,
                    slides: six(),
                }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("Slide 1 of 1"), "{html}");
    assert!(html.contains(r#"aria-label="Go to slide 1""#), "{html}");
    assert!(!html.contains(r#"aria-label="Go to slide 2""#), "{html}");
}
