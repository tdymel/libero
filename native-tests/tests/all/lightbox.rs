//! `Lightbox`: `z` zooms the picture, then the arrows and a pointer drag pan it.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::Button,
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
};
use native_tests::{Key, Page, mount};

const PICTURE: &str = "img[tabindex]";

fn app() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        aria_label: Some("Gallery".into()),
        ..LightboxOptions::default()
    });
    rsx! {
        Button {
            id: "open",
            onclick: move |_| {
                lightbox.open_with(vec![LightboxItem::new("picture.png", "A picture")]);
            },
            "Open"
        }
    }
}

/// A zoom lands after its measure, a timer on an OS thread: waited out before
/// the clock jumps, and libero's frame heal after it (todo 870).
fn settle(page: &mut Page) {
    page.wait(Duration::from_millis(20));
    page.advance(1.0);
    page.wait(Duration::from_millis(40));
}

/// Opens the viewer and zooms its picture with `z`; returns the zoomed transform.
fn zoomed(page: &mut Page) -> String {
    page.click("#open");
    // After the dialog's own first focus, which a timer moves.
    page.wait(Duration::from_millis(20));
    page.focus(PICTURE);
    assert_eq!(page.computed(PICTURE, "transform"), "none");
    page.press(Key::Character("z".into()));
    settle(page);
    let zoomed = page.computed(PICTURE, "transform");
    // Stylo's computed value: the web's `getComputedStyle` would say `matrix(...)`.
    assert_eq!(
        zoomed,
        "scale(2) translate(0px)",
        "z did not zoom:\n{}",
        page.tree()
    );
    zoomed
}

#[test]
fn z_zooms_and_an_arrow_pans() {
    let mut page = mount(app);
    let zoomed = zoomed(&mut page);
    page.press(Key::ArrowLeft);
    settle(&mut page);
    let panned = page.computed(PICTURE, "transform");
    assert_ne!(zoomed, panned, "ArrowLeft did not pan it");
}

/// Todo 864: `z` steps on to 4x and 8x, then back to fit.
#[test]
fn z_steps_to_max_zoom_then_fits() {
    let mut page = mount(app);
    zoomed(&mut page);
    for scale in ["scale(4)", "scale(8)", "none"] {
        page.press(Key::Character("z".into()));
        settle(&mut page);
        let transform = page.computed(PICTURE, "transform");
        assert!(transform.starts_with(scale), "not {scale}: {transform}");
    }
}

/// Todo 864: the toolbar zooms in and out, and a button at its limit is
/// `aria-disabled` and keeps the focus.
#[test]
fn the_zoom_buttons_step_and_disable_at_their_limits() {
    const ZOOM_IN: &str = "button[aria-label=\"Zoom in\"]";
    const ZOOM_OUT: &str = "button[aria-label=\"Zoom out\"]";
    let mut page = mount(app);
    page.click("#open");
    settle(&mut page);
    assert_eq!(
        page.attr(ZOOM_OUT, "aria-disabled").as_deref(),
        Some("true")
    );

    page.click(ZOOM_IN);
    settle(&mut page);
    // Todo 870: the transition used to stay at its start, `scale(1)`.
    let transform = page.computed(PICTURE, "transform");
    assert!(
        transform.starts_with("scale(1.25)"),
        "not zoomed in: {transform}"
    );
    assert_eq!(page.attr(ZOOM_OUT, "aria-disabled"), None);

    page.focus(ZOOM_IN);
    for _ in 0..10 {
        page.press(Key::Enter);
        settle(&mut page);
    }
    let transform = page.computed(PICTURE, "transform");
    assert!(transform.starts_with("scale(8)"), "not at 8x: {transform}");
    assert_eq!(page.attr(ZOOM_IN, "aria-disabled").as_deref(), Some("true"));
    assert!(
        page.is_focused(ZOOM_IN),
        "focus left: {}",
        page.focus_owner()
    );

    page.focus(ZOOM_OUT);
    for _ in 0..12 {
        page.press(Key::Enter);
        settle(&mut page);
    }
    assert_eq!(page.computed(PICTURE, "transform"), "none");
    assert_eq!(
        page.attr(ZOOM_OUT, "aria-disabled").as_deref(),
        Some("true")
    );
    assert!(
        page.is_focused(ZOOM_OUT),
        "focus left: {}",
        page.focus_owner()
    );
}

