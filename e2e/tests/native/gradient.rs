//! Gradient fills natively (todo 937): Blitz paints `linear-gradient`
//! backgrounds, through a `var()` too, but not `background-clip: text`.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Button, Text};
use libero::sx::sx;
use libero::theme::Gradient;

fn app() -> Element {
    rsx! {
        div { id: "plain", style: "width: 200px; height: 40px; background-color: #f00; background-image: linear-gradient(90deg, #000 0%, #fff 100%);" }
        div { id: "var", style: "--g: linear-gradient(90deg, #00f 0%, #0f0 100%); width: 200px; height: 40px; background-image: var(--g);" }
        div { id: "clip", style: "width: 200px; height: 40px; color: transparent; background-clip: text; background-image: linear-gradient(90deg, #000 0%, #000 100%);", "Gradient text" }
    }
}

/// The painted colour at 5%, 50% and 95% across `selector`, mid-height.
fn samples(page: &mut Page, selector: &str) -> Vec<String> {
    let (left, top, width, height) = page.rect(selector);
    let y = (top + height / 2.0) as u32;
    [0.05, 0.5, 0.95]
        .iter()
        .map(|f| page.painted_pixel((left + width * f) as u32, y))
        .collect()
}

#[test]
fn a_linear_gradient_is_painted_in_srgb() {
    let mut page = mount(app);
    page.settle();
    assert_eq!(
        samples(&mut page, "#plain"),
        [
            "rgb(13, 13, 13)",
            "rgb(128, 128, 128)",
            "rgb(242, 242, 242)"
        ]
    );
    assert_eq!(
        samples(&mut page, "#var"),
        ["rgb(0, 13, 242)", "rgb(0, 128, 127)", "rgb(0, 242, 13)"]
    );
}

/// Blitz clips the image to the box, not the glyphs: the whole box paints.
#[test]
fn text_clip_paints_the_whole_box() {
    let mut page = mount(app);
    page.settle();
    assert_eq!(samples(&mut page, "#clip"), ["rgb(0, 0, 0)"; 3]);
}

fn libero_app() -> Element {
    rsx! {
        Button { id: "button", variant: "gradient", sx: sx().width("200px"), "Gradient" }
        Text { id: "text", gradient: Gradient::default(), "Gradient text" }
    }
}

/// A gradient `Button` runs from one stop to the other, and a hover lays the
/// state layer over it, `color-mix` and all.
#[test]
fn a_gradient_button_paints_its_stops_and_hover_layer() {
    let mut page = mount(libero_app);
    page.settle();
    let rest = samples(&mut page, "#button");
    assert_ne!(rest[0], rest[2], "one flat colour: {rest:?}");
    let (left, top, width, height) = page.rect("#button");
    page.move_to((left + width / 2.0) as f32, (top + height / 2.0) as f32);
    page.settle();
    let hovered = samples(&mut page, "#button");
    assert_ne!(rest[0], hovered[0], "no hover layer: {hovered:?}");
}

/// Natively gradient text is its first stop, solid: never transparent glyphs
/// on a painted box.
#[test]
fn gradient_text_is_solid_natively() {
    let mut page = mount(libero_app);
    page.settle();
    let (left, top, width, height) = page.rect("#text");
    let y = (top + height / 2.0) as u32;
    // The light page, past the end of the text.
    let right = page.painted_pixel((left + width - 2.0) as u32, y);
    assert_eq!(
        right, "rgb(255, 255, 255)",
        "the box behind the text is painted"
    );
    let row: Vec<(u32, u32)> = (0..120).map(|dx| (left as u32 + dx, y)).collect();
    let inked = page
        .painted_pixels(&row)
        .iter()
        .filter(|px| **px != [255, 255, 255, 255])
        .count();
    assert!(inked > 0, "no glyph drawn");
}
