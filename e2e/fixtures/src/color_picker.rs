//! `ColorPicker`.

use dioxus::prelude::*;
use libero::components::{Button, ColorCode, ColorPicker, Flex, SliderChangeEvent, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/color-picker", || rsx! { ColorPickerPage {} }),
    ("/color-picker-alpha", || rsx! { ColorPickerAlphaPage {} }),
    (
        "/color-picker-swatches",
        || rsx! { ColorPickerSwatchesPage {} },
    ),
];

/// A palette whose first swatch is the value, for the pressed state.
#[component]
fn ColorPickerSwatchesPage() -> Element {
    let mut color = use_signal(|| "#fa5252".parse::<ColorCode>().unwrap());

    rsx! {
        ColorPicker {
            value: color(),
            with_picker: false,
            swatches: ["#fa5252", "#40c057", "#f8f9fa"],
            oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
        }
    }
}

/// With the alpha slider and the preview, which paints from the root's vars.
#[component]
fn ColorPickerAlphaPage() -> Element {
    let mut color = use_signal(|| "#1c7ed6".parse::<ColorCode>().unwrap());

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ColorPicker {
                value: color(),
                with_alpha: true,
                saturation_label: "Saturation",
                hue_label: "Hue",
                alpha_label: "Alpha",
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
            }
            Text { id: "color", "{color().to_hex()}" }
        }
    }
}

/// `#1c7ed6` is saturation 87%, brightness 84%: room to step both ways. The
/// button before gives a drag focus to take away.
#[component]
fn ColorPickerPage() -> Element {
    let mut color = use_signal(|| "#1c7ed6".parse::<ColorCode>().unwrap());

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "before", "Before" }
            ColorPicker {
                value: color(),
                saturation_label: "Saturation",
                hue_label: "Hue",
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
            }
            Text { id: "color", "{color().to_hex()}" }
            Button { id: "after", "After" }
        }
    }
}
