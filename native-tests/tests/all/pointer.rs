//! Pointer input `LiberoProvider`'s native wrapper watches: a drag followed
//! past its element in place of pointer capture, and a press on a link nested
//! in a label or card.

use dioxus::prelude::*;
use libero::{
    components::{Checkbox, Radio, Slider, SliderChangeEvent, Switch},
    sx::sx,
    theme::ChoiceVariant,
};
use native_tests::{Page, mount};

const THUMB: &str = "[role=slider]";
const TRACK: &str = "[data-state~=size-md] > [data-state~=size-md] > div";
const CHECKBOX: &str = "input[type=checkbox]";

fn slider() -> Element {
    let mut value = use_signal(|| 0.0);
    rsx! {
        // Tall enough that a pointer below the slider is still inside the app.
        div { height: "600px",
            Slider {
                aria_label: "Volume",
                value: Some(value()),
                oninput: move |event: SliderChangeEvent<f64>| value.set(event.value()),
                sx: sx().width("400px"),
            }
        }
    }
}

fn value(page: &Page) -> f64 {
    page.attr(THUMB, "aria-valuenow")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default()
}

#[test]
fn a_drag_leaving_the_slider_keeps_moving_it() {
    let mut page = mount(slider);
    // Right past the track's end and down off the slider, onto the page.
    page.drag(TRACK, 300.0, 100.0);
    assert_eq!(value(&page), 100.0, "{}", page.tree());
}

#[test]
fn a_release_outside_the_slider_ends_the_drag() {
    let mut page = mount(slider);
    page.drag(TRACK, 0.0, 100.0);
    assert!(!page.exists("[data-state~=dragging]"), "{}", page.tree());
    let before = value(&page);
    // Buttons up: a later move over the track is no drag.
    page.hover(TRACK);
    assert_eq!(value(&page), before);
}

/// The wrapper sees only what bubbles through it; below the app's content the
/// pointer is over `<html>`, which only a document listener would hear.
#[test]
#[ignore = "needs Blitz: no document pointer listener reaches Rust"]
fn a_drag_leaving_the_app_keeps_moving_the_slider() {
    fn app() -> Element {
        let mut value = use_signal(|| 0.0);
        rsx! {
            Slider {
                aria_label: "Volume",
                value: Some(value()),
                oninput: move |event: SliderChangeEvent<f64>| value.set(event.value()),
                sx: sx().width("400px"),
            }
        }
    }
    let mut page = mount(app);
    page.drag(TRACK, 300.0, 400.0);
    assert_eq!(value(&page), 100.0, "{}", page.tree());
}

#[test]
fn a_drag_from_the_thumb_moves_it() {
    let mut page = mount(slider);
    page.drag(THUMB, 400.0, 0.0);
    assert_eq!(value(&page), 100.0, "{}", page.tree());
}

/// Blitz's client rect ignores `transform`, so the thumb is centred on its
/// value by layout: its rect is where it is drawn.
#[test]
fn the_thumbs_rect_is_centred_on_the_track() {
    let page = mount(slider);
    let rect = |selector| {
        page.doc
            .inner
            .borrow()
            .get_client_bounding_rect(page.node(selector))
            .expect("a layout box")
    };
    let (thumb, track) = (rect(THUMB), rect(TRACK));
    let centre = |from: f64, size: f64| from + size / 2.0;
    assert!(
        (centre(thumb.y, thumb.height) - centre(track.y, track.height)).abs() < 1.0,
        "thumb {thumb:?}, track {track:?}"
    );
    // At 0 the thumb's centre sits half a thumb in from the track's start.
    assert!(
        (centre(thumb.x, thumb.width) - (track.x + thumb.width / 2.0)).abs() < 1.0,
        "thumb {thumb:?}, track {track:?}"
    );
}

fn checked(page: &Page) -> bool {
    page.attr(CHECKBOX, "checked").is_some_and(|v| v == "true")
}

fn labelled() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Checkbox {
            checked: on(),
            onchange: move |v| on.set(v),
            label: rsx! {
                "I accept the "
                a { href: "#terms", "terms" }
            },
        }
    }
}

#[test]
fn a_link_in_a_checkbox_label_keeps_its_click() {
    let mut page = mount(labelled);
    page.click("label a");
    assert!(!checked(&page), "{}", page.tree());
    page.click("label");
    assert!(checked(&page), "the label's own text toggles it");
}

fn card() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Checkbox {
            checked: on(),
            onchange: move |v| on.set(v),
            variant: ChoiceVariant::Card,
            label: "Newsletter",
            description: rsx! {
                "Read the "
                a { href: "#terms", "terms" }
            },
        }
    }
}

#[test]
fn a_link_in_a_checkbox_card_keeps_its_click() {
    let mut page = mount(card);
    page.click("[data-state~=card] a");
    assert!(!checked(&page), "{}", page.tree());
    page.click("label");
    assert!(checked(&page), "the card's label toggles it");
}

fn radio_card() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Radio {
            checked: on(),
            onselect: move |_| on.set(true),
            variant: ChoiceVariant::Card,
            label: rsx! {
                "Express, see "
                a { href: "#rates", "rates" }
            },
        }
    }
}

#[test]
fn a_link_in_a_radio_card_keeps_its_click() {
    const RADIO: &str = "input[type=radio]";
    let picked = |page: &Page| page.attr(RADIO, "checked").is_some_and(|v| v == "true");
    let mut page = mount(radio_card);
    page.click("[data-state~=card] a");
    assert!(!picked(&page), "{}", page.tree());
    page.click("label");
    assert!(picked(&page), "the card's label picks it");
}

fn switch_card() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Switch {
            checked: on(),
            onchange: move |v| on.set(v),
            variant: ChoiceVariant::Card,
            label: "Sync",
            description: rsx! {
                "See "
                a { href: "#privacy", "privacy" }
            },
        }
    }
}

#[test]
fn a_link_in_a_switch_card_keeps_its_click() {
    const INPUT: &str = "[role=switch]";
    let on = |page: &Page| page.attr(INPUT, "checked").is_some_and(|v| v == "true");
    let mut page = mount(switch_card);
    page.click("[data-state~=card] a");
    assert!(!on(&page), "{}", page.tree());
    page.click("label");
    assert!(on(&page), "the card's label toggles it");
}
