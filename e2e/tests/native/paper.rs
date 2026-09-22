//! A `glass` Paper over wide black and white stripes (todo 812). No Blitz
//! backend draws `backdrop-filter`, so natively glass stays opaque.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::Paper;

fn app() -> Element {
    stage(None)
}

/// A glass `Paper` with a `color`: natively a solid fill, with no tint over the stripes.
fn tinted_app() -> Element {
    stage(Some("#123456"))
}

fn stage(color: Option<&'static str>) -> Element {
    rsx! {
        div { position: "relative", width: "320px", height: "120px",
            div { position: "absolute", top: "0", left: "0", right: "0", bottom: "0", display: "flex",
                for i in 0..8 {
                    div {
                        key: "{i}",
                        width: "40px",
                        background: if i % 2 == 0 { "#000" } else { "#fff" },
                    }
                }
            }
            Paper {
                id: "glass",
                glass: true,
                color: color,
                position: "absolute",
                top: "20px",
                left: "20px",
                width: "280px",
                height: "80px",
            }
        }
    }
}

/// The pixels along the middle of the Paper, clear of its shadow.
fn row(page: &Page) -> Vec<[u8; 4]> {
    let (x, y, w, h) = page.rect("#glass");
    let y = (y + h / 2.0) as u32;
    let points: Vec<(u32, u32)> = (x as u32 + 8..(x + w) as u32 - 8)
        .map(|at| (at, y))
        .collect();
    page.painted_pixels(&points)
}

#[test]
fn a_glass_paper_is_opaque_natively() {
    let page = mount(app);
    let (_, y, _, h) = page.rect("#glass");
    let beside = page.painted_pixels(&[(5, (y + h / 2.0) as u32)])[0];
    assert_eq!(beside, [0, 0, 0, 255], "no stripes to see through to");
    let px = row(&page);
    let odd = px.iter().filter(|p| **p != px[0]).count();
    assert_eq!(odd, 0, "the stripes show through at {odd} pixels");
}

/// A coloured glass has no `glass` token natively: its solid fill, no tint, sheen or blur.
#[test]
fn a_coloured_glass_paper_is_its_solid_fill_natively() {
    let page = mount(tinted_app);
    assert_eq!(
        page.attr("#glass", "data-state").as_deref(),
        Some("colored")
    );
    assert_eq!(
        page.computed("#glass", "background-color"),
        "rgb(18, 52, 86)"
    );
    assert_eq!(page.computed("#glass", "backdrop-filter"), "none");
}

/// Blitz computes the fill but paints none while the var it reads (`--lsx-paper-fill`) is
/// set inline on the same element; a var from `:root` or a parent paints. Same for gradients.
#[test]
#[ignore = "Blitz paints no background from a var set inline on the element itself (todo filed with the 1085 handback)"]
fn a_coloured_glass_paper_paints_its_fill_natively() {
    let page = mount(tinted_app);
    let px = row(&page);
    assert_eq!(px[0], [18, 52, 86, 255]);
    assert!(px.iter().all(|p| *p == px[0]), "the stripes show through");
}

/// Needs a backend that draws `backdrop-filter`, then `DRAWS_BACKDROP_FILTER`
/// turned on for Blitz.
#[test]
#[ignore = "needs Blitz: no anyrender backend draws backdrop-filter (todo 812)"]
fn a_glass_paper_blurs_what_is_behind_it() {
    let page = mount(app);
    let px = row(&page);
    assert!(px.iter().any(|p| *p != px[0]), "opaque, all {:?}", px[0]);
    let jump = px
        .windows(2)
        .map(|pair| pair[0][0].abs_diff(pair[1][0]))
        .max()
        .unwrap_or(0);
    assert!(jump < 32, "sharp stripe edges, a {jump} step");
}
