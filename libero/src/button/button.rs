#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{Color, Size, call_handler, sx as sx_builder, use_resolved_class};

use super::button_props::ButtonProps;

pub fn Button(props: ButtonProps) -> Element {
    let button_sx = sx_builder()
            .background_color(Color::Primary.shade(5))
            .border_radius(Size::Sm)
            .padding(Size::Sm)
            .color(Color::Primary.contrast_main())
            .font_size(Size::Md)
            .font_weight("500")
            .font_family("\"Roboto\", \"Helvetica\", \"Arial\", sans-serif")
            .box_shadow("0px 3px 1px -2px rgba(0, 0, 0, 0.2), 0px 2px 2px 0px rgba(0, 0, 0, 0.14), 0px 1px 5px 0px rgba(0, 0, 0, 0.12)")
            // needed for the ripple pseudo-element to be positioned and clipped
            .position("relative")
            .overflow("hidden")
            // small transition for the pressed shadow and active background
            .transition("background-color 120ms cubic-bezier(0,0,0.2,1), box-shadow 120ms cubic-bezier(0,0,0.2,1)")
            .cursor("pointer")
            .user_select("none")
            .text_align("center")
            .text_transform("uppercase")
            // create an expanding white ellipse using ::after (animates on :active)
            .selector("&::after", sx_builder()
                .content("\"\"")
                .position("absolute")
                .left("50%")
                .top("50%")
                .width("200%")
                .height("200%")
                .pointer_events("none")
                .border_radius("50%")
                .background("rgba(255,255,255,0.1)")
                .opacity("0")
                .transform("translate(-50%, -50%) scaleX(0.02) scaleY(0.6)")
                .transform_origin("center")
                .transition("transform 250ms cubic-bezier(0.4,0,0.2,1), opacity 250ms ease-out")
            )
            .selector("&:active::after", sx_builder()
                .transform("translate(-50%, -50%) scaleX(1.1) scaleY(1.1)")
                .opacity("1")
                .transition("transform 180ms cubic-bezier(0.4,0,0.2,1), opacity 180ms ease-out")
            )
            .hover(sx_builder().background_color(Color::Primary.shade(9)))
            // active pressed state
            .active(
                sx_builder()
                    .box_shadow("0px 1px 1px -1px rgba(0, 0, 0, 0.2)")
            );

    // #[cfg(debug_assertions)]
    // {
    //     let total_size = std::mem::size_of_val(&button_sx);
    //     let declaration_count = button_sx.declarations.len();
    //     let nested_rule_count = button_sx.nested_rules.len();
    //     let declaration_bytes = std::mem::size_of::<crate::sx::SxStaticDeclaration>();
    //     let nested_rule_bytes = std::mem::size_of::<crate::sx::NestedRuleMeta>();
    //     let overhead = total_size
    //         .saturating_sub(declaration_count * declaration_bytes)
    //         .saturating_sub(nested_rule_count * nested_rule_bytes);

    //     eprintln!(
    //         "Button base sx size: {total_size} bytes ({declaration_count} declarations × {declaration_bytes} bytes, {nested_rule_count} nested rules × {nested_rule_bytes} bytes, {overhead} bytes overhead)"
    //     );
    // }

    let class = use_resolved_class(button_sx, props.sx, props.class);

    rsx! {
        div {
            class: class.clone(),
            onclick: move |event| call_handler(&props.onclick, event),
            ..props.attributes,
            {props.children}
        }
    }
}
