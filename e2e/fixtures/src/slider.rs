//! `Slider`.

use dioxus::prelude::*;
use libero::components::{Flex, Slider, SliderChangeEvent, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/slider", || rsx! { SliderPage {} })];

/// The one fixture that exists for the pointer pass. Nothing else in the suite
/// reaches a drag, and the thumb is also the sharpest case for target size.
/// The label and read-out give the contrast pass on-screen text (todo 387).
#[component]
fn SliderPage() -> Element {
    let mut volume = use_signal(|| 40.0f64);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Slider {
                label: "Volume",
                value: volume(),
                min: 0.0f64,
                max: 100.0f64,
                oninput: move |e: SliderChangeEvent<f64>| volume.set(e.value()),
            }
            Text { id: "volume-readout", "{volume():.0}%" }
        }
    }
}
