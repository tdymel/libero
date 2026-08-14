use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::FlexDefaults,
};

/*
 * Notes:
 * - Mantine Group has an option to set equal group width.
 *   We should at least provide a variable to use it on children.
 *   Not sure if we should provide a similar API.
 */

fn flex_focus_sx() -> Sx {
    sx().focus_visible(
        sx().outline("2px solid var(--lsx-primary-6)")
            .outline_offset("2px"),
    )
}

static FLEX_BASE_COLUMN_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .and(FlexDefaults::default_sx(false))
        .and(flex_focus_sx())
});
static FLEX_BASE_ROW_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .and(FlexDefaults::default_sx(true))
        .and(flex_focus_sx())
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

#[derive(Props, Clone, PartialEq)]
pub struct FlexProps {
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
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    children: Element,
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

    let flex_class = crate::context::use_sx(flex_base_sx, crate::SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));
    let dynamic_class =
        crate::context::use_sx(&flex_dynamic_sx(&props), crate::SxLayer::UserDynamic);

    let class = class_list([props.class, flex_class, dynamic_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    rsx! {
        div {
            class: class,
            "data-state": data_state,
            onclick: move |event| props.onclick.call(event),
            ..props.attributes,
            {props.children}
        }
    }
}
