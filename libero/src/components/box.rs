use dioxus::prelude::*;
use dioxus_core::AttributeValue;

use crate::{
    context::use_sx,
    sx::{Sx, sx},
};

const EMPTY_SX: Sx = sx().build();

#[derive(Props, Clone, PartialEq)]
pub struct BoxProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default)]
    sx: Option<&'static Sx>,
    #[props(default)]
    states: Vec<(&'static str, bool)>,
    children: Element,
}

#[component]
pub fn Box(props: BoxProps) -> Element {
    let sx = props.sx;
    use_sx(sx.unwrap_or(&EMPTY_SX));

    let sx_class = sx.map(Sx::class_name);
    let data_state = props
        .states
        .iter()
        .filter_map(|(state, active)| active.then_some(*state))
        .collect::<Vec<_>>()
        .join(" ");
    let data_state = (!data_state.is_empty()).then_some(data_state);

    let mut attributes = props.attributes;
    if let Some(data_state) = data_state {
        attributes.push(Attribute::new(
            "data-state",
            AttributeValue::Text(data_state.into()),
            None,
            false,
        ));
    }

    let class = match (props.class.as_deref(), sx_class.as_deref()) {
        (Some(class), Some(sx_class)) => Some(format!("{class} {sx_class}")),
        (Some(class), None) => Some(class.to_string()),
        (None, Some(sx_class)) => Some(sx_class.to_string()),
        (None, None) => None,
    };

    rsx! {
        div {
            class: class,
            ..attributes,
            {props.children}
        }
    }
}
