use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    sx::{StaticSx, Sx},
    theme::TitleDefaults,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleVariant {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl Default for TitleVariant {
    fn default() -> Self {
        Self::H1
    }
}

impl From<&str> for TitleVariant {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "h1" => Self::H1,
            "h2" => Self::H2,
            "h3" => Self::H3,
            "h4" => Self::H4,
            "h5" => Self::H5,
            "h6" => Self::H6,
            _ => Self::H1,
        }
    }
}

impl From<String> for TitleVariant {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<TitleVariant> {
    fn from(value: &str) -> Self {
        Input::Value(TitleVariant::from(value))
    }
}

impl From<String> for Input<TitleVariant> {
    fn from(value: String) -> Self {
        Input::Value(TitleVariant::from(value))
    }
}

static TITLE_H1_SX: StaticSx = StaticSx::new(|| TitleDefaults::h1_sx().margin("0"));
static TITLE_H2_SX: StaticSx = StaticSx::new(|| TitleDefaults::h2_sx().margin("0"));
static TITLE_H3_SX: StaticSx = StaticSx::new(|| TitleDefaults::h3_sx().margin("0"));
static TITLE_H4_SX: StaticSx = StaticSx::new(|| TitleDefaults::h4_sx().margin("0"));
static TITLE_H5_SX: StaticSx = StaticSx::new(|| TitleDefaults::h5_sx().margin("0"));
static TITLE_H6_SX: StaticSx = StaticSx::new(|| TitleDefaults::h6_sx().margin("0"));

fn get_size_sx(variant: TitleVariant) -> &'static Sx {
    match variant {
        TitleVariant::H1 => &TITLE_H1_SX,
        TitleVariant::H2 => &TITLE_H2_SX,
        TitleVariant::H3 => &TITLE_H3_SX,
        TitleVariant::H4 => &TITLE_H4_SX,
        TitleVariant::H5 => &TITLE_H5_SX,
        TitleVariant::H6 => &TITLE_H6_SX,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TitleProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    variant: TitleVariant,
    #[props(default, into)]
    size: Input<TitleVariant>,
    children: Element,
}

#[component]
pub fn Title(props: TitleProps) -> Element {
    let effective_size = props.size.as_ref().copied().unwrap_or(props.variant);
    let size_sx = get_size_sx(effective_size);

    let framework_class = crate::context::use_sx(size_sx, crate::SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));

    let class = class_list([props.class, framework_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    match props.variant {
        TitleVariant::H1 => {
            rsx! {
                h1 {
                    class: class,
                    "data-state": data_state,
                    ..props.attributes,
                    {props.children}
                }
            }
        }
        TitleVariant::H2 => {
            rsx! {
                h2 {
                    class: class,
                    "data-state": data_state,
                    ..props.attributes,
                    {props.children}
                }
            }
        }
        TitleVariant::H3 => {
            rsx! {
                h3 {
                    class: class,
                    "data-state": data_state,
                    ..props.attributes,
                    {props.children}
                }
            }
        }
        TitleVariant::H4 => {
            rsx! {
                h4 {
                    class: class,
                    "data-state": data_state,
                    ..props.attributes,
                    {props.children}
                }
            }
        }
        TitleVariant::H5 => {
            rsx! {
                h5 {
                    class: class,
                    "data-state": data_state,
                    ..props.attributes,
                    {props.children}
                }
            }
        }
        TitleVariant::H6 => {
            rsx! {
                h6 {
                    class: class,
                    "data-state": data_state,
                    ..props.attributes,
                    {props.children}
                }
            }
        }
    }
}
