//! `use_popover` against Blitz's own layout: the portal outlet is in the
//! document's flow and placed against `<main>`. Placement, flip, shift and
//! matched width are e2e's shared scenarios (`popover::`).

use dioxus::prelude::*;
use libero::{
    components::{Box, Button},
    hooks::{Align, PopoverOptions, PopoverWidth, Side, use_element, use_popover},
};
use native_tests::{Key, Page, mount};

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
    /// Makes the document taller than the viewport, so the root scrolls.
    #[props(default)]
    tall: bool,
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
            if props.tall {
                div { height: "3000px" }
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

/// The outlet is `position: absolute`, and Blitz places it against `<main>`,
/// which a first child's top margin collapsing through moves down: libero
/// shifts it back.
#[test]
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

/// The outlet is in the document's flow, so a scrolled root moves it too.
#[test]
fn it_lands_below_the_anchor_in_a_scrolled_document() {
    let mut page = mount(|| {
        rsx! { Demo { at: (200.0, 500.0), side: Side::Bottom, align: Align::Start, width: PopoverWidth::Auto, tall: true } }
    });
    page.hover(ANCHOR);
    page.wheel(ANCHOR, 300.0);
    let (_, ay, _, ah) = page.rect(ANCHOR);
    assert!(
        close_to(ay, 200.0),
        "the root did not scroll: anchor at {ay}"
    );
    // The harness's pointer has no page offset, so a key opens it.
    page.focus(ANCHOR);
    page.press(Key::Enter);
    page.wait(std::time::Duration::from_millis(50));
    let (_, ay, _, ah2) = page.rect(ANCHOR);
    let (_, fy, ..) = page.rect(FLOATING);
    assert!(
        close_to(ah, ah2) && close_to(fy, ay + ah + GAP),
        "anchor bottom {}, box top {fy}",
        ay + ah
    );
}

#[test]
fn it_follows_the_anchor_when_the_document_scrolls_while_open() {
    let mut page = mount(|| {
        rsx! { Demo { at: (200.0, 500.0), side: Side::Bottom, align: Align::Start, width: PopoverWidth::Auto, tall: true } }
    });
    open(&mut page);
    page.hover(ANCHOR);
    page.wheel(ANCHOR, 200.0);
    page.wait(std::time::Duration::from_millis(50));
    let (_, ay, _, ah) = page.rect(ANCHOR);
    let (_, fy, ..) = page.rect(FLOATING);
    assert!(
        close_to(ay, 300.0),
        "the root did not scroll: anchor at {ay}"
    );
    assert!(
        close_to(fy, ay + ah + GAP),
        "anchor bottom {}, box top {fy}",
        ay + ah
    );
}
