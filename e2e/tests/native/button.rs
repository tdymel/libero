//! `Button`: a pressed button paints the house on-state ring (todo 715). The
//! element-children layout is e2e's shared scenario.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{ActionIcon, Button, Flex},
    sx::sx,
};

fn pressed() -> Element {
    rsx! {
        Flex { gap: "md", sx: sx().padding("16px"),
            Button { id: "on", variant: "filled", selected: true, "Bold" }
            Button { id: "off", variant: "filled", selected: false, "Bold" }
            ActionIcon { id: "bare-on", aria_label: "Bold", selected: true, span { "B" } }
            ActionIcon { id: "bare-off", aria_label: "Bold", selected: false, span { "B" } }
        }
    }
}

/// `on` paints the ring `edge` px in from its start and top edges, `off` does not.
fn assert_ring(page: &Page, on: &str, off: &str, edge: f64) {
    // `(dx, dy)` in from the top-left corner.
    let at = |id: &str, dx: f64, dy: f64| {
        let (x, y, ..) = page.rect(id);
        ((x + dx) as u32, (y + dy) as u32)
    };
    let (_, _, w, h) = page.rect(on);
    let points = |id: &str| {
        [
            at(id, edge, h / 2.0),
            at(id, w / 2.0, edge),
            at(id, 6.0, h / 2.0),
        ]
    };
    let [left, top, fill] = points(on);
    let [off_left, off_top, off_fill] = points(off);
    let px = page.painted_pixels(&[left, top, fill, off_left, off_top, off_fill]);
    assert_ne!(px[0], px[2], "{on}: no ring at the start edge: {px:?}");
    assert_ne!(px[1], px[2], "{on}: no ring at the top edge: {px:?}");
    assert_eq!(px[3], px[5], "{off}: a ring: {px:?}");
    assert_eq!(px[4], px[5], "{off}: a ring: {px:?}");
}

/// The ring in the label's colour (todo 715): the pixel row just inside a
/// button's 1px border, the outermost one of a chromeless toggle.
#[test]
fn a_pressed_button_paints_the_on_state_ring() {
    let page = mount(pressed);
    assert_ring(&page, "#on", "#off", 1.5);
    assert_ring(&page, "#bare-on", "#bare-off", 0.5);
}
