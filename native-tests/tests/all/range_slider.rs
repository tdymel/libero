//! `RangeSlider`: the keys move the focused thumb, a drag the grabbed one.

use dioxus::prelude::*;
use libero::{
    components::{RangeSlider, SliderChangeEvent},
    sx::sx,
};
use native_tests::{Key, mount};

fn app() -> Element {
    let mut price = use_signal(|| (20.0f64, 80.0f64));
    rsx! {
        RangeSlider {
            label: "Price",
            value: price(),
            min: 0.0f64,
            max: 100.0f64,
            oninput: move |e: SliderChangeEvent<(f64, f64)>| price.set(e.value()),
            sx: sx().width("400px"),
        }
        span { id: "readout", "{price().0:.0}-{price().1:.0}" }
    }
}

#[test]
fn home_moves_the_focused_lower_thumb_to_the_minimum() {
    let mut page = mount(app);
    page.tab();
    assert!(page.is_focused("[role=slider]"), "{}", page.focus_owner());
    page.press(Key::Home);
    assert_eq!(page.text("#readout"), "0-80", "{}", page.tree());
}

#[test]
fn end_moves_the_focused_upper_thumb_to_the_maximum() {
    let mut page = mount(app);
    page.tab();
    page.tab();
    page.press(Key::End);
    assert_eq!(page.text("#readout"), "20-100", "{}", page.tree());
}

#[test]
fn a_drag_moves_the_grabbed_upper_thumb_only() {
    const UPPER: &str = "[role=slider][aria-valuenow=\"80\"]";
    let mut page = mount(app);
    page.drag(UPPER, -80.0, 0.0);
    let readout = page.text("#readout");
    assert!(
        readout.starts_with("20-") && readout != "20-80",
        "{readout}\n{}",
        page.tree()
    );
}
