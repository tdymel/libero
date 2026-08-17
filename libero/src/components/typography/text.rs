use dioxus::prelude::*;

use crate::{
    components::{Box, HtmlTag, Input, States, common::base_props},
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

fn get_size_sx(size: &ThemeAwareValue) -> &'static StaticSx {
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

base_props! {
    pub struct TextProps {
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// Which element to render as - `p` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        children: Element,
    }
}

#[component]
pub fn Text(props: TextProps) -> Element {
    let effective_size = props
        .size
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::String("md".to_string()));
    let size_sx = get_size_sx(&effective_size);
    let component = props.component.as_ref().copied().unwrap_or(HtmlTag::P);

    rsx! {
        Box {
            class: props.class,
            sx: props.sx,
            states: props.states,
            component,
            framework_sx: size_sx,
            attributes: props.attributes,
            {props.children}
        }
    }
}
