use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Slider, SliderChangeEvent, SliderMark, Text};

#[component]
pub fn SliderPage() -> Element {
    let mut volume = use_signal(|| 40.0);
    let mut zoom = use_signal(|| 1.0);
    let mut committed = use_signal(|| 40.0);

    rsx! {
        DocPage {
            title: "Slider",
            lead: rsx! {
                Text {
                    "A value dragged along a track. Controlled: it renders "
                    Code { "value" }
                    " and asks for a new one through "
                    Code { "on_change" }
                    ". Pointer, touch and keyboard all drive it - the thumb is a "
                    Code { "role=\"slider\"" }
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
                    Code { "size" }
                    " scales the track and thumb; "
                    Code { "radius" }
                    " rounds the track independently."
                }
                Flex {
                    direction: "column",
                    gap: "lg",
                    for size in ["xs", "sm", "md", "lg", "xl"] {
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
                    Code { "step" }
                    " is the grid the value snaps to, measured from "
                    Code { "min" }
                    " - and it sets how many decimals an emitted value keeps, so "
                    Code { "0.1" }
                    " never reports "
                    Code { "0.30000000000000004" }
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
                    Code { "marks" }
                    " puts ticks on the track. A "
                    Code { "SliderMark::labeled" }
                    " one gets a caption under it, and the slider reserves the room for it; "
                    Code { "SliderMark::new" }
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
                title: "Label",
                Text {
                    Code { "label" }
                    " formats a bubble over the thumb, and the thumb's "
                    Code { "aria-valuetext" }
                    " with it. It fades in while dragging and while the thumb has keyboard "
                    "focus - drag the thumb, or tab to it and press an arrow key, to see it. "
                    "Without "
                    Code { "label" }
                    " there is no bubble at all."
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
                    Code { "Start" }
                    " and "
                    Code { "End" }
                    " bracket one drag, "
                    Code { "Change" }
                    " carries every value in between - so expensive work can wait for "
                    Code { "End" }
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
