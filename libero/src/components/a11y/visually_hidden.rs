use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, common::base_props, layout::use_box},
    sx::{StaticSx, sx},
};

/// The shared sr-only recipe, `absolute`. The hidden inputs of `Checkbox`,
/// `Radio`, `Switch` and `Chip` sit in a `relative` label, so Tab onto one
/// scrolls its label into view. Under `fixed` it scrolled nothing, and the
/// label stayed off-screen (todo 63, measured).
pub(crate) static VISUALLY_HIDDEN_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .width("1px")
        .height("1px")
        .padding("0")
        .margin("-1px")
        .overflow("hidden")
        .clip("rect(0, 0, 0, 0)")
        .white_space("nowrap")
        .border_width("0")
});

/// The same recipe, `fixed`. For a box nothing hosts: an absolute span with no
/// positioned ancestor resolves against the page and adds to its scrollable
/// overflow, and a focusable one scrolls the document to reach it. A fixed box
/// is never overflow and scrolls nothing into view. `FocusTrapInitialFocus`
/// (whose `FocusTrap` is `display: contents`) and the `VisuallyHidden`
/// component (whose caller may supply no positioned parent) use it.
pub(crate) static VISUALLY_HIDDEN_FIXED_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .width("1px")
        .height("1px")
        .padding("0")
        .margin("-1px")
        .overflow("hidden")
        .clip_path("inset(50%)")
        .white_space("nowrap")
        .border_width("0")
});

base_props! {
    pub struct VisuallyHiddenProps {
        children: Element,
    }
}

/// Text for assistive technology only.
///
/// `fixed`, not the hosted inputs' `absolute`: it needs no positioned parent,
/// so it never makes the page scrollable wherever it lands.
#[component]
pub fn VisuallyHidden(props: VisuallyHiddenProps) -> Element {
    use_box()
        .framework_sx(&VISUALLY_HIDDEN_FIXED_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Span, props.attributes, props.children)
}
