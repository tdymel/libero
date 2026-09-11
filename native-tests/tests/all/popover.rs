//! `use_popover`: the box lands on its side of the anchor with the gap between,
//! flips when that side has no room, shifts back inside the viewport, and
//! `PopoverWidth::Match` takes the anchor's width.

use dioxus::prelude::*;
use libero::{
    components::{Box, Button},
    hooks::{Align, PopoverOptions, PopoverWidth, Side, use_element, use_popover},
};
use native_tests::{Page, VIEWPORT, mount};

const ANCHOR: &str = "#anchor";
const FLOATING: &str = "#floating";
const GAP: f64 = 8.0;

#[derive(Props, Clone, PartialEq)]
struct DemoProps {
    /// Where the anchor sits: `(margin-left, margin-top)` in pixels.
    at: (f64, f64),
    side: Side,
    align: Align,
    width: PopoverWidth,
    /// Lets the anchor's top margin collapse through `<main>`.
    #[props(default)]
    collapse: bool,
}

#[allow(non_snake_case)]
fn Demo(props: DemoProps) -> Element {
    let anchor = use_element();
    let mut opened = use_signal(|| false);
    let options = PopoverOptions::new(GAP, 0.0)
        .side(props.side)
        .align(props.align)
        .width(props.width);
    let popover = use_popover(anchor, opened(), options);
    let floating = *popover.floating();
    popover.show(opened().then(|| {
        rsx! {
            Box {
                id: "floating",
                style: popover.style(),
                onmounted: floating.mount(),
                div { width: "150px", height: "60px", "Popover content" }
            }
        }
    }));
    let (x, y) = props.at;
    let trigger = rsx! {
        div { margin_left: "{x}px", margin_top: "{y}px", width: "max-content",
            Button {
                id: "anchor",
                width: "220px",
                onmounted: anchor.mount(),
                onclick: move |_| opened.toggle(),
                "Anchor"
            }
        }
    };
    // See `a_collapsed_top_margin_shifts_the_portal_outlet`.
    if props.collapse {
        trigger
    } else {
        rsx! { div { display: "flow-root", {trigger} } }
    }
}

fn open(page: &mut Page) {
    page.click(ANCHOR);
    page.wait(std::time::Duration::from_millis(50));
    assert!(
        page.exists(FLOATING),
        "a click did not open it:\n{}",
        page.tree()
    );
}

fn close_to(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

#[test]
fn it_lands_below_the_anchor_at_its_start() {
    let mut page = mount(|| {
        rsx! { Demo { at: (200.0, 100.0), side: Side::Bottom, align: Align::Start, width: PopoverWidth::Auto } }
    });
    open(&mut page);
    let (ax, ay, _, ah) = page.rect(ANCHOR);
    let (fx, fy, ..) = page.rect(FLOATING);
    assert!(
        close_to(fx, ax) && close_to(fy, ay + ah + GAP),
        "anchor at ({ax}, {ay}) {ah} tall, box at ({fx}, {fy})"
    );
}

/// The outlet is `position: absolute` at `top: 0`, and Blitz places it against
/// `<main>`, which a first child's top margin collapsing through moves down.
#[test]
#[ignore = "needs Blitz: an absolute box is placed against its parent, not the initial containing block"]
fn a_collapsed_top_margin_shifts_the_portal_outlet() {
    let mut page = mount(|| {
        rsx! { Demo { at: (200.0, 100.0), side: Side::Bottom, align: Align::Start, width: PopoverWidth::Auto, collapse: true } }
    });
    open(&mut page);
    let (_, ay, _, ah) = page.rect(ANCHOR);
    let (_, fy, ..) = page.rect(FLOATING);
    assert!(
        close_to(fy, ay + ah + GAP),
        "anchor bottom {}, box top {fy}",
        ay + ah
    );
}

#[test]
fn it_flips_above_an_anchor_at_the_viewport_bottom() {
    let mut page = mount(|| {
        rsx! { Demo { at: (200.0, f64::from(VIEWPORT.1) - 60.0), side: Side::Bottom, align: Align::Start, width: PopoverWidth::Auto } }
    });
    open(&mut page);
    let (_, ay, ..) = page.rect(ANCHOR);
    let (_, fy, _, fh) = page.rect(FLOATING);
    assert!(
        close_to(fy + fh + GAP, ay),
        "anchor top {ay}, box {fy}..{}",
        fy + fh
    );
}

#[test]
fn it_shifts_back_inside_the_viewport() {
    let mut page = mount(|| {
        rsx! { Demo { at: (f64::from(VIEWPORT.0) - 230.0, 100.0), side: Side::Bottom, align: Align::Center, width: PopoverWidth::Auto } }
    });
    open(&mut page);
    let (fx, _, fw, _) = page.rect(FLOATING);
    assert!(
        fx >= 0.0 && fx + fw <= f64::from(VIEWPORT.0),
        "box at {fx}, {fw} wide"
    );
}

#[test]
fn a_matched_width_is_the_anchors() {
    let mut page = mount(|| {
        rsx! { Demo { at: (200.0, 100.0), side: Side::Bottom, align: Align::Start, width: PopoverWidth::Match } }
    });
    open(&mut page);
    let (_, _, aw, _) = page.rect(ANCHOR);
    let (_, _, fw, _) = page.rect(FLOATING);
    assert!(close_to(fw, aw), "anchor {aw} wide, box {fw}");
}
