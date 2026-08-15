use dioxus::prelude::*;

use crate::{
    components::{Box, HtmlTag, Input, States},
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

impl From<TitleVariant> for HtmlTag {
    fn from(variant: TitleVariant) -> Self {
        match variant {
            TitleVariant::H1 => Self::H1,
            TitleVariant::H2 => Self::H2,
            TitleVariant::H3 => Self::H3,
            TitleVariant::H4 => Self::H4,
            TitleVariant::H5 => Self::H5,
            TitleVariant::H6 => Self::H6,
        }
    }
}

static TITLE_H1_SX: StaticSx = StaticSx::new(|| TitleDefaults::h1_sx().margin("0"));
static TITLE_H2_SX: StaticSx = StaticSx::new(|| TitleDefaults::h2_sx().margin("0"));
static TITLE_H3_SX: StaticSx = StaticSx::new(|| TitleDefaults::h3_sx().margin("0"));
static TITLE_H4_SX: StaticSx = StaticSx::new(|| TitleDefaults::h4_sx().margin("0"));
static TITLE_H5_SX: StaticSx = StaticSx::new(|| TitleDefaults::h5_sx().margin("0"));
static TITLE_H6_SX: StaticSx = StaticSx::new(|| TitleDefaults::h6_sx().margin("0"));

fn get_size_sx(variant: TitleVariant) -> &'static StaticSx {
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

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states: props.states,
            component: HtmlTag::from(props.variant),
            framework_sx: get_size_sx(effective_size),
            attributes: props.attributes,
            {props.children}
        }
    }
}
