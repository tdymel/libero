use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{IntoChildren, base_props, focus_ring_sx, input_from_str, variables},
        layout::use_box,
    },
    str_enum::str_enum,
    sx::{StaticSx, ThemeAwareValue, sx},
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
        assert_eq!(FlexWrap::from("wrap"), FlexWrap::Wrap);
        assert_eq!(FlexWrap::from("wrap-reverse"), FlexWrap::NoWrap);
        assert_eq!(FlexWrap::from("nonsense"), FlexWrap::NoWrap);
    }
}

// Column is the unconditional base and `row` overrides it. `gap` folds in
// afterwards, so an explicit `gap` beats either axis's default spacing.
// The per-instance vars reset to `initial` so a nested Flex does not inherit them.
static FLEX_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .display("flex")
        .var(FLEX_ALIGN_VAR, "initial")
        .var(FLEX_JUSTIFY_VAR, "initial")
        .var(FLEX_WRAP_VAR, "initial")
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
        /// Between each child, not before the first or after the last.
        /// Ignored against upstream main, which cannot split children apart.
        #[props(default)]
        divider: Option<Element>,
        #[cfg(feature = "dioxus-fork")]
        children: Vec<Element>,
        #[cfg(not(feature = "dioxus-fork"))]
        children: Element,
    }
}

#[component]
pub fn Flex(props: FlexProps) -> Element {
    let direction = props.direction.copied_or_default();
    let variables: Input<Variables> = flex_variables(&props).into();

    let mut states = props
        .states
        .unwrap_or_default()
        .with("row", direction == FlexDirection::Row);
    if let Some(gap) = props.gap.as_ref().copied() {
        states = states.with(gap.state_name(), true);
    }
    let states: Input<States> = states.into();

    let own_children = props.children.into_children();
    let last_index = own_children.len().saturating_sub(1);
    let children = match props.divider {
        None => rsx! { {own_children.into_iter()} },
        Some(divider) => {
            #[cfg(not(feature = "dioxus-fork"))]
            crate::utils::warn(
                "Flex `divider` needs the \"dioxus-fork\" feature - upstream main merges \
                 children into one node, so no divider is rendered.",
            );
            rsx! {
                for (index, child) in own_children.into_iter().enumerate() {
                    {child}
                    if index != last_index {
                        {divider.clone()}
                    }
                }
            }
        }
    };

    use_box()
        .framework_sx(&FLEX_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Div, props.attributes, children)
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
            #[cfg(feature = "dioxus-fork")]
            children: Vec::new(),
            #[cfg(not(feature = "dioxus-fork"))]
            children: rsx! {},
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
