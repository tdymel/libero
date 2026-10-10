//! Todo 920: Blitz draws an SVG `<img>` `contain` whatever its `object-fit`,
//! letterboxed outside its transform. libero draws it as a background instead.

use std::time::Duration;

use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use dioxus::prelude::*;
use e2e::native::{Key, Modifiers, Page, mount};
use libero::{
    components::{Avatar, Image},
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
    sx::sx,
};

const BLUE: &str = "rgb(0, 0, 255)";
const RED: &str = "rgb(255, 0, 0)";

/// A square, blue over red.
const HALVES: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100' width='100' height='100'><rect width='100' height='50' fill='blue'/><rect y='50' width='100' height='50' fill='red'/></svg>";
/// Twice as wide as tall, blue beside red.
const WIDE: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 200 100' width='200' height='100'><rect width='100' height='100' fill='blue'/><rect x='100' width='100' height='100' fill='red'/></svg>";
/// Twice as tall as wide, all blue.
const TALL: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 200' width='100' height='200'><rect width='100' height='200' fill='blue'/></svg>";

/// Pictures load at a layout; one more round draws them.
fn loaded(app: fn() -> Element) -> Page {
    let mut page = mount(app);
    page.wait(Duration::from_millis(20));
    page
}

/// The painted colour at a fraction of the first match's box.
fn painted_at(page: &Page, selector: &str, fx: f64, fy: f64) -> String {
    let (x, y, width, height) = page.rect(selector);
    page.painted_pixel((x + width * fx) as u32, (y + height * fy) as u32)
}

fn image_app() -> Element {
    rsx! {
        div { style: "width: 200px; height: 100px",
            Image { id: "cover", src: HALVES, alt: "Halves", fit: "cover" }
        }
        div { style: "width: 200px; height: 100px",
            Image { id: "backed", src: HALVES, alt: "Halves", sx: sx().background("muted.1") }
        }
        div { style: "width: 200px; height: 100px",
            Image { id: "contain", src: HALVES, alt: "Halves", fit: "contain" }
        }
    }
}

/// `cover` filled the box with the middle of the square; natively it was drawn
/// `contain`, a square with empty sides.
#[test]
fn an_svg_image_takes_its_fit() {
    let page = loaded(image_app);
    assert_eq!(painted_at(&page, "#cover", 0.05, 0.25), BLUE);
    assert_eq!(painted_at(&page, "#cover", 0.05, 0.75), RED);
    // The docs demo's `background` shorthand once reset the picture away.
    assert_eq!(painted_at(&page, "#backed", 0.05, 0.25), BLUE);
    assert_ne!(painted_at(&page, "#contain", 0.05, 0.25), BLUE);
    assert_eq!(painted_at(&page, "#contain", 0.5, 0.25), BLUE);
}

fn avatar_app() -> Element {
    rsx! {
        Avatar { id: "avatar", name: "Ada Lovelace", src: WIDE, size: "xl" }
    }
}

/// An avatar is a square that covers: a wide picture loses its sides, not
/// its top and bottom.
#[test]
fn an_svg_avatar_covers_its_circle() {
    let page = loaded(avatar_app);
    assert_eq!(painted_at(&page, "#avatar", 0.3, 0.2), BLUE);
    assert_eq!(painted_at(&page, "#avatar", 0.7, 0.8), RED);
}

fn gallery(sources: &[&str]) -> Vec<LightboxItem> {
    sources
        .iter()
        .enumerate()
        .map(|(i, src)| LightboxItem::new(*src, format!("Picture {}", i + 1)))
        .collect()
}

fn lightbox_app() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        aria_label: Some("Gallery".into()),
        ..LightboxOptions::default()
    });
    rsx! {
        button {
            id: "open",
            onclick: move |_| {
                lightbox.open_with(gallery(&[HALVES, WIDE, TALL]));
            },
            "Open"
        }
    }
}

const PICTURE: &str = "img[tabindex=\"0\"]";

