use dioxus::prelude::*;

use crate::{
    components::{
        Box,
        props::{AlignInput, JustifyInput},
        util::classes,
    },
    css::SizeCssVar,
    sx::{StaticSx, SxInput, sx},
    theme::{Size, StackDefaults},
};

static STACK_BASE_SX: StaticSx = StaticSx::new(|| sx().display("flex"));

fn stack_dynamic_sx(props: &StackProps) -> crate::sx::Sx {
    let is_row = matches!(props.direction.as_deref(), Some("row")) || props.wrap.is_some();

    StackDefaults::default_sx(is_row)
        .apply_if(props.align.value(), |sx, align| sx.align_items(align))
        .apply_if(props.justify.value(), |sx, justify| {
            sx.justify_content(justify)
        })
        .apply_if(props.gap.as_deref(), |sx, gap| {
            sx.gap(match Size::parse_dynamic(gap) {
                Some(size) => SizeCssVar::SPACING.value(size),
                None => gap.to_string(),
            })
        })
        .apply_if(props.wrap, |sx, wrap| {
            sx.flex_wrap(if wrap { "wrap" } else { "nowrap" })
        })
        .apply_if(props.direction.as_deref(), |sx, direction| {
            sx.flex_direction(direction)
        })
}

#[derive(Props, Clone, PartialEq)]
pub struct StackProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: SxInput,
    #[props(default)]
    states: Vec<(&'static str, bool)>,
    #[props(default, into)]
    align: AlignInput,
    #[props(default, into)]
    justify: JustifyInput,
    #[props(default)]
    gap: Option<String>,
    #[props(default)]
    direction: Option<String>,
    #[props(default)]
    wrap: Option<bool>,
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
