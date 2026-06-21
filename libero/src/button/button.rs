#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{call_handler, sx as sx_builder, use_resolved_class};

use super::button_props::ButtonProps;

pub fn Button(props: ButtonProps) -> Element {
    let class = use_resolved_class(
        sx_builder()
            .background_color("red")
            .width(200)
            .height(200)
            .color("white")
            .font_size("md")
            .padding(10)
            .opacity("0.5"),
        props.sx,
        props.class,
    );

    rsx! {
        button {
            class: class,
            onclick: move |event| call_handler(&props.onclick, event),
            ..props.attributes,
            {props.children}
        }
    }
}
