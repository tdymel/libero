use dioxus::prelude::*;

use crate::{context::use_sx, sx::Sx};

#[derive(Props, Clone, PartialEq)]
pub struct BoxProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default)]
    sx: Option<&'static Sx>,
    children: Element,
}

#[component]
pub fn Box(props: BoxProps) -> Element {
    let sx_class = props.sx.map(|sx| {
        use_sx(sx);
        sx.class_name()
    });

    let class = match (props.class.as_deref(), sx_class.as_deref()) {
        (Some(class), Some(sx_class)) => Some(format!("{class} {sx_class}")),
        (Some(class), None) => Some(class.to_string()),
        (None, Some(sx_class)) => Some(sx_class.to_string()),
        (None, None) => None,
    };

    rsx! {
        div {
            class: class,
            ..props.attributes,
            {props.children}
        }
    }
}
