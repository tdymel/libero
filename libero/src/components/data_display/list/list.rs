use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ListDefaults, Size},
};

static LIST_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .list_style("none")
        .margin("0")
        .padding("0")
});

fn get_size_sx(size: Size) -> &'static Sx {
    static XS: StaticSx = StaticSx::new(ListDefaults::xs_sx);
    static SM: StaticSx = StaticSx::new(ListDefaults::sm_sx);
    static MD: StaticSx = StaticSx::new(ListDefaults::md_sx);
    static LG: StaticSx = StaticSx::new(ListDefaults::lg_sx);
    static XL: StaticSx = StaticSx::new(ListDefaults::xl_sx);
    static XXL: StaticSx = StaticSx::new(ListDefaults::xxl_sx);

    match size {
        Size::Xs => &XS,
        Size::Sm => &SM,
        Size::Md => &MD,
        Size::Lg => &LG,
        Size::Xl => &XL,
        Size::Xxl => &XXL,
    }
}

base_props! {
    pub struct ListProps {
        /// Item gap and nested-list indent - `theme.list.size` (`Md`) by default.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        children: Element,
    }
}

#[component]
pub fn List(props: ListProps) -> Element {
    let theme = use_theme();
    let size = match props.size.as_ref() {
        Some(ThemeAwareValue::Size(size)) => *size,
        _ => theme.list.size,
    };
    let size_class = crate::hooks::use_css(get_size_sx(size), crate::CssLayer::Framework);

    rsx! {
        Box {
            component: "ul",
            class: props.class.unwrap_or_default().with(size_class),
            sx: props.sx,
            states: props.states,
            framework_sx: &LIST_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
