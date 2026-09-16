//! `Button`: a caller's unwrapped label keeps its flex layout (todo 663), and
//! a pressed button paints the house on-state line (todo 646).

use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex},
    sx::sx,
};
use native_tests::mount;

/// The docs search field and colour swatch, cut down as in the e2e fixture:
/// the label span unwrapped with `display: contents` (f3064f24).
fn unwrapped() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", sx: sx().padding("16px"),
            Button {
                id: "search-like",
                variant: "outlined",
                sx: sx()
                    .width("240px")
                    .gap("sm")
                    .justify_content("flex-start")
                    .selector("& > [data-slot='label']", sx().display("contents")),
                span { id: "search-icon", display: "inline-flex", width: "16px", height: "16px" }
                "Search"
                span { id: "search-kbd", display: "inline-block", width: "24px", height: "12px", margin_left: "auto" }
            }
            Button {
                id: "swatch",
                variant: "standard",
                aria_label: "Blue",
                sx: sx()
                    .width("48px")
                    .padding("0")
                    .selector("& > [data-slot='label']", sx().display("contents")),
                Box { id: "swatch-fill", sx: sx().width("100%").height("100%").background("blue") }
            }
        }
    }
}

#[test]
fn an_unwrapped_label_keeps_the_callers_flex_layout() {
    let page = mount(unwrapped);
    assert_eq!(
        page.computed("#search-like > [data-slot='label']", "display"),
        "contents"
    );
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
}

fn pressed() -> Element {
    rsx! {
        Flex { gap: "md", sx: sx().padding("16px"),
            Button { id: "on", variant: "filled", selected: true, "Bold" }
            Button { id: "off", variant: "filled", selected: false, "Bold" }
        }
    }
}

/// The line sits 3px above the bottom edge, centred, in the label's colour.
#[test]
fn a_pressed_button_paints_the_on_state_line() {
    let page = mount(pressed);
    // The bar's lower row: 1px border, 3px gap, 2px line up from the bottom.
    let at = |id: &str, share: f64| {
        let (x, y, w, h) = page.rect(id);
        ((x + w * share) as u32, (y + h - 5.0) as u32)
    };
    let (on, off) = (at("#on", 0.5), at("#off", 0.5));
    let fill = |id: &str| at(id, 0.2);
    let px = page.painted_pixels(&[on, fill("#on"), off, fill("#off")]);
    assert_ne!(px[0], px[1], "no line under the pressed label: {px:?}");
    assert_eq!(px[2], px[3], "a line under the unpressed one: {px:?}");
}
