use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    sx::{StaticSx, Sx},
    theme::{KbdDefaults, Size},
};

static KBD_BASE_SX: StaticSx = StaticSx::new(|| {
    KbdDefaults::theme_vars()
        .display("inline-block")
        .font_weight("700")
        .padding("0.12em 0.45em")
        .text_align("center")
});

base_props! {
    pub struct KbdProps {
        /// Font size. Everything else about the look is `Theme::kbd` only.
        #[props(default, into)]
        size: Input<Size>,
        children: Element,
    }
}

/// A single keyboard key, rendered as a real `<kbd>`.
#[component]
pub fn Kbd(props: KbdProps) -> Element {
    // Matches Mantine's own default.
    let size = props.size.copied_or(Size::Sm);

    let states = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true);

    rsx! {
        Box {
            component: "kbd",
            class: props.class,
            sx: props.sx,
            states,
            framework_sx: &KBD_BASE_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
