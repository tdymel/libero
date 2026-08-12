use dioxus::prelude::*;

use crate::{
    components::{Box, Input, util::classes},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::StackDefaults,
};

static STACK_BASE_SX: StaticSx = StaticSx::new(|| sx().display("flex"));

fn stack_dynamic_sx(props: &StackProps) -> crate::sx::Sx {
    let is_row = matches!(props.direction.as_ref(), Some(ThemeAwareValue::String(value)) if value == "row")
        || props.wrap.as_ref().is_some();

    StackDefaults::default_sx(is_row)
        .apply_if(props.align.as_ref(), |sx, align| {
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
    #[props(default)]
    states: Vec<(&'static str, bool)>,
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
    let stack_class = crate::context::use_sx(&STACK_BASE_SX, crate::SxLayer::Framework);
    let dynamic_class =
        crate::context::use_sx(&stack_dynamic_sx(&props), crate::SxLayer::UserDynamic);

    let class = classes(props.class, stack_class);
    let class = classes(class, dynamic_class);

    rsx! {
        Box {
            attributes: props.attributes,
            class: class,
            sx: props.sx,
            states: props.states,
            variables: Vec::new(),
            {props.children}
        }
    }
}
