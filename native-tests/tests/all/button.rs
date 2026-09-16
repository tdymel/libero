//! `Button`: element children keep the caller's flex layout (todos 662, 663),
//! and a pressed button paints the house on-state line (todo 646).

use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex},
    sx::sx,
};
use native_tests::mount;

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
