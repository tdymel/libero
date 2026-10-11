//! `AspectRatio` in Blitz: a focused child's ring rides an `::after` overlay over its
//! picture (todo 2480), as on the web (todo 2522).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::AspectRatio;

fn linked_picture() -> Element {
    rsx! {
        div { style: "width: 320px",
            AspectRatio { ratio: 4.0 / 3.0,
                a { id: "link", href: "#here",
                    div { style: "width: 100%; height: 100%; background: rgb(51, 102, 153)" }
                }
            }
        }
    }
}

/// The colour 1px inside the start edge, and at the centre, of `selector`.
pub fn edge_and_centre(page: &Page, selector: &str) -> (String, String) {
    let (x, y, width, height) = page.rect(selector);
    let middle = (y + height / 2.0) as u32;
    (
        page.painted_pixel((x + 1.0) as u32, middle),
        page.painted_pixel((x + width / 2.0) as u32, middle),
    )
}

/// Blue all over.
const BLUE_SQUARE: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10' width='10' height='10'><rect width='10' height='10' fill='blue'/></svg>";

fn rounded_picture() -> Element {
    rsx! {
        div { style: "width: 200px",
            AspectRatio { id: "ratio", ratio: 1.0, style: "border-radius: 24px",
                img { id: "cover", src: BLUE_SQUARE, alt: "A blue field" }
            }
        }
    }
}

/// Todo 3239: the picture clips itself to the box's radius, as on the web.
#[test]
fn a_picture_is_cut_to_the_rounded_corners_of_the_box() {
    let mut page = mount(rounded_picture);
    assert!(
        page.wait_for(|page| {
            let (x, y, width, height) = page.rect("#cover");
            page.painted_pixel((x + width / 2.0) as u32, (y + height / 2.0) as u32)
                == "rgb(0, 0, 255)"
        }),
        "the picture never painted: {}",
        page.tree()
    );
    let (x, y, width, _) = page.rect("#cover");
    let corner = page.painted_pixel((x + 1.0) as u32, (y + 1.0) as u32);
    assert_ne!(corner, "rgb(0, 0, 255)", "the corner is not cut");
    let edge = page.painted_pixel((x + width / 2.0) as u32, (y + 1.0) as u32);
    assert_eq!(edge, "rgb(0, 0, 255)", "the straight edge is cut");
}

#[test]
fn a_focused_link_paints_its_ring_over_its_picture() {
    let mut page = mount(linked_picture);
    let (edge, centre) = edge_and_centre(&page, "#link");
    assert_eq!(edge, centre, "the picture does not fill the link");

    page.tab();
    assert!(
        page.is_focused("#link"),
        "Tab reached {}",
        page.focus_owner()
    );
    let (edge, centre) = edge_and_centre(&page, "#link");
    assert_ne!(edge, centre, "no ring over the picture: {}", page.tree());
}
