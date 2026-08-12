use dioxus::prelude::*;
use dioxus_core::AttributeValue;

use crate::{
    SxLayer,
    components::{Input, States},
    context::use_sx,
    sx::Sx,
};

#[derive(Props, Clone, PartialEq)]
pub struct BoxProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: States,
    #[props(default)]
    variables: Vec<(&'static str, String)>,
    children: Element,
}

#[component]
pub fn Box(props: BoxProps) -> Element {
    let sx_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, SxLayer::UserStatic));

    let data_state = props.states.data_state();

    let mut attributes = props.attributes;

    if !props.variables.is_empty() {
        let variables_style = props
            .variables
            .into_iter()
            .map(|(name, value)| format!("--lsx-{name}:{value};"))
            .collect::<String>();

        let existing_style = attributes.iter().find_map(|attribute| {
            if attribute.name == "style" {
                match &attribute.value {
                    AttributeValue::Text(value) => Some(value.to_string()),
                    _ => None,
                }
            } else {
                None
            }
        });

        attributes.retain(|attribute| attribute.name != "style");

        let style = match existing_style {
            Some(existing_style) => format!("{existing_style}{variables_style}"),
            None => variables_style,
        };

        attributes.push(Attribute::new(
            "style",
            AttributeValue::Text(style.into()),
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
            "data-state": data_state,
            ..attributes,
            {props.children}
        }
    }
}
