use dioxus::prelude::*;

use crate::{
    components::{
        Box, HtmlTag, Input, States,
        common::{base_props, class_list, focus_ring_sx},
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::ContainerDefaults,
};

static CONTAINER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .height("100%")
        .margin_left("auto")
        .margin_right("auto")
        .and(ContainerDefaults::default_sx())
        .focus_visible(focus_ring_sx())
});

fn container_dynamic_sx(props: &ContainerProps) -> Sx {
    sx().apply_if(props.size.as_ref(), |sx, size| sx.max_width(size.clone()))
        .apply_if(props.gutters.as_ref(), |sx, gutters| {
            sx.padding_left(gutters.clone())
                .padding_right(gutters.clone())
        })
}

base_props! {
    pub struct ContainerProps {
        /// Which element to render as - `div` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        gutters: Input<ThemeAwareValue>,
        children: Element,
    }
}

#[component]
pub fn Container(props: ContainerProps) -> Element {
    let dynamic_class =
        crate::hooks::use_css(&container_dynamic_sx(&props), crate::CssLayer::UserDynamic);

    rsx! {
        Box {
            component: props.component,
            class: class_list([props.class, dynamic_class]),
            sx: props.sx,
            states: props.states,
            framework_sx: &CONTAINER_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
