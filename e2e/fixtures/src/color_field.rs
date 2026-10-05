//! `ColorField` between two buttons, with the alpha slider and swatches.

use dioxus::prelude::*;
use libero::components::{Button, ColorCode, ColorField, Flex, SliderChangeEvent, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/color-field/alpha", || rsx! { ColorFieldPage {} }),
    ("/color-field/swatches", || rsx! { SwatchesFieldPage {} }),
    ("/color-field/keep-text", || rsx! { KeepTextFieldPage {} }),
];

const SWATCHES: [&str; 3] = ["#fa5252", "#40c057", "#228be6"];

/// `#1c7ed6` held; the readout shows it as hexa.
#[component]
fn ColorFieldPage() -> Element {
    let mut color = use_signal(|| "#1c7ed6".parse::<ColorCode>().unwrap());

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            ColorField {
                label: "Accent",
                with_alpha: true,
                swatches: SWATCHES,
                value: color(),
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
            }
            Button { id: "after", "After" }
            Text { id: "readout", "{color().to_hexa()}" }
        }
    }
}

/// Swatches only: the dropdown holds no picker.
#[component]
fn SwatchesFieldPage() -> Element {
    let mut color = use_signal(|| "#40c057".parse::<ColorCode>().unwrap());

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            ColorField {
                label: "Tag color",
                with_picker: false,
                swatches: SWATCHES,
                value: color(),
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
            }
            Button { id: "after", "After" }
            Text { id: "readout", "{color().to_hex()}" }
        }
    }
}

/// `fix_on_blur: false` and no dropdown: unparsable text stays on blur.
/// `#ends` lists every `End` (2296).
#[component]
fn KeepTextFieldPage() -> Element {
    let mut color = use_signal(|| "#40c057".parse::<ColorCode>().unwrap());
    let mut ends = use_signal(Vec::<String>::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            ColorField {
                label: "Border color",
                with_picker: false,
                with_eye_dropper: false,
                fix_on_blur: false,
                value: color(),
                oninput: move |event: SliderChangeEvent<ColorCode>| {
                    if let SliderChangeEvent::End(end) = event {
                        ends.push(end.to_hex());
                    }
                    color.set(event.value());
                },
            }
            Button { id: "after", "After" }
            Text { id: "readout", "{color().to_hex()}" }
            Text { id: "ends", {ends().join(" ")} }
        }
    }
}
