use dioxus::prelude::*;

use crate::SxDyn;

#[derive(Props, PartialEq, Clone)]
pub struct ButtonProps {
    #[props(default, into)]
    pub sx: Option<SxDyn>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}
