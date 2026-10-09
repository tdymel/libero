//! `CircularProgress`'s painted arc: Blitz bakes the inline svg from its attributes.

use std::f64::consts::FRAC_1_SQRT_2;

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::CircularProgress;

const RING: &str = "#ring";

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
