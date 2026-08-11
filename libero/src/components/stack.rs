use dioxus::prelude::*;

use crate::{
    components::{Box, util::classes},
    css::SizeCssVar,
    sx::{Sx, sx},
    theme::Size,
};

pub trait StackValue {
    fn as_str(&self) -> &'static str;
}

impl StackValue for &'static str {
    fn as_str(&self) -> &'static str {
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackAlign {
    Start,
    Center,
    End,
    Stretch,
}

impl StackValue for StackAlign {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "flex-start",
            Self::Center => "center",
            Self::End => "flex-end",
            Self::Stretch => "stretch",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackJustify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

impl StackValue for StackJustify {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Start => "flex-start",
            Self::Center => "center",
            Self::End => "flex-end",
            Self::Between => "space-between",
            Self::Around => "space-around",
            Self::Evenly => "space-evenly",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackGap {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl StackValue for StackGap {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }
}

const STACK_BASE_SX: Sx = sx()
    .display("flex")
    .flex_direction("column")
    .align_items("var(--lsx-stack-align)")
    .justify_content("var(--lsx-stack-justify)")
    .gap("var(--lsx-stack-gap)")
    .build();

#[derive(Props, Clone, PartialEq)]
pub struct StackProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default)]
    sx: Option<&'static Sx>,
    #[props(default)]
    states: Vec<(&'static str, bool)>,
    #[props(default)]
    align: Option<&'static str>,
    #[props(default)]
    justify: Option<&'static str>,
    #[props(default)]
    gap: Option<&'static str>,
    children: Element,
}

#[component]
pub fn Stack(props: StackProps) -> Element {
    crate::context::use_sx(&STACK_BASE_SX);

    let gap = props.gap.unwrap_or("md");
    let variables = vec![
        ("stack-align", props.align.unwrap_or("stretch").to_string()),
        (
            "stack-justify",
            props.justify.unwrap_or("flex-start").to_string(),
        ),
        (
            "stack-gap",
            match Size::parse(gap) {
                Some(size) => SizeCssVar::SPACING.value(size),
                None => gap.to_string(),
            },
        ),
    ];

    let class = classes(props.class, STACK_BASE_SX.class_name());

    rsx! {
        Box {
            attributes: props.attributes,
            class: class,
            sx: props.sx,
            states: props.states,
            variables: variables,
            {props.children}
        }
    }
}
