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
