use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    hooks::use_theme,
    sx::StaticSx,
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
    let theme = use_theme();
    let size = props.size.copied_or(theme.kbd.size);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .into();

    use_box()
        .framework_sx(&KBD_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(HtmlTag::Kbd, props.attributes, props.children)
}
