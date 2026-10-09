//! `CircularProgress`'s painted arc: Blitz bakes the inline svg from its attributes.

use std::f64::consts::{FRAC_1_SQRT_2, TAU};

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::CircularProgress;
use libero::hooks::use_accessibility;

const RING: &str = "#ring";
const SAMPLES: u32 = 120;

/// A quarter of the `md` edge is 9px: the label stays at the 12px floor.
#[test]
fn the_label_keeps_to_the_12px_floor_natively() {
    fn app() -> Element {
        rsx! {
            CircularProgress { id: "ring", aria_label: "Upload", value: 42.0, "42%" }
        }
    }
    let page = mount(app);
    assert_eq!(
        page.computed("#ring > [data-slot=label]", "font-size"),
        "12px"
    );
}

/// At 200% text (a 32px root) the `md` ring grows with its label (WCAG 1.4.4): "42%" stays inside.
#[test]
fn the_md_label_fits_the_ring_at_200_percent_text() {
    fn app() -> Element {
        rsx! {
            style { "html {{ font-size: 32px }}" }
            CircularProgress { id: "ring", aria_label: "Upload", value: 42.0, "42%" }
        }
    }
    let page = mount(app);
    let (_, _, edge, _) = page.rect(RING);
    let (_, _, label, _) = page.rect("#ring > [data-slot=label]");
    assert!((edge - 72.0).abs() < 0.5, "the ring is {edge}px");
    assert_eq!(
        page.computed("#ring > [data-slot=label]", "font-size"),
        "24px"
    );
    assert!(label < 0.8 * edge, "the label is {label}px in {edge}px");
}

/// A frozen quarter would read as 25% done: still, the unknown amount is a dashed ring
/// all the way round. Blitz bakes the dashes from the svg attributes, not the CSS.
#[test]
fn reduced_motion_dashes_the_ring_all_round() {
    fn app() -> Element {
        let accessibility = use_accessibility();
        rsx! {
            button { id: "still", onclick: move |_| accessibility.set_reduced_motion(Some(true)) }
            CircularProgress { id: "ring", aria_label: "Loading", size: "xxl" }
        }
    }
    let mut page = mount(app);
    page.click("#still");
    let arc = format!("{RING} > svg");
    let (x, y, width, _) = page.rect(RING);
    let (cx, cy) = (x + width / 2.0, y + width / 2.0);
    // The ring's middle line, at 45% of the edge for the `md` thickness of 10%.
    let points: Vec<(u32, u32)> = (0..SAMPLES)
        .map(|i| {
            let angle = f64::from(i) * TAU / f64::from(SAMPLES);
            let at = 0.45 * width;
            (
                (cx + at * angle.sin()) as u32,
                (cy - at * angle.cos()) as u32,
            )
        })
        .collect();
    let colour = page.computed(&arc, "color");
    let drawn = page
        .painted_pixels(&points)
        .into_iter()
        .filter(|&[r, g, b, _]| format!("rgb({r}, {g}, {b})") == colour)
        .count();
    let share = drawn as f64 / f64::from(SAMPLES);
    assert!(
        (0.4..0.8).contains(&share),
        "{drawn} of {SAMPLES} samples paint the arc"
    );
}

/// A quarter drawn clockwise from the top: its middle paints the arc colour, the
/// opposite side only the track.
#[test]
fn a_quarter_paints_the_arc_on_its_track() {
    fn app() -> Element {
        rsx! {
            CircularProgress { id: "ring", aria_label: "Upload", value: 25.0, size: "xxl", color: "error" }
        }
    }
    let page = mount(app);
    let arc = format!("{RING} > svg");
    assert_eq!(page.painted_stroke(&arc), page.computed(&arc, "color"));

    let (x, y, width, _) = page.rect(RING);
    let (cx, cy) = (x + width / 2.0, y + width / 2.0);
    // The ring's middle line, at 45% of the edge for the `md` thickness of 10%.
    let at = 0.45 * width * FRAC_1_SQRT_2;
    let pixels = page.painted_pixels(&[
        ((cx + at) as u32, (cy - at) as u32),
        ((cx - at) as u32, (cy + at) as u32),
    ]);
    let [drawn, undrawn] = [pixels[0], pixels[1]];
    let rgb = |[r, g, b, _]: [u8; 4]| format!("rgb({r}, {g}, {b})");
    assert_eq!(rgb(drawn), page.computed(&arc, "color"), "half past one");
    assert_ne!(rgb(undrawn), rgb(drawn), "half past seven paints the arc");
    assert_ne!(
        rgb(undrawn),
        "rgb(255, 255, 255)",
        "half past seven paints no track"
    );
}
