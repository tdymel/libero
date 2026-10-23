//! `RangeSlider`.

use dioxus::prelude::*;
use libero::components::{Flex, RangeSlider, Slider, SliderChangeEvent, Text};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/range-slider", || rsx! { RangeSliderPage {} }),
    ("/range-slider/required", || rsx! { RequiredPage {} }),
    ("/range-slider/vertical", || rsx! { VerticalPage {} }),
];

/// Todo 3963: both sliders upright, bottom to top, on the shared core.
#[component]
fn VerticalPage() -> Element {
    let mut level = use_signal(|| 50.0f64);
    let mut band = use_signal(|| (20.0f64, 80.0f64));
    rsx! {
        Flex { direction: "row", gap: "xl", padding: "md",
            Slider {
                label: "Level",
                orientation: "vertical",
                sx: sx().height("200px"),
                value: level(),
                oninput: move |e: SliderChangeEvent<f64>| level.set(e.value()),
            }
            RangeSlider {
                label: "Band",
                orientation: "vertical",
                sx: sx().height("200px"),
                value: band(),
                oninput: move |e: SliderChangeEvent<(f64, f64)>| band.set(e.value()),
            }
            Text { id: "level-readout", "{level():.0}" }
            Text { id: "band-readout", "{band().0:.0}-{band().1:.0}" }
        }
    }
}

/// Todo 1174: required said in the name, as a thumb takes no `aria-required`.
#[component]
fn RequiredPage() -> Element {
    // Controlled with `oninput`: a bare `value` warns that it can never change.
    let mut volume = use_signal(|| 40.0f64);
    let mut price = use_signal(|| (20.0f64, 80.0f64));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Slider {
                label: "Volume",
                required: true,
                value: volume(),
                oninput: move |e: SliderChangeEvent<f64>| volume.set(e.value()),
            }
            RangeSlider {
                label: "Price",
                required: true,
                value: price(),
                oninput: move |e: SliderChangeEvent<(f64, f64)>| price.set(e.value()),
            }
        }
    }
}

/// Two thumbs: a drag must move the one it grabbed and leave the other.
#[component]
fn RangeSliderPage() -> Element {
    let mut price = use_signal(|| (20.0f64, 80.0f64));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            RangeSlider {
                label: "Price",
                value: price(),
                min: 0.0f64,
                max: 100.0f64,
                oninput: move |e: SliderChangeEvent<(f64, f64)>| price.set(e.value()),
            }
            Text { id: "price-readout", "{price().0:.0}-{price().1:.0}" }
        }
    }
}