/// Todo 916: a double-click steps to 2x.
#[test]
fn a_double_click_zooms_in() {
    let mut page = mount(app);
    page.click("#open");
    settle(&mut page);
    let (x, y, width, height) = page.rect(PICTURE);
    let (x, y) = ((x + width / 2.0) as f32, (y + height / 2.0) as f32);
    page.click_at(x, y);
    page.click_at(x, y);
    settle(&mut page);
    let transform = page.computed(PICTURE, "transform");
    assert!(transform.starts_with("scale(2)"), "not at 2x: {transform}");
}

/// A wheel up zooms in, as on the web: Blitz's finger sign is turned round
/// (todo 844).
#[test]
fn a_wheel_up_zooms_in() {
    let mut page = mount(app);
    page.click("#open");
    page.hover(PICTURE);
    page.wheel(PICTURE, -100.0);
    settle(&mut page);
    let transform = page.computed(PICTURE, "transform");
    assert!(
        transform.starts_with("scale("),
        "not zoomed in: {transform}"
    );
}

/// Three pictures, so the stage is a `Carousel` that has to scroll. `#open-2`
/// opens on the second.
fn gallery_app() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        aria_label: Some("Gallery".into()),
        ..LightboxOptions::default()
    });
    let gallery = || {
        (1..=3)
            .map(|i| {
                LightboxItem::new(format!("{i}.png"), format!("Picture {i}"))
                    .caption(format!("Caption {i}"))
            })
            .collect::<Vec<_>>()
    };
    rsx! {
        Button {
            id: "open",
            onclick: move |_| {
                lightbox.open_with(gallery());
            },
            "Open"
        }
        Button {
            id: "open-2",
            onclick: move |_| {
                lightbox.open_with((gallery(), 1));
            },
            "Open the second"
        }
    }
}

/// Todo 916: opened on its second picture, the stage stayed on the first, so
/// every zoom went to a picture out of sight.
#[test]
fn a_gallery_opened_on_a_later_picture_shows_it() {
    let mut page = mount(gallery_app);
    // As a window does: the vdom runs dry before the dialog is laid out.
    page.click_before_layout("#open-2");
    settle(&mut page);
    assert_centred(&page, 1);
}

/// Horizontal offset of `selector`'s centre from the viewport's.
fn off_centre(page: &Page, selector: &str) -> f64 {
    let (x, _, width, _) = page.rect(selector);
    x + width / 2.0 - native_tests::VIEWPORT.0 as f64 / 2.0
}

fn assert_centred(page: &Page, index: usize) {
    let frame = off_centre(page, &format!("[data-lightbox-frame=\"{index}\"]"));
    let dialog = off_centre(page, "[role=dialog]");
    assert!(
        dialog.abs() <= 1.0 && frame.abs() <= 1.0,
        "picture {index} off centre by {frame}px, the dialog by {dialog}px"
    );
}

/// Todo 652: Blitz's `scroll_size` was the overflow alone, so the stage
/// scrolled only part of its range. Picture 2 of 3 sat 453px right of centre,
/// and the last one could not be reached.
#[test]
fn every_picture_comes_to_rest_centred() {
    let mut page = mount(gallery_app);
    page.click("#open");
    settle(&mut page);
    assert_centred(&page, 0);
    page.focus(PICTURE);
    for index in 1..3 {
        page.press(Key::ArrowRight);
        settle(&mut page);
        assert_centred(&page, index);
    }
}

/// Todo 916: Blitz hit-tests a zoomed picture past its frame's clip, so the
/// second press on "Zoom in" panned the picture instead. The caption and the
/// strip come after the stage and keep theirs.
#[test]
fn a_zoomed_picture_leaves_the_presses_round_it_alone() {
    const ZOOM_IN: &str = "button[aria-label=\"Zoom in\"]";
    const ZOOMED: &str = "scale(1.5625) translate(0px)";
    let mut page = mount(gallery_app);
    page.click("#open");
    settle(&mut page);
    for scale in ["scale(1.25) translate(0px)", ZOOMED] {
        page.click(ZOOM_IN);
        settle(&mut page);
        assert_eq!(page.computed(PICTURE, "transform"), scale);
    }
    page.click("[role=dialog] p");
    settle(&mut page);
    assert_eq!(page.computed(PICTURE, "transform"), ZOOMED);
    page.click("[aria-label=\"Go to slide 2\"]");
    settle(&mut page);
    assert_centred(&page, 1);
}

#[test]
fn a_drag_pans_a_zoomed_picture() {
    let mut page = mount(app);
    let zoomed = zoomed(&mut page);
    page.drag(PICTURE, 40.0, 0.0);
    settle(&mut page);
    let panned = page.computed(PICTURE, "transform");
    assert_ne!(zoomed, panned, "the drag did not pan it");
}
