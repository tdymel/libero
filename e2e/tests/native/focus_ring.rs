//! Colours Blitz painted unlike the web (todo 478): `outline` before `box-shadow` (the halo hid
//! the stripe), and an inline `<svg>`'s `currentColor` baked in at build (stale after a scheme switch).

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::{ColorScheme, Key, Page, mount, mount_in};
use libero::{
    components::{Button, Icon},
    hooks::use_color_scheme,
};

// libero's scheme re-read interval, and a margin for a loaded machine.
const TICK: Duration = Duration::from_millis(800);

fn ring_app() -> Element {
    rsx! {
        Button { id: "go", "Go" }
    }
}

fn stopping_app() -> Element {
    rsx! {
        button { id: "go", onkeydown: |event: KeyboardEvent| event.stop_propagation(), "Go" }
    }
}

/// A component's stopped key never reaches the wrapper's `keydown`: its release arms the ring.
#[test]
fn a_stopped_key_rings_the_focus_after_a_click() {
    let mut page = mount(stopping_app);
    page.click("#go");
    assert!(
        page.is_focused("#go"),
        "the click focused {}",
        page.focus_owner()
    );
    assert_eq!(page.attr("#go", "data-lsx-focus-visible"), None);
    page.press(Key::Named(NamedKey::ArrowDown));
    assert_eq!(
        page.attr("#go", "data-lsx-focus-visible").as_deref(),
        Some("")
    );
}

fn pressing_app() -> Element {
    rsx! {
        button { id: "first", "First" }
        button { id: "go", onpointerdown: |event: PointerEvent| event.stop_propagation(), "Go" }
    }
}

/// A component's stopped `pointerdown` never reaches the wrapper: the click still disarms the ring.
#[test]
fn a_stopped_press_disarms_the_ring() {
    let mut page = mount(pressing_app);
    page.tab();
    assert!(
        page.is_focused("#first"),
        "Tab reached {}",
        page.focus_owner()
    );
    assert_eq!(
        page.attr("#first", "data-lsx-focus-visible").as_deref(),
        Some("")
    );
    page.click("#go");
    page.focus("#first");
    assert_eq!(page.attr("#first", "data-lsx-focus-visible"), None);
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
