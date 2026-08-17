use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States, Variables,
        common::{base_props, focus_ring_sx, variables},
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{FLEX_ALIGN_VAR, FLEX_JUSTIFY_VAR, FLEX_WRAP_VAR, FlexDefaults, Size, SizeCss},
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

fn flex_variables(props: &FlexProps) -> Variables {
    variables()
        .with(
            FLEX_ALIGN_VAR,
            props.align.as_ref().and_then(ThemeAwareValue::raw),
        )
        .with(
            FLEX_JUSTIFY_VAR,
            props.justify.as_ref().and_then(ThemeAwareValue::raw),
        )
        .with(
            FLEX_WRAP_VAR,
            props.wrap.as_ref().and_then(ThemeAwareValue::raw),
        )
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

    let variables = flex_variables(&props);

    let last_index = props.children.len().saturating_sub(1);

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states,
            variables,
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
