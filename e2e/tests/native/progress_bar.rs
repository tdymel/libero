//! `ProgressBar`'s reduced-motion fill: a full bar striped by a `mask-image`.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::ProgressBar;
use libero::hooks::use_accessibility;

const BAR: &str = "#bar";
const FILL: &str = "#bar > [data-slot=fill]";

fn app() -> Element {
    let accessibility = use_accessibility();
    rsx! {
        button { id: "still", onclick: move |_| accessibility.set_reduced_motion(Some(true)) }
        ProgressBar { id: "bar", aria_label: "Loading", size: "xl" }
    }
}

/// Blitz draws the mask: the fill colour alternates with the track along the bar.
#[test]
fn the_reduced_motion_fill_is_striped_full_width() {
    let mut page = mount(app);
    page.click("#still");
    let (x, y, width, height) = page.rect(FILL);
    assert_eq!(
        width,
        page.rect(BAR).2,
        "a full bar, not the sweep's quarter"
    );

    let row = (y + height / 2.0) as u32;
    let points: Vec<_> = (x as u32..(x + width) as u32).map(|at| (at, row)).collect();
    let rgb = |[r, g, b, _]: [u8; 4]| format!("rgb({r}, {g}, {b})");
    let painted: Vec<String> = page.painted_pixels(&points).into_iter().map(rgb).collect();
    let fill = page.computed(FILL, "background-color");
    let drawn = painted.iter().filter(|pixel| **pixel == fill).count();
    let stripes = painted.windows(2).filter(|pair| pair[0] != pair[1]).count();
    assert!(
        drawn > points.len() / 4 && drawn < points.len() * 3 / 4,
        "{drawn} of {} pixels paint the fill",
        points.len()
    );
    assert!(stripes > 40, "{stripes} colour changes along the bar");
}
