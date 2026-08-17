use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States,
        common::{base_props, focus_ring_sx},
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::FlexDefaults,
};

/*
 * Notes:
 * - Mantine Group has an option to set equal group width.
 *   We should at least provide a variable to use it on children.
 *   Not sure if we should provide a similar API.
 * - No MUI-Stack-style `divider` prop: `children: Element` is an opaque
 *   compiled VNode, not a list we can walk and splice at runtime without
 *   reaching into unstable dioxus_core internals. Would need a breaking
 *   `items: Vec<Element>` prop to do safely.
 */

static FLEX_BASE_COLUMN_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .and(FlexDefaults::default_sx(false))
        .and(sx().focus_visible(focus_ring_sx()))
});
static FLEX_BASE_ROW_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .and(FlexDefaults::default_sx(true))
        .and(sx().focus_visible(focus_ring_sx()))
});

fn flex_dynamic_sx(props: &FlexProps) -> crate::sx::Sx {
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

base_props! {
    pub struct FlexProps {
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
}

#[component]
pub fn Flex(props: FlexProps) -> Element {
    let is_row =
        matches!(props.direction.as_ref(), Some(ThemeAwareValue::String(value)) if value == "row");
    let flex_base_sx = if is_row {
        &FLEX_BASE_ROW_SX
    } else {
        &FLEX_BASE_COLUMN_SX
    };

    let dynamic_class =
        crate::hooks::use_css(&flex_dynamic_sx(&props), crate::CssLayer::UserDynamic);

    rsx! {
        Box {
            class: props.class.unwrap_or_default().with(dynamic_class),
            sx: props.sx,
            states: props.states,
            framework_sx: flex_base_sx,
            attributes: props.attributes,
            {props.children}
        }
    }
}
