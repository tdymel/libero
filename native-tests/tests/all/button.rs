//! `Button`: element children keep the caller's flex layout (todos 662, 663),
//! and a pressed button paints the house on-state ring (todo 715).

use dioxus::prelude::*;
use libero::{
    components::{ActionIcon, Box, Button, Flex},
    sx::sx,
};
use native_tests::{Page, mount};

/// The docs search field and colour swatch, cut down as in the e2e fixture:
/// the children are the button's own flex items, with no caller sx (todo 662).
fn element_children() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", sx: sx().padding("16px"),
            Button {
                id: "search-like",
                variant: "outlined",
                sx: sx().width("240px").gap("sm").justify_content("flex-start"),
                span { id: "search-icon", display: "inline-flex", width: "16px", height: "16px" }
                "Search"
                span { id: "search-kbd", display: "inline-block", width: "24px", height: "12px", margin_left: "auto" }
            }
            Button {
                id: "swatch",
                variant: "standard",
                aria_label: "Blue",
                sx: sx().width("48px").padding("0"),
                Box { id: "swatch-fill", sx: sx().width("100%").height("100%").background("blue") }
            }
            Button { id: "with-icon",
                icon: rsx! { span { id: "glyph", display: "inline-block", width: "16px", height: "16px" } },
                span { id: "with-icon-text", "Save" }
            }
        }
    }
}

#[test]
fn element_children_keep_the_callers_flex_layout() {
    let page = mount(element_children);
    let (sx, _, sw, _) = page.rect("#search-like");
    let pad: f64 = page
        .computed("#search-like", "padding-right")
        .trim_end_matches("px")
        .parse()
        .unwrap();
    let (kx, _, kw, _) = page.rect("#search-kbd");
    let (ix, ..) = page.rect("#search-icon");
    assert!(
        (sx + sw - pad - 1.0 - (kx + kw)).abs() <= 1.0,
        "kbd not at the end: kbd right {} vs button right {}\n{}",
        kx + kw,
        sx + sw,
        page.tree()
    );
    assert!(ix - sx <= 24.0, "icon not at the start: {ix} vs {sx}");

    let (_, _, w, h) = page.rect("#swatch");
    let (_, _, fw, fh) = page.rect("#swatch-fill");
    assert!(
        fw >= w - 2.0 && fh >= h - 2.0,
        "swatch fill collapsed: {fw}x{fh} in {w}x{h}\n{}",
        page.tree()
    );

    // The `icon` slot: whole, a gap before the text, on its centre line.
    let (gx, gy, gw, gh) = page.rect("#glyph");
    let (tx, ty, _, th) = page.rect("#with-icon-text");
    assert!(gw >= 15.0, "icon squeezed: {gw}\n{}", page.tree());
    assert!(tx - (gx + gw) >= 2.0, "no gap: {} to {tx}", gx + gw);
    assert!(
        ((gy + gh / 2.0) - (ty + th / 2.0)).abs() <= 1.5,
        "icon off the text centre: {gy}+{gh} vs {ty}+{th}"
    );
}

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
