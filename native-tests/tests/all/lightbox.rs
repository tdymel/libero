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
    assert_eq!(
        zoomed,
        "matrix(2, 0, 0, 2, 0, 0)",
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

#[test]
#[ignore = "Blitz: no pointer hit lands on portaled `position: fixed` content, so the drag never reaches the picture"]
fn a_drag_pans_a_zoomed_picture() {
    let mut page = mount(app);
    let zoomed = zoomed(&mut page);
    page.drag(PICTURE, 40.0, 0.0);
    page.advance(1.0);
    let panned = page.computed(PICTURE, "transform");
    assert_ne!(zoomed, panned, "the drag did not pan it");
}
