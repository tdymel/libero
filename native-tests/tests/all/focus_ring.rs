//! Colours Blitz painted differently from the web (todo 478). It paints an
//! `outline` before the `box-shadow`s, so the ring's halo covered its stripe;
//! and it bakes an inline `<svg>`'s `currentColor` in when it builds the box,
//! so an icon kept the old scheme's colour after a switch.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::{Button, Icon},
    hooks::use_color_scheme,
};
use native_tests::{ColorScheme, Page, mount, mount_in};

// libero's scheme re-read interval, and a margin for a loaded machine.
const TICK: Duration = Duration::from_millis(800);

fn ring_app() -> Element {
    rsx! {
        Button { id: "go", "Go" }
    }
}

fn icon_app() -> Element {
    let scheme = use_color_scheme();
    rsx! {
        button { id: "toggle", onclick: move |_| scheme.toggle(), "Toggle" }
        Icon {
            variant: "standard",
            color: "primary",
            svg {
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                path { d: "M9 18l6-6-6-6" }
            }
        }
    }
}

/// `rgb(..) 0px 0px 0px 2px, rgb(..) ...` as one string per shadow.
fn shadows(computed: &str) -> Vec<String> {
    computed
        .split(", rgb")
        .enumerate()
        .map(|(index, shadow)| match index {
            0 => shadow.to_string(),
            _ => format!("rgb{shadow}"),
        })
        .collect()
}

#[test]
fn the_stripe_is_painted_above_the_halo() {
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let mut page = mount_in(ring_app, scheme);
        page.tab();
        assert!(page.is_focused("#go"), "Tab reached {}", page.focus_owner());

        let stripe = page.computed("#go", "outline-color");
        let shadows = shadows(&page.computed("#go", "box-shadow"));
        let halo = shadows[0].split(" 0px").next().unwrap().to_string();
        assert_ne!(halo, stripe, "{scheme:?}");
        // First on top: the halo's inner band, the stripe, the halo's outer band.
        assert_eq!(
            shadows[..3],
            [
                format!("{halo} 0px 0px 0px 2px"),
                format!("{stripe} 0px 0px 0px 4px"),
                format!("{halo} 0px 0px 0px 6px"),
            ],
            "{scheme:?}"
        );
    }
}

fn painted_stroke(page: &Page) -> String {
    page.painted_stroke("svg")
}

#[test]
fn an_icon_repaints_when_the_app_switches_its_scheme() {
    let mut page = mount(icon_app);
    let light = painted_stroke(&page);
    assert_eq!(light, page.computed("svg", "color"));

    page.click("#toggle");
    let dark = page.computed("svg", "color");
    assert_ne!(light, dark, "the scheme did not switch");
    assert_eq!(painted_stroke(&page), dark);
}

#[test]
fn an_icon_repaints_when_the_window_switches_its_scheme() {
    let mut page = mount(icon_app);
    page.set_color_scheme(ColorScheme::Dark);
    page.wait(TICK);
    let dark = page.computed("svg", "color");
    assert_eq!(painted_stroke(&page), dark);
}
