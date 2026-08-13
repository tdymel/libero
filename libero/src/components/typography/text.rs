use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    sx::{StaticSx, Sx, ThemeAwareValue},
    theme::{Size, TextDefaults},
};

static TEXT_XS_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::xs_sx()
        .margin("0")
        .padding("0")
        .text_decoration("none")
});
static TEXT_SM_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::sm_sx()
        .margin("0")
        .padding("0")
        .text_decoration("none")
});
static TEXT_MD_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::md_sx()
        .margin("0")
        .padding("0")
        .text_decoration("none")
});
static TEXT_LG_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::lg_sx()
        .margin("0")
        .padding("0")
        .text_decoration("none")
});
static TEXT_XL_SX: StaticSx = StaticSx::new(|| {
    TextDefaults::xl_sx()
        .margin("0")
        .padding("0")
        .text_decoration("none")
});

fn get_size_sx(size: &ThemeAwareValue) -> &'static Sx {
    match size {
        ThemeAwareValue::Size(s) => match s {
            Size::Xs => &TEXT_XS_SX,
            Size::Sm => &TEXT_SM_SX,
            Size::Md => &TEXT_MD_SX,
            Size::Lg => &TEXT_LG_SX,
            Size::Xl => &TEXT_XL_SX,
        },
        _ => &TEXT_MD_SX,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TextProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    #[props(default)]
    span: Option<bool>,
    children: Element,
}

#[component]
pub fn Text(props: TextProps) -> Element {
    let effective_size = props
        .size
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::String("md".to_string()));
    let size_sx = get_size_sx(&effective_size);

    let framework_class = crate::context::use_sx(size_sx, crate::SxLayer::Framework);
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| crate::context::use_sx(sx, crate::SxLayer::UserStatic));

    let class = class_list([props.class, framework_class, static_class]);
    let data_state = props.states.as_ref().and_then(States::data_state);

    let use_span = props.span.unwrap_or(false);

    if use_span {
        rsx! {
            span {
                class: class,
                "data-state": data_state,
                ..props.attributes,
                {props.children}
            }
        }
    } else {
        rsx! {
            p {
                class: class,
                "data-state": data_state,
                ..props.attributes,
                {props.children}
            }
        }
    }
}
