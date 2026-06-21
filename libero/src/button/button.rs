use dioxus::prelude::*;

use crate::{SxDyn, sx as sx_builder, use_sx};

#[component]
pub fn Button(sx: Option<SxDyn>) -> Element {
    let base_class = use_sx(
        sx_builder()
            .background_color("red")
            .width(200)
            .height(200)
            .opacity("0.5"),
    );
    let class = match sx {
        Some(sx) => {
            let override_class = use_sx(sx);
            format!("{} {}", base_class, override_class)
        }
        None => base_class,
    };

    rsx! {
        button {
            class: class,
            "Sample button"
        }
    }
}
