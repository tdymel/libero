use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, base_props},
        layout::use_box,
    },
    hooks::use_gradient_style,
    platform::draws_backdrop_filter,
    sx::{StaticSx, Sx, sx},
    theme::{Gradient, PAPER_BORDER_COLOR, PaperDefaults, Size, gradient_surface_sx},
};

/// The library's surface as an `Sx`, to build a [`PaperProps::framework_sx`] on,
/// which replaces `Paper`'s own base. Chained declarations override its defaults.
///
/// ```
/// # use libero::{components::paper_sx, sx::StaticSx};
/// static CARD_SX: StaticSx = StaticSx::new(|| paper_sx().padding("lg"));
/// ```
pub fn paper_sx() -> Sx {
    PaperDefaults::theme_vars()
        // No underline, so `component: "a"` reads as a card, not as a link.
        .display("block")
        .text_decoration("none")
        .when(
            "bordered",
            sx().border(format!("1px solid {}", PAPER_BORDER_COLOR.value())),
        )
        .when("gradient", gradient_surface_sx())
        // After `gradient`, which its shorthand would otherwise reset.
        .when(
            "glass",
            PaperDefaults::glass_sx().when("gradient", PaperDefaults::glass_gradient_sx()),
        )
}

static PAPER_BASE_SX: StaticSx = StaticSx::new(paper_sx);

base_props! {
    // `href`/`target`: a surface rendered as an `<a>` is a clickable card.
    extends(a);
    pub struct PaperProps {
        /// Corner radius, a step on the radius scale.
        #[props(default, into)]
        radius: Input<Size>,
        /// Elevation, a step on the shadow scale.
        #[props(default, into)]
        shadow: Input<Size>,
        /// A hairline border in the themed surface border colour.
        #[props(default)]
        bordered: bool,
        /// Frosted glass; opaque where it can't blur (natively, reduced transparency).
        #[props(default)]
        glass: bool,
        /// A linear gradient fill; `Gradient::default()` is the theme's.
        #[props(default)]
        gradient: Option<Gradient>,
        /// Which element to render as - `div` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Per-instance CSS custom properties, for a component built on `Paper`.
        #[props(default, into)]
        variables: Input<Variables>,
        /// Replaces `Paper`'s base styles; build it from [`paper_sx`].
        #[props(default)]
        framework_sx: Option<&'static StaticSx>,
        children: Element,
    }
}

/// A surface: a background, a corner radius, an elevation, optionally a border.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Paper, Text, Title};
/// # use libero::sx::sx;
/// # fn app() -> Element {
/// rsx! {
///     Paper { shadow: "sm", radius: "md", sx: sx().padding("lg"),
///         Title { size: "md", "Invoice #4021" }
///         Text { "Due 30 September." }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/paper>
#[component]
pub fn Paper(props: PaperProps) -> Element {
    let states = paper_states(&props);
    let gradient = use_gradient_style(props.gradient.as_ref(), true, false);

    use_box()
        .framework_sx(props.framework_sx.unwrap_or(&PAPER_BASE_SX))
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&props.variables)
        .style(gradient)
        .prepare()
        .render(
            props.component.copied_or_default(),
            props.attributes,
            props.children,
        )
}

