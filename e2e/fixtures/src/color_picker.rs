//! `ColorPicker`.

use dioxus::prelude::*;
use libero::components::{Button, ColorCode, ColorPicker, Flex, SliderChangeEvent, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/color-picker", || rsx! { ColorPickerPage {} })];

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
