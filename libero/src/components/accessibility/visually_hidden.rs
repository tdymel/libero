use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, common::base_props, layout::use_box},
    sx::{StaticSx, Sx, sx},
    theme::PAPER_BACKGROUND,
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
        // Blitz ignores `clip`: it painted the input as a speck and its focus
        // outline as a blue square (todo 757).
        .opacity("0")
});

/// The same recipe, `fixed`. For a box nothing hosts: an absolute span with no
/// positioned ancestor resolves against the page and adds to its scrollable
/// overflow, and a focusable one scrolls the document to reach it. A fixed box
/// is never overflow and scrolls nothing into view. `FocusTrapInitialFocus`
/// (whose `FocusTrap` is `display: contents`) and the `VisuallyHidden`
/// component (whose caller may supply no positioned parent) use it.
pub(crate) static VISUALLY_HIDDEN_FIXED_SX: StaticSx = StaticSx::new(fixed_recipe);

/// The fixed recipe, undone while focus is inside: a skip link shows itself on
/// focus. Never the default, or a hidden native input would pop out.
static VISUALLY_HIDDEN_FOCUSABLE_SX: StaticSx = StaticSx::new(|| {
    fixed_recipe().selector(
        "&:focus-within",
        sx().width("auto")
            .height("auto")
            .margin("0")
            .overflow("visible")
            .clip_path("none")
            .white_space("normal")
            .background(PAPER_BACKGROUND.value()),
    )
});

fn fixed_recipe() -> Sx {
    sx().position("fixed")
        .width("1px")
        .height("1px")
        .padding("0")
        .margin("-1px")
        .overflow("hidden")
        .clip_path("inset(50%)")
        .white_space("nowrap")
        .border_width("0")
}

base_props! {
    pub struct VisuallyHiddenProps {
        /// Shows the content while focus is inside it, for a skip link.
        #[props(default)]
        focusable: bool,
        children: Element,
    }
}

/// Text for assistive technology only.
///
/// `fixed`, not the hosted inputs' `absolute`: it needs no positioned parent,
/// so it never makes the page scrollable wherever it lands.
///
/// `focusable` reveals it while focus is inside, on a paper background. It
/// stays `fixed` at its place in the flow, so a skip link belongs at the top
/// of the page; further down, host it in a positioned parent and set
/// `position: absolute` through `sx`, or Tab never scrolls it into view.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::VisuallyHidden;
/// # fn app() -> Element {
/// rsx! {
///     VisuallyHidden { focusable: true,
///         a { href: "#main", "Skip to content" }
///     }
/// }
/// # }
/// ```
#[component]
pub fn VisuallyHidden(props: VisuallyHiddenProps) -> Element {
    let recipe = if props.focusable {
        &VISUALLY_HIDDEN_FOCUSABLE_SX
    } else {
        &VISUALLY_HIDDEN_FIXED_SX
    };

    use_box()
        .framework_sx(recipe)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Span, props.attributes, props.children)
}
