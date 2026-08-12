use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::StackDefaults,
};

/*
 * Notes:
 * - Mantine Group has an option to set equal group width.
 *   We should at least provide a variable to use it on children.
 *   Not sure if we should provide a similar API.
 */

static STACK_BASE_COLUMN_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").and(StackDefaults::default_sx(false)));
static STACK_BASE_ROW_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").and(StackDefaults::default_sx(true)));

fn stack_dynamic_sx(props: &StackProps) -> crate::sx::Sx {
    sx().apply_if(props.align.as_ref(), |sx, align| {
        sx.align_items(align.clone())
    })
    .apply_if(props.justify.as_ref(), |sx, justify| {
        sx.justify_content(justify.clone())
    })
    .apply_if(props.gap.as_ref(), |sx, gap| sx.gap(gap.clone()))
    .apply_if(props.wrap.as_ref(), |sx, wrap| sx.flex_wrap(wrap.clone()))
    .apply_if(props.direction.as_ref(), |sx, direction| {
        sx.flex_direction(direction.clone())
    })
}

#[derive(Props, Clone, PartialEq)]
pub struct StackProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    align: Input<ThemeAwareValue>,
    #[props(default, into)]
    justify: Input<ThemeAwareValue>,
    #[props(default, into)]
    gap: Input<ThemeAwareValue>,
    #[props(default, into)]
    direction: Input<ThemeAwareValue>,
    #[props(default, into)]
    wrap: Input<ThemeAwareValue>,
    children: Element,
}

#[component]
pub fn Stack(props: StackProps) -> Element {
    let is_row =
        matches!(props.direction.as_ref(), Some(ThemeAwareValue::String(value)) if value == "row");
    let stack_base_sx = if is_row {
        &STACK_BASE_ROW_SX
    } else {
        &STACK_BASE_COLUMN_SX
    };

    let stack_class = crate::context::use_sx(stack_base_sx, crate::SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));
    let dynamic_class =
        crate::context::use_sx(&stack_dynamic_sx(&props), crate::SxLayer::UserDynamic);

    let class = class_list([props.class, stack_class, dynamic_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    rsx! {
        div {
            class: class,
            "data-state": data_state,
            ..props.attributes,
            {props.children}
        }
    }
}
