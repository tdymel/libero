//! `ColorPicker`.

use dioxus::prelude::*;
use libero::components::{
    AlphaSlider, Button, ColorCode, ColorPicker, Flex, HueSlider, Slider, SliderChangeEvent, Text,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/color-picker", || rsx! { ColorPickerPage {} }),
    ("/color-picker-alpha", || rsx! { ColorPickerAlphaPage {} }),
    (
        "/color-picker-swatches",
        || rsx! { ColorPickerSwatchesPage {} },
    ),
    ("/color-picker-sizes", || rsx! { ColorPickerSizesPage {} }),
    ("/color-sliders-rtl", || rsx! { ColorSlidersRtlPage {} }),
    ("/color-picker-states", || rsx! { ColorPickerStatesPage {} }),
];

/// Todos 5195, 5196: a disabled and a read-only picker, each with alpha and swatches.
#[component]
fn ColorPickerStatesPage() -> Element {
    let value = "#1c7ed6".parse::<ColorCode>().unwrap();
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ColorPicker {
                id: "disabled",
                value,
                with_alpha: true,
                swatches: ["#fa5252", "#1c7ed6"],
                disabled: true,
                oninput: move |_| {},
            }
            ColorPicker {
                id: "readonly",
                value,
                with_alpha: true,
                swatches: ["#fa5252", "#1c7ed6"],
                readonly: true,
                oninput: move |_| {},
            }
        }
    }
}

/// Todo 5119: on an RTL page the colour scales still run left to right, while a plain
/// `Slider` mirrors. Each sits a quarter of the way along its range.
#[component]
fn ColorSlidersRtlPage() -> Element {
    rsx! {
        div { dir: "rtl",
            Flex { direction: "column", gap: "lg", max_width: "320px",
                HueSlider { id: "hue", value: 90.0, aria_label: "Hue" }
                AlphaSlider {
                    id: "alpha",
                    value: 0.25,
                    color: ColorCode::hex(0x1c7ed6),
                    aria_label: "Alpha",
                }
                Slider { id: "plain", value: 25.0, aria_label: "Plain", oninput: move |_| {} }
            }
        }
    }
}

/// The two sizes whose sliders sit closest, each with alpha and a swatch row beneath.
#[component]
fn ColorPickerSizesPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md",
            for size in ["xs", "sm"] {
                ColorPicker {
                    key: "{size}",
                    id: "picker-{size}",
                    size,
                    with_alpha: true,
                    value: "#1c7ed6".parse::<ColorCode>().unwrap(),
                    swatches: ["#fa5252", "#40c057"],
                    saturation_label: "Saturation",
                    hue_label: "Hue",
                    alpha_label: "Alpha",
                    oninput: move |_| {},
                }
            }
        }
    }
}

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
