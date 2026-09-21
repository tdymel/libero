//! `Slider`.

use dioxus::prelude::*;
use libero::{
    components::{Flex, RangeSlider, Slider, SliderChangeEvent, SliderValue, Text},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/slider", || rsx! { SliderPage {} }),
    ("/slider/states", || rsx! { SliderStatesPage {} }),
    ("/slider/drag", || rsx! { SliderDragPage {} }),
    ("/slider/scroll", || rsx! { SliderScrollPage {} }),
];

/// Todo 1020: a 300px slider and range mid-way down a page taller than any
/// screen. `#slider-end` and `#range-end` show the value each last committed.
#[component]
fn SliderScrollPage() -> Element {
    let mut value = use_signal(|| 0.0f64);
    let mut ended = use_signal(String::new);
    let mut range = use_signal(|| (20.0f64, 80.0f64));
    let mut range_ended = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", sx: sx().padding_top("60vh").padding_bottom("150vh"),
            Slider {
                aria_label: "Volume",
                value: Some(value()),
                oninput: move |e: SliderChangeEvent<f64>| {
                    value.set(e.value());
                    if let SliderChangeEvent::End(v) = e {
                        ended.set(format!("{v:.0}"));
                    }
                },
                sx: sx().width("300px"),
            }
            Text { id: "slider-end", "{ended}" }
            RangeSlider {
                aria_label: "Price",
                value: range(),
                oninput: move |e: SliderChangeEvent<(f64, f64)>| {
                    range.set(e.value());
                    if let SliderChangeEvent::End((a, b)) = e {
                        range_ended.set(format!("{a:.0}-{b:.0}"));
                    }
                },
                sx: sx().width("300px"),
            }
            Text { id: "range-end", "{range_ended}" }
        }
    }
}

/// A 400px slider at 0, for the shared web/native drag scenarios.
#[component]
fn SliderDragPage() -> Element {
    let mut value = use_signal(|| 0.0f64);

    rsx! {
        Slider {
            aria_label: "Volume",
            value: Some(value()),
            oninput: move |e: SliderChangeEvent<f64>| value.set(e.value()),
            sx: sx().width("400px"),
        }
    }
}

/// The pointer pass's fixture: the only drag in the suite, and the sharpest target-size case.
/// The label and read-out give the contrast pass text (todo 387).
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

#[derive(Clone, Copy, Debug, PartialEq, SliderValue)]
enum Quality {
    Low,
    Medium,
    High,
}

/// The docs' switches, one slider each: required and discrete, continuous
/// with `step: 0`, read-only and disabled.
#[component]
fn SliderStatesPage() -> Element {
    let mut quality = use_signal(|| Quality::Medium);
    let mut gain = use_signal(|| 50.0f64);

    rsx! {
        Flex { direction: "column", gap: "lg", max_width: "320px",
            Slider {
                id: "quality",
                label: "Quality",
                required: true,
                value: quality(),
                oninput: move |e: SliderChangeEvent<Quality>| quality.set(e.value()),
            }
            Slider {
                id: "gain",
                label: "Gain",
                value: gain(),
                step: 0.0f64,
                format: Callback::new(|value: f64| format!("{value:.1} dB")),
                oninput: move |e: SliderChangeEvent<f64>| gain.set(e.value()),
            }
            Slider {
                id: "fixed",
                label: "Fixed",
                readonly: true,
                value: 30.0f64,
                oninput: move |_: SliderChangeEvent<f64>| {},
            }
            Slider {
                id: "locked",
                // No visible label: axe leaves a disabled field's text
                // unmeasured, which the contrast coverage check refuses.
                aria_label: "Locked",
                disabled: true,
                value: 70.0f64,
                oninput: move |_: SliderChangeEvent<f64>| {},
            }
        }
    }
}
