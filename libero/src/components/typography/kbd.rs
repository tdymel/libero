use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props},
        layout::use_box,
    },
    hooks::{use_range_css, use_theme},
    sx::StaticSx,
    theme::{KbdDefaults, Responsive, Size},
};

static KBD_BASE_SX: StaticSx = StaticSx::new(|| {
    KbdDefaults::theme_vars()
        .display("inline-block")
        .font_weight("700")
        .padding("0.12em 0.45em")
        .text_align("center")
        .white_space("nowrap")
});

base_props! {
    pub struct KbdProps {
        /// Font size; the rest of the look is `Theme::kbd`.
        /// Per breakpoint too: `responsive(Size::Sm).md(Size::Lg)`.
        #[props(default, into)]
        size: Input<Responsive<Size>>,
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
    let sizes = props.size.copied_or(theme.kbd.size.into());

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .responsive(&sizes, |size| size.state_name())
        .into();
    use_range_css(
        &KBD_BASE_SX,
        sizes.range_copies(|size| size.state_name(), KbdDefaults::size_sx),
    );

    use_box()
        .framework_sx(&KBD_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(HtmlTag::Kbd, props.attributes, props.children)
}
