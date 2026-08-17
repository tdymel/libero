use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States,
        common::{base_props, focus_ring_sx},
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{FlexDefaults, Size, SizeCss},
};

/*
 * Notes:
 * - Mantine Group has an option to set equal group width.
 *   We should at least provide a variable to use it on children.
 *   Not sure if we should provide a similar API.
 */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

impl Default for FlexDirection {
    fn default() -> Self {
        Self::Column
    }
}

impl From<&str> for FlexDirection {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "row" => Self::Row,
            _ => Self::Column,
        }
    }
}

impl From<String> for FlexDirection {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<FlexDirection> {
    fn from(value: &str) -> Self {
        Input::Value(FlexDirection::from(value))
    }
}

impl From<String> for Input<FlexDirection> {
    fn from(value: String) -> Self {
        Input::Value(FlexDirection::from(value))
    }
}

// Column is the unconditional base (also the default when `direction` is
// unset); `row` overrides it. `gap` sizes are folded in afterwards so an
// explicit `gap` prop always wins over either axis's own default spacing,
// regardless of direction.
static FLEX_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .display("flex")
        .and(FlexDefaults::default_sx(false))
        .and(sx().focus_visible(focus_ring_sx()))
        .when("row", FlexDefaults::default_sx(true));

    Size::ALL.into_iter().fold(base, |base, size| {
        base.when(size.state_name(), sx().gap(SizeCss::SPACING.value(size)))
    })
});

fn flex_dynamic_sx(props: &FlexProps) -> Sx {
    sx().apply_if(props.align.as_ref(), |sx, align| {
        sx.align_items(align.clone())
    })
    .apply_if(props.justify.as_ref(), |sx, justify| {
        sx.justify_content(justify.clone())
    })
    .apply_if(props.wrap.as_ref(), |sx, wrap| sx.flex_wrap(wrap.clone()))
}

base_props! {
    pub struct FlexProps {
        #[props(default, into)]
        align: Input<ThemeAwareValue>,
        #[props(default, into)]
        justify: Input<ThemeAwareValue>,
        #[props(default, into)]
        gap: Input<Size>,
        #[props(default, into)]
        direction: Input<FlexDirection>,
        #[props(default, into)]
        wrap: Input<ThemeAwareValue>,
        /// Rendered between each child (not before the first or after the
        /// last) - e.g. `divider: rsx! { Divider {} }`.
        #[props(default)]
        divider: Option<Element>,
        children: Vec<Element>,
    }
}

#[component]
pub fn Flex(props: FlexProps) -> Element {
    let direction = props.direction.as_ref().copied().unwrap_or_default();

    let mut states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("row", direction == FlexDirection::Row);
    if let Some(gap) = props.gap.as_ref().copied() {
        states = states.with(gap.state_name(), true);
    }

    let dynamic_class =
        crate::hooks::use_css(&flex_dynamic_sx(&props), crate::CssLayer::UserDynamic);

    let last_index = props.children.len().saturating_sub(1);

    rsx! {
        Box {
            class: props.class.unwrap_or_default().with(dynamic_class),
            sx: props.sx,
            states,
            framework_sx: &FLEX_BASE_SX,
            attributes: props.attributes,
            for (index, child) in props.children.into_iter().enumerate() {
                {child}
                if index != last_index {
                    if let Some(divider) = props.divider.clone() {
                        {divider}
                    }
                }
            }
        }
    }
}
