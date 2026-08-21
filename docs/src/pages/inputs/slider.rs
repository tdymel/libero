use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Slider, SliderChangeEvent, SliderMark, Text};

const SIZES: [&str; 5] = ["xs", "sm", "md", "lg", "xl"];

#[component]
pub fn SliderPage() -> Element {
    let mut volume = use_signal(|| 40.0);
    let mut zoom = use_signal(|| 1.0);
    let mut committed = use_signal(|| 40.0);
    let mut size_index = use_signal(|| 2.0);

    rsx! {
        DocPage {
            title: "Slider",
            lead: rsx! {
                Text {
                    "A value dragged along a track. Controlled: it renders "
                    Code { source: "value" }
                    " and asks for a new one through "
                    Code { source: "on_change" }
                    ". Pointer, touch and keyboard all drive it - the thumb is a "
                    Code { source: "role=\"slider\"" }
                    " with arrows, Page keys, Home and End."
                }
            },
            DocSection {
                title: "Basic",
                Slider {
                    aria_label: "Volume",
                    value: volume(),
                    on_change: move |event: SliderChangeEvent| volume.set(event.value()),
                }
                Text { "Value: {volume()}" }
            }
            DocSection {
                title: "Sizes",
                Text {
                    Code { source: "size" }
                    " scales the track and thumb; "
                    Code { source: "radius" }
                    " rounds the track independently."
                }
                Flex {
                    direction: "column",
                    gap: "lg",
                    for size in SIZES {
                        Slider {
                            key: "{size}",
                            size,
                            value: volume(),
                            on_change: move |event: SliderChangeEvent| volume.set(event.value()),
                        }
                    }
                }
            }
            DocSection {
                title: "Colors",
                Flex {
                    direction: "column",
                    gap: "lg",
                    Slider {
                        color: "success",
                        value: volume(),
                        on_change: move |event: SliderChangeEvent| volume.set(event.value()),
                    }
                    Slider {
                        color: "error",
                        value: volume(),
                        on_change: move |event: SliderChangeEvent| volume.set(event.value()),
                    }
                }
            }
            DocSection {
                title: "Steps and range",
                Text {
                    Code { source: "step" }
                    " is the grid the value snaps to, measured from "
                    Code { source: "min" }
                    " - and it sets how many decimals an emitted value keeps, so "
                    Code { source: "0.1" }
                    " never reports "
                    Code { source: "0.30000000000000004" }
                    "."
                }
                Slider {
                    min: 0.5,
                    max: 3.0,
                    step: 0.1,
                    value: zoom(),
                    label: Callback::new(|value: f64| format!("{value}x")),
                    on_change: move |event: SliderChangeEvent| zoom.set(event.value()),
                }
                Text { "Zoom: {zoom()}x" }
            }
            DocSection {
                title: "Marks",
                Text {
                    Code { source: "marks" }
                    " puts ticks on the track. A "
                    Code { source: "SliderMark::labeled" }
                    " one gets a caption under it, and the slider reserves the room for it; "
                    Code { source: "SliderMark::new" }
                    " is a bare tick."
                }
                Flex {
                    direction: "column",
                    gap: "xl",
                    Slider {
                        size: "lg",
                        value: volume(),
                        marks: vec![
                            SliderMark::labeled(0.0, "0%"),
                            SliderMark::new(25.0),
                            SliderMark::labeled(50.0, "50%"),
                            SliderMark::new(75.0),
                            SliderMark::labeled(100.0, "100%"),
                        ],
                        on_change: move |event: SliderChangeEvent| volume.set(event.value()),
                    }
                    Slider {
                        color: "success",
                        value: volume(),
                        marks: vec![
                            SliderMark::new(20.0),
                            SliderMark::new(40.0),
                            SliderMark::new(60.0),
                            SliderMark::new(80.0),
                        ],
                        on_change: move |event: SliderChangeEvent| volume.set(event.value()),
                    }
                }
            }
            DocSection {
                title: "Discrete steps",
                Text {
                    "A "
                    Code { source: "step" }
                    " of one over an index makes the slider pick from a list instead of a "
                    "number - here the size scale, with a mark per step and a "
                    Code { source: "label" }
                    " that names it. This one resizes itself as you drag."
                }
                Slider {
                    aria_label: "Size",
                    size: SIZES[size_index() as usize],
                    min: 0.0,
                    max: (SIZES.len() - 1) as f64,
                    step: 1.0,
                    value: size_index(),
                    label: Callback::new(|value: f64| SIZES[value as usize].to_string()),
                    marks: SIZES
                        .iter()
                        .enumerate()
                        .map(|(index, size)| SliderMark::labeled(index as f64, *size))
                        .collect(),
                    on_change: move |event: SliderChangeEvent| size_index.set(event.value()),
                }
            }
            DocSection {
                title: "Label",
                Text {
                    Code { source: "label" }
                    " formats the bubble over the thumb, and the thumb's "
                    Code { source: "aria-valuetext" }
                    " with it. The bubble is a "
                    Code { source: "Tooltip" }
                    ", shown on hover, while dragging and while the thumb has keyboard "
                    "focus. Without "
                    Code { source: "label" }
                    " it shows the bare value."
                }
                Slider {
                    size: "lg",
                    value: volume(),
                    label: Callback::new(|value: f64| format!("{value}%")),
                    on_change: move |event: SliderChangeEvent| volume.set(event.value()),
                }
            }
            DocSection {
                title: "Change events",
                Text {
                    Code { source: "Start" }
                    " and "
                    Code { source: "End" }
                    " bracket one drag, "
                    Code { source: "Change" }
                    " carries every value in between - so expensive work can wait for "
                    Code { source: "End" }
                    "."
                }
                Slider {
                    value: volume(),
                    on_change: move |event: SliderChangeEvent| {
                        volume.set(event.value());
                        if let SliderChangeEvent::End(value) = event {
                            committed.set(value);
                        }
                    },
                }
                Text { "Committed on release: {committed()}" }
            }
            DocSection {
                title: "Disabled",
                Slider { value: 60.0, disabled: true, on_change: move |_| {} }
            }
        }
    }
}
