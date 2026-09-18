//! `Lightbox`: `z` zooms the picture, then the arrows and a pointer drag pan it.

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

/// Opens the viewer and zooms its picture with `z`; returns the zoomed transform.
fn zoomed(page: &mut Page) -> String {
    page.click("#open");
    page.focus(PICTURE);
    assert_eq!(page.computed(PICTURE, "transform"), "none");
    page.press(Key::Character("z".into()));
    page.advance(1.0);
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
    page.advance(1.0);
    let panned = page.computed(PICTURE, "transform");
    assert_ne!(zoomed, panned, "ArrowLeft did not pan it");
}

/// A wheel up zooms in, as on the web: Blitz's finger sign is turned round
/// (todo 844).
#[test]
fn a_wheel_up_zooms_in() {
    let mut page = mount(app);
    page.click("#open");
    page.hover(PICTURE);
    page.wheel(PICTURE, -100.0);
    page.advance(1.0);
    let transform = page.computed(PICTURE, "transform");
    assert!(
        transform.starts_with("scale("),
        "not zoomed in: {transform}"
    );
}

/// Three pictures, so the stage is a `Carousel` that has to scroll.
fn gallery_app() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        aria_label: Some("Gallery".into()),
        ..LightboxOptions::default()
    });
    rsx! {
        Button {
            id: "open",
            onclick: move |_| {
                lightbox.open_with(
                    (1..=3)
                        .map(|i| LightboxItem::new(format!("{i}.png"), format!("Picture {i}")))
                        .collect::<Vec<_>>(),
                );
            },
            "Open"
        }
    }
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
    page.advance(1.0);
    assert_centred(&page, 0);
    page.focus(PICTURE);
    for index in 1..3 {
        page.press(Key::ArrowRight);
        page.advance(1.0);
        assert_centred(&page, index);
    }
}

#[test]
fn a_drag_pans_a_zoomed_picture() {
    let mut page = mount(app);
    let zoomed = zoomed(&mut page);
    page.drag(PICTURE, 40.0, 0.0);
    page.advance(1.0);
    let panned = page.computed(PICTURE, "transform");
    assert_ne!(zoomed, panned, "the drag did not pan it");
}
