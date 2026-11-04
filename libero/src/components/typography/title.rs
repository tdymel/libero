use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            Cut, HtmlTag, Input, States, Truncate, base_props, clamp_height_sx, clamp_sx,
            gradient_text_sx, truncate_sx, with_own,
        },
        layout::use_box,
    },
    hooks::{use_gradient_style, use_range_css, use_theme},
    sx::{StaticSx, sx},
    theme::{Gradient, Responsive, Size, TITLE_LINE_HEIGHT, TitleDefaults},
    utils::{Warned, use_warn_once},
};

static TITLE_BASE_SX: StaticSx = StaticSx::new(|| {
    TitleDefaults::theme_vars()
        .and(truncate_sx(TITLE_LINE_HEIGHT))
        .margin("0")
        // A long word at `xxl` is wider than a 320px column (WCAG 1.4.10). Unlike
        // `break-word`, `anywhere` also shrinks min-content, so a flex parent stays narrow.
        .overflow_wrap("anywhere")
        // A link in a title takes the title's type, not `Anchor`'s own Text sizing (todo 5359).
        .selector(
            "& :is(a, button)[data-state*=\"underline-\"]",
            sx().font_family("inherit")
                .font_size("inherit")
                .font_weight("inherit")
                .letter_spacing("inherit")
                .line_height("inherit"),
        )
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
/// Once per tag, so a later change of `component` warns too (todo 5038).
fn warn_unless_heading(warned: &Warned, component: HtmlTag) {
    use HtmlTag::{H1, H2, H3, H4, H5, H6};
    if !matches!(component, H1 | H2 | H3 | H4 | H5 | H6) {
        warned.once(component.as_str(), || {
            format!(
                "Title: component `{}` is not a heading, so it is missing from the outline; \
                 use h1 to h6, or `Text` for the look alone",
                component.as_str()
            )
        });
    }
}

base_props! {
    pub struct TitleProps {
        /// Per breakpoint too: `responsive(Size::Lg).md(Size::Xxl)`; the tag follows the base size.
        #[props(default, into)]
        size: Input<Responsive<Size>>,
        /// Unset, `size`'s heading tag. Set it to keep the outline order under another size;
        /// a tag other than h1 to h6 warns in debug builds.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Paints the glyphs in a gradient from the theme's first stop: `("secondary", 45)` or
        /// a [`Gradient`]. A literal stop's contrast is the caller's to check.
        #[props(default, into)]
        gradient: Option<Gradient>,
        /// Cuts a title on one line with an ellipsis at its `end` or `start`.
        /// `line_clamp` wins when both are set. Blitz paints no ellipsis: it clips the text at the cut end.
        #[props(default, into)]
        truncate: Input<Truncate>,
        /// Cuts the title after this many lines with an ellipsis; over `truncate`, and `0`
        /// is off. Blitz has no line clamp: it cuts at the line height, without an ellipsis.
        /// Per breakpoint too, `responsive(2).md(0)`, where `0` lifts the clamp in that range.
        #[props(default, into)]
        line_clamp: Input<Responsive<u32>>,
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
    let sizes = props.size.copied_or(theme.title.size.into());
    let style = use_gradient_style(
        props.gradient.as_ref(),
        None,
        props.gradient.is_some(),
        true,
    );

    let cut = Cut::of(&props.truncate, &props.line_clamp);
    let states: Input<States> = cut
        .states()
        .into_iter()
        .fold(
            props
                .states
                .unwrap_or_default()
                .responsive(&sizes, |size| size.state_name())
                .with("gradient", props.gradient.is_some()),
            |states, (state, on)| states.with(state, on),
        )
        .into();
    let lines = cut.variables();
    let sx = with_own(clamp_sx(&props.line_clamp), &props.sx);
    use_range_css(
        &TITLE_BASE_SX,
        sizes.range_copies(
            |size| size.state_name(),
            |size| TitleDefaults::size_sx(size).and(clamp_height_sx(TITLE_LINE_HEIGHT, size)),
        ),
    );
    let children = match cut.is_start() {
        true => rsx! { bdi { {props.children} } },
        false => props.children,
    };

    let component = props
        .component
        .as_ref()
        .copied()
        // The caller's base size, never the theme's: a theme must not move the outline.
        .unwrap_or_else(|| {
            default_component(props.size.as_ref().map_or(Size::Xxl, Responsive::base))
        });
    let warned = use_warn_once();
    warn_unless_heading(&warned, component);

    use_box()
        .framework_sx(&TITLE_BASE_SX)
        .class(&props.class)
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(&sx)
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

    thread_local! {
        static TAG: std::cell::Cell<Option<Signal<&'static str>>> = const { std::cell::Cell::new(None) };
    }

    #[component]
    fn Switching() -> Element {
        let tag = use_signal(|| "h2");
        TAG.with(|cell| cell.set(Some(tag)));
        rsx! { LiberoProvider { Title { component: tag(), "Turns" } } }
    }

    #[test]
    fn a_component_changed_after_mount_warns() {
        crate::utils::take_warnings();
        let mut dom = VirtualDom::new(Switching);
        dom.rebuild_in_place();
        assert!(crate::utils::take_warnings().is_empty());
        let mut tag = TAG.with(|cell| cell.get()).expect("rendered");
        dom.in_runtime(|| tag.set("div"));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let warnings = crate::utils::take_warnings();
        assert!(
            warnings.iter().any(|w| w.starts_with("Title:")),
            "{warnings:?}"
        );
    }
}
