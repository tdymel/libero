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
            // needed for the ripple pseudo-element to be positioned and clipped
            .other("position", "relative")
            .other("overflow", "hidden")
            // small transition for the pressed scale and shadow
            .other("transition", "transform 120ms cubic-bezier(0,0,0.2,1), box-shadow 120ms cubic-bezier(0,0,0.2,1)")
            .cursor("pointer")
            .user_select("none")
            .text_align("center")
            .text_transform("uppercase")
            // create an expanding white ellipse using ::after (animates on :active)
            .selector("&::after", sx_builder()
                .other("content", "\"\"")
                .other("position", "absolute")
                .other("left", "50%")
                .other("top", "50%")
                .other("width", "200%")
                .other("height", "200%")
                .other("pointer-events", "none")
                .other("border-radius", "50%")
                .other("background", "rgba(255,255,255,0.3)")
                .other("opacity", "0")
                .other("transform", "translate(-50%, -50%) scaleX(0.02) scaleY(0.6)")
                .other("transform-origin", "center")
                .other("transition", "transform 250ms cubic-bezier(0.4,0,0.2,1), opacity 250ms ease-out")
            )
            .selector("&:active::after", sx_builder()
                .other("transform", "translate(-50%, -50%) scaleX(1.1) scaleY(1.1)")
                .other("opacity", "1")
                .other("transition", "transform 180ms cubic-bezier(0.4,0,0.2,1), opacity 180ms ease-out")
            )
            // active pressed transform
            .active(
                sx_builder()
                    .other("transform", "scale(0.98)")
                    .box_shadow("0px 1px 1px -1px rgba(0, 0, 0, 0.2)")
            ),
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
