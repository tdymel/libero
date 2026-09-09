//! `Carousel`.

use dioxus::prelude::*;
use libero::components::{Button, Carousel, Flex, Slider, SliderChangeEvent, Text, TextField};

use crate::Routes;

pub const ROUTES: Routes = &[("/carousel", || rsx! { CarouselPage {} })];

/// The fixture for the one thing a carousel cannot get from structure: a key
/// pressed **inside** a slide.
///
/// `Carousel`'s track handler acts on the arrows, Home and End, and
/// `prevent_default()`s them. Its slides hold whatever a caller puts there,
/// which is code the component cannot ask to stop propagating, so the press
/// arrives at the track whatever it landed on. Three guard arms separate the
/// two cases (`carousel.rs`, `key_taken || typing_target || arrow_target`),
/// and the first slide holds one control per arm:
///
/// * a `TextField` and a `Slider` - the library's own controls. The slider
///   takes the arrows and marks them by preventing their default, which is
///   `key_taken`; the text field is left to the browser, which is
///   `typing_target`.
/// * a raw `<input type="range">` and a raw radio pair - HTML a caller wrote,
///   which nothing in the library marks and which the browser steps with the
///   arrows anyway. That is `arrow_target`.
/// * a `Button`, which **no** arm covers. It is the positive control: arrows
///   pressed on it have to move the strip, or "the strip did not move" is a
///   sentence about a carousel that never moves.
///
/// The rest of the slides are plain text. Only the resting slide is out of
/// `inert`, so a control on another one could not be focused to press a key
/// in it.
#[component]
fn CarouselPage() -> Element {
    let mut note = use_signal(|| "carousel".to_string());
    let mut volume = use_signal(|| 40.0f64);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Carousel {
                aria_label: "Slide content",
                // Not the theme default (`false`). The dots are a carousel's
                // likeliest WCAG 2.5.8 failure and the `Suite` baseline cannot
                // measure a control that is not drawn (todo 381).
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
