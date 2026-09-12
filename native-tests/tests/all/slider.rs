//! `Slider` under a pointer drag. A drag from the thumb and one leaving the
//! slider are in `pointer.rs`.

use dioxus::prelude::*;
use libero::{
    components::{Slider, SliderChangeEvent},
    sx::sx,
};
use native_tests::{Key, Page, mount};

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

#[test]
fn pressing_the_track_centre_and_dragging_right_follows_the_pointer() {
    let mut page = mount(app);
    assert_eq!(page.attr(THUMB, "aria-valuenow").as_deref(), Some("0"));
    // The 400px track's centre is 50; a quarter of it further is 75.
    page.drag(TRACK, 100.0, 0.0);
    let now: f64 = page
        .attr(THUMB, "aria-valuenow")
        .and_then(|v| v.parse().ok())
        .unwrap_or_default();
    assert!(
        (now - 75.0).abs() < 2.0,
        "the value is {now}\n{}",
        page.tree()
    );
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

/// The thumb's anchor is placed by `left` and centred by margins.
#[test]
fn the_thumb_moves_with_the_value() {
    let mut page = mount(app);
    let x = |page: &Page| {
        let thumb = page.node(THUMB);
        page.doc
            .inner
            .borrow()
            .get_client_bounding_rect(thumb)
            .map(|r| r.x)
    };
    let at_zero = x(&page);
    page.focus(THUMB);
    page.press(Key::End);
    page.advance(1.0);
    assert_eq!(page.attr(THUMB, "aria-valuenow").as_deref(), Some("100"));
    let at_end = x(&page);
    assert!(
        at_end
            .zip(at_zero)
            .is_some_and(|(end, zero)| end - zero > 300.0),
        "the thumb went from {at_zero:?} to {at_end:?}"
    );
}
