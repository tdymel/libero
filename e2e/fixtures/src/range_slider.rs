//! `RangeSlider`.

use dioxus::prelude::*;
use libero::components::{Flex, RangeSlider, Slider, SliderChangeEvent, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/range-slider", || rsx! { RangeSliderPage {} }),
    ("/range-slider/required", || rsx! { RequiredPage {} }),
];

/// Todo 1174: required said in the name, as a thumb takes no `aria-required`.
#[component]
fn RequiredPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Slider { label: "Volume", required: true, value: 40.0 }
            RangeSlider { label: "Price", required: true, value: (20.0, 80.0) }
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
