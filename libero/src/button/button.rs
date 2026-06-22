#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{Color, Size, call_handler, sx as sx_builder, use_resolved_class};

use super::button_props::ButtonProps;

pub fn Button(props: ButtonProps) -> Element {
    let class = use_resolved_class(
        sx_builder()
            .background_color(Color::Primary)
            .border_radius(Size::Sm)
            .padding(Size::Md)
            .color("white")
            .font_size(Size::Md)
            .box_shadow("0px 3px 1px -2px rgba(0, 0, 0, 0.2), 0px 2px 2px 0px rgba(0, 0, 0, 0.14), 0px 1px 5px 0px rgba(0, 0, 0, 0.12)")
            .cursor("pointer")
            .user_select("none")
            .text_align("center")
            .text_transform("uppercase")
            .hover(sx_builder().background_color("red")),
        props.sx,
        props.class,
    );

    rsx! {
        div {
            class: class,
            onclick: move |event| call_handler(&props.onclick, event),
            ..props.attributes,
            {props.children}
        }
    }
}
