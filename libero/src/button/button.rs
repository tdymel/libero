use dioxus::prelude::*;

use crate::{sx as sx_builder, use_sx};

use super::button_props::ButtonProps;

pub fn Button(props: ButtonProps) -> Element {
    let base_class = use_sx(
        sx_builder()
            .background_color("red")
            .width(200)
            .height(200)
            .opacity("0.5"),
    );
    let sx_class = props.sx.map(use_sx);
    let user_class = props.class.unwrap_or_default();

    let class = match sx_class {
        Some(sx_class) if user_class.is_empty() => format!("{} {}", base_class, sx_class),
        Some(sx_class) => format!("{} {} {}", base_class, sx_class, user_class),
        None if user_class.is_empty() => base_class,
        None => format!("{} {}", base_class, user_class),
    };

    rsx! {
        button {
            class: class,
            onclick: move |event| {
                if let Some(handler) = &props.onclick {
                    handler.call(event);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}