fn opened() -> Page {
    let mut page = loaded(lightbox_app);
    page.click("#open");
    page.wait(Duration::from_millis(20));
    page.advance(1.0);
    page
}

/// The docs preview's strip showed each picture at its own size inside the
/// same tiles: every thumbnail covers its tile.
#[test]
fn every_svg_thumbnail_fills_its_tile() {
    let page = opened();
    for n in 1..=3 {
        let thumbnail = format!("[aria-label=\"Go to picture {n}\"] img");
        let corner = painted_at(&page, &thumbnail, 0.05, 0.05);
        assert_eq!(corner, BLUE, "thumbnail {n} is letterboxed");
    }
}

/// `scale-down` keeps the 100px square at its size and centred (Blitz drew it stage-tall);
/// zoomed 2x it stays centred rather than drifting by its letterbox.
#[test]
fn a_zoomed_svg_stays_centred_in_its_frame() {
    let mut page = opened();
    let frame = "[data-lightbox-frame=\"0\"]";
    let (x, y, width, height) = page.rect(frame);
    let (cx, cy) = (x + width / 2.0, y + height / 2.0);
    let at = |page: &Page, dx: f64, dy: f64| page.painted_pixel((cx + dx) as u32, (cy + dy) as u32);
    assert_eq!(at(&page, 0.0, -45.0), BLUE);
    assert_ne!(at(&page, 0.0, -55.0), BLUE, "grown past its natural size");

    page.focus(PICTURE);
    page.press(Key::Character("z".into()));
    page.wait(Duration::from_millis(20));
    page.advance(1.0);
    page.wait(Duration::from_millis(40));
    let row: Vec<String> = [-105.0, -95.0, 95.0, 105.0]
        .into_iter()
        .map(|dx| at(&page, dx, -50.0))
        .collect();
    assert_eq!(
        row.iter().map(|px| px == BLUE).collect::<Vec<_>>(),
        [false, true, true, false],
        "not centred at 2x: {row:?}"
    );
}

/// The frame round a shrunk picture still zooms on a double-click.
#[test]
fn a_double_click_beside_a_shrunk_svg_zooms() {
    let mut page = opened();
    let frame = "[data-lightbox-frame=\"0\"]";
    let (x, y, width, height) = page.rect(frame);
    // Between the slide control and the picture.
    let (px, py) = ((x + width * 0.3) as f32, (y + height / 2.0) as f32);
    assert!(page.hits_at(frame, px, py) && !page.hits_at(PICTURE, px, py));
    page.click_at(px, py);
    page.click_at(px, py);
    page.wait(Duration::from_millis(20));
    page.advance(1.0);
    let transform = page.computed(PICTURE, "transform");
    assert!(transform.starts_with("scale(2)"), "not at 2x: {transform}");
}

fn finger(x: f32, y: f32, down: bool) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Finger(1),
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: if down {
            MouseEventButtons::Primary
        } else {
            MouseEventButtons::empty()
        },
        mods: Modifiers::empty(),
        details: PointerDetails::default(),
        element: Point { x: 0.0, y: 0.0 },
        active_pointers: Default::default(),
    }
}

/// A swipe down that starts in the frame round a shrunk picture still drags
/// it (todo 928).
#[test]
fn a_swipe_beside_a_shrunk_svg_starts() {
    let mut page = opened();
    let frame = "[data-lightbox-frame=\"0\"]";
    let (x, y, width, height) = page.rect(frame);
    let (px, py) = ((x + width * 0.3) as f32, (y + height / 2.0) as f32);
    assert!(page.hits_at(frame, px, py) && !page.hits_at(PICTURE, px, py));
    page.dispatch(UiEvent::PointerDown(finger(px, py, true)));
    page.dispatch(UiEvent::PointerMove(finger(px, py + 40.0, true)));
    let transform = page.computed(PICTURE, "transform");
    page.dispatch(UiEvent::PointerUp(finger(px, py + 40.0, false)));
    assert!(
        transform.contains("40"),
        "the picture did not follow: {transform:?}"
    );
}
