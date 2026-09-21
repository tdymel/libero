//! `Carousel`.

use dioxus::prelude::*;
use libero::components::{Button, Carousel, Flex, Slider, SliderChangeEvent, Text, TextField};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/carousel", || rsx! { CarouselPage {} }),
    ("/carousel/loop", || rsx! { LoopPage {} }),
    ("/carousel/autoplay", || rsx! { AutoplayPage {} }),
    ("/carousel/text", || rsx! { TextPage {} }),
];

/// Plain text slides, so a drag lands on nothing that takes the pointer.
#[component]
fn TextPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Carousel {
                aria_label: "Text slides",
                draggable: true,
                slides: (1..=4)
                    .map(|n| rsx! {
                        Text { "Slide {n}" }
                    })
                    .collect(),
            }
        }
    }
}

/// Five slides with a button each, three up and looping: at rest on slide 1
/// the centred strip shows the clone of slide 5 on its left.
#[component]
fn LoopPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "600px",
            Carousel {
                aria_label: "Looping cards",
                per_view: 3.0,
                r#loop: true,
                slides: (1..=5)
                    .map(|n| rsx! {
                        Button { variant: "outlined", "Open {n}" }
                    })
                    .collect(),
            }
        }
    }
}

/// Autoplay on a short delay, so a test sees it advance within its wait.
#[component]
fn AutoplayPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Button { id: "before", variant: "outlined", "Before" }
            Carousel {
                aria_label: "Rotating",
                autoplay: true,
                autoplay_delay: 400u32,
                indicators: true,
                slides: (1..=4)
                    .map(|n| rsx! {
                        Text { "Slide {n}" }
                    })
                    .collect(),
            }
        }
    }
}

/// Keys pressed inside a slide: one control per track guard arm (`key_taken`, `typing_target`,
/// `arrow_target`), plus a `Button` no arm covers as the positive control.
#[component]
fn CarouselPage() -> Element {
    let mut note = use_signal(|| "carousel".to_string());
    let mut volume = use_signal(|| 40.0f64);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Carousel {
                aria_label: "Slide content",
                // Not the default: the dots are the likeliest WCAG 2.5.8 failure (todo 381).
                indicators: true,
                slides: vec![
                    rsx! {
                        Flex { direction: "column", gap: "sm",
                            TextField {
                                label: "Note",
                                value: note(),
                                oninput: move |next| note.set(next),
                            }
                            Slider {
                                aria_label: "Volume",
                                value: volume(),
                                min: 0.0f64,
                                max: 100.0f64,
                                oninput: move |e: SliderChangeEvent<f64>| volume.set(e.value()),
                            }
                            input {
                                id: "raw-range",
                                r#type: "range",
                                min: "0",
                                max: "100",
                                step: "1",
                                value: "50",
                                "aria-label": "Raw range",
                            }
                            div { role: "radiogroup", "aria-label": "Raw choice",
                                label {
                                    input {
                                        id: "raw-radio-a",
                                        r#type: "radio",
                                        name: "raw-choice",
                                        checked: true,
                                    }
                                    " A"
                                }
                                label {
                                    input { id: "raw-radio-b", r#type: "radio", name: "raw-choice" }
                                    " B"
                                }
                            }
                            Button { id: "slide-button", variant: "outlined", "Plain button" }
                        }
                    },
                    rsx! {
                        Text { "The second slide, and nothing to press on it." }
                    },
                    rsx! {
                        Text { "The third slide, and nothing to press on it." }
                    },
                ],
            }
        }
    }
}