/// A token only per axis the caller set, unlike `Button`: a `[data-state]` block
/// would beat the radius or shadow a surface built on [`paper_sx`] chains on.
fn paper_states(props: &PaperProps) -> Input<States> {
    let radius = props.radius.as_ref();
    let shadow = props.shadow.as_ref();
    let glass = props.glass && draws_backdrop_filter();
    let gradient = props.gradient.is_some();

    if radius.is_none() && shadow.is_none() && !props.bordered && !glass && !gradient {
        return props.states.clone();
    }

    let mut states = props.states.clone().unwrap_or_default();
    if let Some(radius) = radius {
        states = states.active(radius.radius_state_name());
    }
    if let Some(shadow) = shadow {
        states = states.active(shadow.shadow_state_name());
    }
    states
        .with("bordered", props.bordered)
        .with("glass", glass)
        .with("gradient", gradient)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    fn props() -> PaperProps {
        PaperProps {
            radius: Input::None,
            shadow: Input::None,
            bordered: false,
            glass: false,
            gradient: None,
            component: Input::None,
            variables: Input::None,
            framework_sx: None,
            children: rsx! {},
            attributes: Vec::new(),
            class: Input::None,
            sx: Input::None,
            states: Input::None,
        }
    }

    #[test]
    fn a_bare_paper_carries_no_data_state() {
        assert!(matches!(paper_states(&props()), Input::None));
    }

    #[test]
    fn each_axis_gets_its_own_namespaced_token() {
        let states = paper_states(&PaperProps {
            radius: Input::Value(Size::Lg),
            shadow: Input::Value(Size::Xl),
            bordered: true,
            ..props()
        });

        assert_eq!(
            states.as_ref().and_then(States::data_state).as_deref(),
            Some("radius-lg shadow-xl bordered")
        );
    }

    #[test]
    fn the_caller_s_own_states_survive() {
        let states = paper_states(&PaperProps {
            states: Input::Value(States::new().active("selected")),
            radius: Input::Value(Size::Sm),
            ..props()
        });

        assert_eq!(
            states.as_ref().and_then(States::data_state).as_deref(),
            Some("selected radius-sm")
        );
    }

    /// Natively there is no `glass` token at all, so the surface stays opaque.
    #[test]
    fn glass_is_a_token_only_where_the_renderer_blurs() {
        let states = paper_states(&PaperProps {
            glass: true,
            ..props()
        });
        let state = states.as_ref().and_then(States::data_state);

        if draws_backdrop_filter() {
            assert_eq!(state.as_deref(), Some("glass"));
        } else {
            assert_eq!(state, None);
        }
    }

    #[test]
    fn glass_turns_opaque_under_reduced_transparency_and_forced_colours() {
        let css = Stylesheet::from(&paper_sx());
        let css = css.as_str();

        assert!(
            css.contains("background:var(--lsx-glass-background);"),
            "{css}"
        );
        assert!(
            css.contains("backdrop-filter:var(--lsx-glass-blur);"),
            "{css}"
        );
        for query in [
            "prefers-reduced-transparency:reduce",
            "forced-colors:active",
        ] {
            let block = css
                .split("@media")
                .find(|block| block.replace(' ', "").contains(query))
                .unwrap_or_else(|| panic!("no {query} block: {css}"));
            assert!(block.contains("glass"), "{block}");
            assert!(block.contains("backdrop-filter:none;"), "{block}");
            assert!(
                block.contains("background:var(--lsx-paper-background);"),
                "{block}"
            );
        }
    }

    /// Rings inside a gradient take its label and first stop, as in `Header`.
    #[test]
    fn a_gradient_hands_its_label_to_rings_inside() {
        let css = Stylesheet::from(&paper_sx());
        let gradient = css
            .as_str()
            .split('}')
            .find(|block| block.contains("gradient") && block.contains("background-image"))
            .unwrap_or_else(|| panic!("no gradient block: {}", css.as_str()));

        assert!(
            gradient.contains("--lsx-focus-contrast:var(--lsx-gradient-contrast);"),
            "{gradient}"
        );
        assert!(
            gradient.contains("--lsx-focus-ring-halo:var(--lsx-gradient-from);"),
            "{gradient}"
        );
    }

    #[test]
    fn the_surface_reads_the_themed_background_rather_than_a_literal() {
        let css = Stylesheet::from(&paper_sx());
        let css = css.as_str();

        assert!(
            css.contains("background:var(--lsx-paper-background);"),
            "{css}"
        );
        assert!(
            css.contains("border-radius:var(--lsx-paper-radius);"),
            "{css}"
        );
        assert!(css.contains("box-shadow:var(--lsx-paper-shadow);"), "{css}");
        assert!(
            css.contains("border:1px solid var(--lsx-paper-border-color);"),
            "{css}"
        );
        // `background()` can't read a `var()`, so the surface publishes it by hand.
        assert!(
            css.contains("--lsx-focus-contrast:var(--lsx-paper-contrast);"),
            "{css}"
        );
        assert!(
            css.contains("--lsx-focus-ring-halo:var(--lsx-paper-background);"),
            "{css}"
        );
    }
}
