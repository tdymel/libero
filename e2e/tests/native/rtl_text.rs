//! Blitz aligns `start` and `end` as left and right whatever the direction,
//! so libero spells them out physically under `dir=rtl` (todo 734).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::Textarea;

fn app() -> Element {
    rsx! { Fixture { dir: "rtl" } }
}

fn ltr_app() -> Element {
    rsx! { Fixture { dir: "ltr" } }
}

#[component]
fn Fixture(dir: &'static str) -> Element {
    rsx! {
        div { dir, style: "width: 600px",
            p { id: "paragraph", "Short line" }
            h2 { id: "heading", "Short line" }
            p { id: "centred", style: "text-align: center", "Short line" }
            Textarea { counter: true, maxlength: 20 }
        }
    }
}

/// Dark pixels in the first and the last 40px of the element's middle row.
fn ink(page: &Page, selector: &str) -> (usize, usize) {
    let (left, top, width, height) = page.rect(selector);
    let y = (top + height / 2.0) as u32;
    let row = |from: f64| (0..40).map(|x| (from as u32 + x, y)).collect::<Vec<_>>();
    let dark = |pixels: Vec<[u8; 4]>| pixels.iter().filter(|p| p[0] < 128).count();
    (
        dark(page.painted_pixels(&row(left))),
        dark(page.painted_pixels(&row(left + width - 40.0))),
    )
}

#[test]
fn rtl_text_sits_right() {
    let page = mount(app);
    for selector in ["#paragraph", "#heading"] {
        let (left, right) = ink(&page, selector);
        assert!(
            left == 0 && right > 0,
            "{selector}: ink left={left} right={right}"
        );
    }
}

/// Whether the counter's box is painted, and its horizontal centre relative to the textarea's.
fn counter_side(page: &Page) -> (bool, f64) {
    let (left, top, width, height) = page.rect("[data-slot=counter]");
    let (frame_left, _, frame_width, _) = page.rect("textarea");
    let y = (top + height / 2.0) as u32;
    let row = (0..width as u32)
        .map(|x| (left as u32 + x, y))
        .collect::<Vec<_>>();
    let inked = page.painted_pixels(&row).iter().any(|p| p[0] < 128);
    (inked, left + width / 2.0 - (frame_left + frame_width / 2.0))
}

#[test]
fn a_counter_at_the_end_sits_left_under_rtl() {
    let (inked, offset) = counter_side(&mount(app));
    assert!(inked && offset < 0.0, "inked={inked} offset={offset}");
}

#[test]
fn a_counter_at_the_end_sits_right_under_ltr() {
    let (inked, offset) = counter_side(&mount(ltr_app));
    assert!(inked && offset > 0.0, "inked={inked} offset={offset}");
}

#[test]
fn an_explicit_centre_stays_centred_under_rtl() {
    let page = mount(app);
    let (left, right) = ink(&page, "#centred");
    assert!(left == 0 && right == 0, "ink left={left} right={right}");
}
