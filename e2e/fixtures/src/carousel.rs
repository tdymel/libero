//! `Carousel`.

use dioxus::prelude::*;
use libero::components::{Button, Carousel, Flex, Slider, SliderChangeEvent, Text, TextField};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/carousel", || rsx! { CarouselPage {} }),
    ("/carousel/loop", || rsx! { LoopPage {} }),
    ("/carousel/autoplay", || rsx! { AutoplayPage {} }),
    ("/carousel/text", || rsx! { TextPage {} }),
    ("/carousel/buttons", || rsx! { ButtonsPage {} }),
    ("/carousel/fits", || rsx! { FitsPage {} }),
    ("/carousel/replaced", || rsx! { ReplacedPage {} }),
];

/// A focusable `video` as a slide's child, whose outset ring needs the slide's padding (todo 2577).
#[component]
fn ReplacedPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Carousel {
                aria_label: "Clips",
                slides: (1..=2)
                    .map(|n| rsx! {
                        video { id: "clip-{n}", controls: true, tabindex: "0", style: "width: 100%; background: #369" }
                    })
                    .collect(),
            }
        }
    }
}

/// A draggable strip of buttons that count their clicks (todo 2358).
#[component]
fn ButtonsPage() -> Element {
    let mut clicks = use_signal(|| 0u32);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Carousel {
                aria_label: "Pressable slides",
                draggable: true,
                slides: (1..=3)
                    .map(|n| rsx! {
                        Button { variant: "outlined", onclick: move |_| clicks += 1, "Press {n}" }
                    })
                    .collect(),
            }
            Text { id: "clicks", "{clicks}" }
        }
    }
}

/// Autoplay with nothing to rotate: one slide, and three that fit (todo 2361).
#[component]
fn FitsPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "600px",
            Carousel {
                aria_label: "One slide",
                autoplay: true,
                slides: vec![rsx! { Text { "Only slide" } }],
            }
            Carousel {
                aria_label: "Three that fit",
                autoplay: true,
                per_view: 3.0,
                slides: (1..=3).map(|n| rsx! { Text { "Slide {n}" } }).collect(),
            }
        }
    }
}

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
