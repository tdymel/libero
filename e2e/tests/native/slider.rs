//! `Slider` under Blitz's pointer: a fast drag, the bubble over the hit area, no text selection.
//! Thumb and leaving drags are in `pointer.rs`; track drag and End are shared scenarios.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{RangeSlider, Slider, SliderChangeEvent},
    sx::sx,
    theme::Size,
};

const THUMB: &str = "[role=slider]";
const TRACK: &str = "[data-state~=size-md] > [data-state~=size-md] > div";

fn app() -> Element {
    let mut value = use_signal(|| 0.0);
    rsx! {
        Slider {
            aria_label: "Volume",
            value: Some(value()),
            oninput: move |event: SliderChangeEvent<f64>| value.set(event.value()),
            sx: sx().width("400px"),
        }
    }
}

/// The thumb sits under the pointer while the drag is on, with no transition
/// easing it there: the animation clock is never advanced here.
#[test]
fn the_thumb_keeps_up_with_a_fast_drag() {
    let mut page = mount(app);
    let (tx, ty, tw, th) = page.rect(TRACK);
    let y = (ty + th / 2.0) as f32;
    let x0 = (tx + tw / 2.0) as f32;
    page.press_at(x0, y);
    for step in 1..=4u8 {
        page.move_to(x0 + 40.0 * f32::from(step), y);
    }
    let now: f64 = page
        .attr(THUMB, "aria-valuenow")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    let (thumb_x, _, thumb_w, _) = page.rect(THUMB);
    let centre = thumb_x + thumb_w / 2.0;
    let pointer_x = f64::from(x0 + 160.0);
    page.release_at(x0 + 160.0, y);
    assert!(
        (centre - pointer_x).abs() <= 12.0,
        "value {now}, thumb centre {centre}, pointer {pointer_x}\n{}",
        page.tree()
    );
}

const BUBBLE: &str = "[role=tooltip]:not([hidden])";

/// Room above for the bubble, text on both sides for a selection to land in.
fn between_text() -> Element {
    rsx! {
        p { id: "above", "Lorem ipsum dolor sit amet, consectetur adipiscing elit" }
        div { height: "40px" }
        SizedSlider { size: Size::Xs }
        SizedSlider { size: Size::Sm }
        SizedSlider { size: Size::Md }
        p { id: "below", "Ut enim ad minim veniam, quis nostrud exercitation" }
    }
}

#[component]
fn SizedSlider(size: Size) -> Element {
    let mut value = use_signal(|| 50.0);
    // Padding, not a margin: a move over a margin hits no app box and is lost
    // (see `pointer.rs`).
    rsx! {
        div { padding_bottom: "40px",
            Slider {
                aria_label: "Volume",
                size,
                value: Some(value()),
                oninput: move |event: SliderChangeEvent<f64>| value.set(event.value()),
                sx: sx().width("400px"),
            }
        }
    }
}

fn value_of(page: &Page, thumb: &str) -> f64 {
    page.attr(thumb, "aria-valuenow")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default()
}

/// Todo 649: the open bubble sat 4px above the thumb, whose 24px hit area
/// overhangs a 16px thumb by 4px: the bubble covered its top row.
#[test]
fn the_open_bubble_leaves_the_thumbs_whole_hit_area_to_it() {
    let mut page = mount(between_text);
    for thumb in [
        "[data-state~=size-xs] [role=slider]",
        "[data-state~=size-sm] [role=slider]",
        "[data-state~=size-md] [role=slider]",
    ] {
        page.hover(thumb);
        assert!(page.exists(BUBBLE), "hovering {thumb} opened no bubble");
        let (x, y, w, h) = page.rect(thumb);
        let (cx, cy) = ((x + w / 2.0) as f32, (y + h / 2.0) as f32);
        let (_, bubble_y, _, bubble_h) = page.rect(BUBBLE);
        let top = cy - 12.0;
        assert!(
            bubble_y + bubble_h <= f64::from(top),
            "{thumb}'s bubble ends at {}, its hit area starts at {top}",
            bubble_y + bubble_h
        );
        // The hit area's top row, just inside it.
        let top = top + 0.5;
        assert!(
            page.hits_at(thumb, cx, top),
            "{thumb}'s hit area top took no hit"
        );

        let before = value_of(&page, thumb);
        page.press_at(cx, top);
        page.move_to(cx + 40.0, top);
        page.release_at(cx + 40.0, top);
        assert!(
            value_of(&page, thumb) > before,
            "a press there did not drag {thumb}: {before} -> {}\n{}",
            value_of(&page, thumb),
            page.tree()
        );
    }
}

/// Todo 650: a drag moves the value, it never selects the text it crosses.
#[test]
fn a_drag_over_text_selects_none_of_it() {
    const SELECTED: [u8; 4] = [180, 213, 255, 255];
    let mut page = mount(between_text);
    let thumb = "[data-state~=size-md] [role=slider]";
    let rows = |page: &Page| {
        let mut points = Vec::new();
        for text in ["#above", "#below"] {
            let (x, y, w, h) = page.rect(text);
            points.extend(
                (0..16).map(|i| ((x + 2.0 + i as f64 * w / 16.0) as u32, (y + h / 2.0) as u32)),
            );
        }
        page.painted_pixels(&points)
            .into_iter()
            .filter(|p| *p == SELECTED)
            .count()
    };

    // The harness does see a selection: a press and drag on the text makes one.
    {
        let mut page = mount(between_text);
        let (x, y, _, h) = page.rect("#below");
        let y = (y + h / 2.0) as f32;
        page.press_at(x as f32 + 2.0, y);
        page.move_to(x as f32 + 200.0, y);
        page.release_at(x as f32 + 200.0, y);
        assert!(rows(&page) > 0, "a text drag painted no selection");
    }

    for text in ["#above", "#below"] {
        let (x, y, w, h) = page.rect(thumb);
        let (cx, cy) = ((x + w / 2.0) as f32, (y + h / 2.0) as f32);
        let (tx, ty, tw, th) = page.rect(text);
        let (ex, ey) = ((tx + tw) as f32 - 10.0, (ty + th / 2.0) as f32);
        page.press_at(cx, cy);
        for step in 1..=8u8 {
            let t = f32::from(step) / 8.0;
            page.move_to(cx + (ex - cx) * t, cy + (ey - cy) * t);
        }
        page.release_at(ex, ey);
        assert_eq!(rows(&page), 0, "a drag towards {text} selected it");
    }
}

fn range() -> Element {
    rsx! {
        RangeSlider { label: "Price", value: Some((20.0, 80.0)), sx: sx().width("400px") }
    }
}

/// The two thumbs sit in a `group` the label names on the web (todo 1151).
#[test]
fn a_range_slider_is_a_group_of_two_sliders_in_the_ax_tree() {
    let page = mount(range);
    assert_eq!(
        page.accessible("[role=group]").0,
        "Group",
        "{}",
        page.tree()
    );
    for thumb in ["[aria-label=Minimum]", "[aria-label=Maximum]"] {
        assert_eq!(page.accessible(thumb).0, "Slider", "{}", page.tree());
    }
}

/// Blitz maps `role` only: no `aria-labelledby`, `aria-label` or `aria-value*`.
#[test]
#[ignore = "needs Blitz: aria-labelledby is not mapped into the AX tree"]
fn the_label_names_the_range_slider_group_in_the_ax_tree() {
    let page = mount(range);
    assert_eq!(page.accessible("[role=group]").1, "Price");
}
