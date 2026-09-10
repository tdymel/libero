//! `Slider` under a pointer drag. The thumb is placed by a `transform`, which
//! Blitz's client rect ignores, so the drag starts on the track instead.

use dioxus::prelude::*;
use libero::{
    components::{Slider, SliderChangeEvent},
    sx::sx,
};
use native_tests::mount;

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
