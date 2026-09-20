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
        /// Font size; the rest of the look is `Theme::kbd`.
        #[props(default, into)]
        size: Input<Size>,
        children: Element,
    }
}

/// A single keyboard key, rendered as a `<kbd>`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Kbd;
/// # fn app() -> Element {
/// rsx! {
///     Kbd { "Ctrl" }
///     " + "
///     Kbd { "K" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/kbd>
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
