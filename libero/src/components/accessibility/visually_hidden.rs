use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, base_props},
        layout::use_box,
    },
    sx::{StaticSx, Sx, sx},
    theme::{PaperDefaults, Z_INDEX_FLOAT},
};

/// The sr-only recipe, `absolute`, for hidden inputs in a `relative` label:
/// Tab then scrolls the label into view; `fixed` did not (todo 63).
pub(crate) static VISUALLY_HIDDEN_SX: StaticSx = StaticSx::new(visually_hidden_sx);

/// [`VISUALLY_HIDDEN_SX`]'s recipe, for a selector inside another sheet.
pub(crate) fn visually_hidden_sx() -> Sx {
    sx().position("absolute")
        .width("1px")
        .height("1px")
        .padding("0")
        .margin("-1px")
        .overflow("hidden")
        .clip("rect(0, 0, 0, 0)")
        .white_space("nowrap")
        .border_width("0")
        // Blitz ignores `clip` and painted a speck and focus square (todo 757).
        .opacity("0")
}

/// For a control's `& > input`: over the drawn box's centre rather than 1px before
/// it, so a click at the input's own centre hits the box (todo 1992).
pub(crate) fn hidden_input_centred_sx() -> Sx {
    sx().top("50%").left("50%").margin("0")
}

/// The same recipe, `fixed`, for a box with no positioned host: an absolute one
/// would add to the page's scrollable overflow.
pub(crate) static VISUALLY_HIDDEN_FIXED_SX: StaticSx = StaticSx::new(fixed_recipe);

/// The fixed recipe, undone while focus is inside, for a skip link, on the float layer
/// above a sticky `Header` (2.4.11). Never the default, or a hidden native input would pop out.
static VISUALLY_HIDDEN_FOCUSABLE_SX: StaticSx = StaticSx::new(|| {
    fixed_recipe().selector(
        "&:focus-within",
        PaperDefaults::background_sx()
            .z_index(Z_INDEX_FLOAT.value())
            .width("auto")
            .height("auto")
            .margin("0")
            .overflow("visible")
            .clip_path("none")
            .white_space("normal"),
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

/// Content for assistive technology only.
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
///
/// Docs: <https://libero-ui.dev/accessibility/visually-hidden>
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A focused skip link must clear a sticky `Header` (WCAG 2.4.11).
    #[test]
    fn a_focused_skip_link_takes_the_float_layer() {
        let css = crate::css::Stylesheet::from(&*VISUALLY_HIDDEN_FOCUSABLE_SX);
        let css = css.as_str();
        let focused = &css[css.find(":focus-within{").expect(css)..];

        assert!(focused.contains("z-index:var(--lsx-z-index-float"), "{css}");
    }
}
