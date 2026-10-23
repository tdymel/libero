use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            Cut, HtmlTag, Input, States, Truncate, base_props, gradient_text_sx, truncate_sx,
        },
        layout::use_box,
    },
    hooks::{use_gradient_style, use_theme},
    sx::StaticSx,
    theme::{Gradient, Size, TITLE_LINE_HEIGHT, TitleDefaults},
    utils::warn,
};

static TITLE_BASE_SX: StaticSx = StaticSx::new(|| {
    TitleDefaults::theme_vars()
        .and(truncate_sx(TITLE_LINE_HEIGHT))
        .margin("0")
        // A long word at `xxl` is wider than a 320px column (WCAG 1.4.10).
        .overflow_wrap("break-word")
        .when("gradient", gradient_text_sx())
});

/// `Xxl` is h1, down to `Xs` as h6.
fn default_component(size: Size) -> HtmlTag {
    match size {
        Size::Xxl => HtmlTag::H1,
        Size::Xl => HtmlTag::H2,
        Size::Lg => HtmlTag::H3,
        Size::Md => HtmlTag::H4,
        Size::Sm => HtmlTag::H5,
        Size::Xs => HtmlTag::H6,
    }
}

/// Todo 2545: another tag keeps the heading look but leaves the outline silently.
fn warn_unless_heading(component: HtmlTag) {
    use HtmlTag::{H1, H2, H3, H4, H5, H6};
    if !matches!(component, H1 | H2 | H3 | H4 | H5 | H6) {
        warn(&format!(
            "Title: component `{}` is not a heading, so it is missing from the outline; \
             use h1 to h6, or `Text` for the look alone",
            component.as_str()
        ));
    }
}

base_props! {
    pub struct TitleProps {
        #[props(default, into)]
        size: Input<Size>,
        /// Unset, `size`'s heading tag. Set it to keep the outline order under another size;
        /// a tag other than h1 to h6 warns in debug builds.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Paints the glyphs in a gradient from the theme's first stop: `("secondary", 45)` or
        /// a [`Gradient`]. A literal stop's contrast is the caller's to check.
        #[props(default, into)]
        gradient: Option<Gradient>,
        /// Cuts a title on one line with an ellipsis at its `end` or `start`.
        /// `line_clamp` wins when both are set.
        #[props(default, into)]
        truncate: Input<Truncate>,
        /// Cuts the title after this many lines with an ellipsis; over `truncate`, and `0`
        /// is off. Blitz has no line clamp: it cuts at the line height, without an ellipsis.
        #[props(default)]
        line_clamp: Option<u32>,
        children: Element,
    }
}

/// A heading, `h1` to `h6` by `size`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Title;
/// # fn app() -> Element {
/// rsx! {
///     Title { size: "xl", "Getting started" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/typography/title>
#[component]
pub fn Title(props: TitleProps) -> Element {
    let theme = use_theme();
    let chosen_size = props.size.copied_or(theme.title.size);
    let style = use_gradient_style(
        props.gradient.as_ref(),
        None,
        props.gradient.is_some(),
        true,
    );

    let cut = Cut::of(&props.truncate, props.line_clamp);
    let states: Input<States> = cut
        .states()
        .into_iter()
        .fold(
            props
                .states
                .unwrap_or_default()
                .with(chosen_size.state_name(), true)
                .with("gradient", props.gradient.is_some()),
            |states, (state, on)| states.with(state, on),
        )
        .into();
    let lines = cut.variables();
    let children = match cut.is_start() {
        true => rsx! { bdi { {props.children} } },
        false => props.children,
    };

    let component = props
        .component
        .as_ref()
        .copied()
        // The caller's size, never the theme's: a theme must not move the outline.
        .unwrap_or_else(|| default_component(props.size.copied_or(Size::Xxl)));
    use_hook(|| warn_unless_heading(component));

    use_box()
        .framework_sx(&TITLE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&lines)
        .style(style)
        .prepare()
        .render(component, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LiberoProvider, utils::warnings_of};

    fn warns(app: fn() -> Element) -> bool {
        warnings_of(app)
            .iter()
            .any(|warning| warning.starts_with("Title:"))
    }

    #[test]
    fn only_a_tag_outside_h1_to_h6_warns() {
        assert!(warns(
            || rsx! { LiberoProvider { Title { component: "div", "Look only" } } }
        ));
        assert!(!warns(
            || rsx! { LiberoProvider { Title { component: "h2", "Second level" } } }
        ));
        assert!(!warns(|| rsx! { LiberoProvider { Title { "Default" } } }));
    }
}
