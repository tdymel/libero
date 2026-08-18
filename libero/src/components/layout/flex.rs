use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States, Variables,
        common::{base_props, focus_ring_sx, input_from_str, variables},
    },
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{FLEX_ALIGN_VAR, FLEX_JUSTIFY_VAR, FLEX_WRAP_VAR, FlexDefaults, Size, SizeCss},
};

/*
 * Notes:
 * - Mantine Group has an option to set equal group width.
 *   We should at least provide a variable to use it on children.
 *   Not sure if we should provide a similar API.
 */

str_enum! {
    pub enum FlexDirection {
        Row = "row",
        #[default]
        Column = "column",
    }
}

input_from_str!(FlexDirection);

str_enum! {
    pub enum FlexWrap {
        Wrap = "wrap",
        #[default]
        NoWrap = "nowrap",
        WrapReverse = "wrap-reverse",
    }
}

impl FlexWrap {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wrap => "wrap",
            Self::NoWrap => "nowrap",
            Self::WrapReverse => "wrap-reverse",
        }
    }
}

impl From<bool> for FlexWrap {
    fn from(value: bool) -> Self {
        if value { Self::Wrap } else { Self::NoWrap }
    }
}

impl From<FlexWrap> for Input<FlexWrap> {
    fn from(value: FlexWrap) -> Self {
        Input::Value(value)
    }
}

impl From<bool> for Input<FlexWrap> {
    fn from(value: bool) -> Self {
        Input::Value(FlexWrap::from(value))
    }
}

input_from_str!(FlexWrap);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_wrap_parses_bools_and_strings() {
        assert_eq!(FlexWrap::from(true).as_str(), "wrap");
        assert_eq!(FlexWrap::from(false).as_str(), "nowrap");
        assert_eq!(FlexWrap::from("wrap-reverse"), FlexWrap::WrapReverse);
        assert_eq!(FlexWrap::from("nonsense"), FlexWrap::NoWrap);
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
        .with(FLEX_ALIGN_VAR, props.align.resolve(None))
        .with(FLEX_JUSTIFY_VAR, props.justify.resolve(None))
        .with(
            FLEX_WRAP_VAR,
            props.wrap.as_ref().map(|wrap| wrap.as_str().to_string()),
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
        wrap: Input<FlexWrap>,
        /// Rendered between each child (not before the first or after the
        /// last) - e.g. `divider: rsx! { Divider {} }`.
        #[props(default)]
        divider: Option<Element>,
        children: Vec<Element>,
    }
}

#[component]
pub fn Flex(props: FlexProps) -> Element {
    let direction = props.direction.copied_or_default();

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

#[cfg(test)]
mod variables_tests {
    use super::*;

    fn flex_props(wrap: Input<FlexWrap>, align: Input<ThemeAwareValue>) -> FlexProps {
        FlexProps {
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            align,
            justify: Input::None,
            gap: Input::None,
            direction: Input::None,
            wrap,
            divider: None,
            children: Vec::new(),
        }
    }

    #[test]
    fn wrap_is_emitted_as_its_css_keyword() {
        let props = flex_props(FlexWrap::NoWrap.into(), Input::None);

        assert_eq!(
            flex_variables(&props).to_string(),
            format!("{}:nowrap;", FLEX_WRAP_VAR.name())
        );
    }

    #[test]
    fn only_the_set_properties_are_emitted() {
        let props = flex_props(Input::None, "center".into());
        let variables = flex_variables(&props).to_string();

        assert_eq!(variables, format!("{}:center;", FLEX_ALIGN_VAR.name()));
    }

    #[test]
    fn nothing_set_emits_nothing() {
        assert_eq!(
            flex_variables(&flex_props(Input::None, Input::None)).to_string(),
            ""
        );
    }
}
