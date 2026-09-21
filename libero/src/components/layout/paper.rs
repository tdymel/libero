use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_color, base_props, contrast_color, fill_color,
            variables,
        },
        layout::use_box,
    },
    hooks::{use_glass_tint, use_gradient_style},
    platform::draws_backdrop_filter,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CssVar, FOCUS_RING_HALO, Gradient, NamedColorCss, PAPER_BORDER_COLOR, PaperDefaults, Size,
        gradient_surface_sx,
    },
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
        .when("colored", colored_sx())
        .when("gradient", gradient_surface_sx())
        // After `gradient`, which its shorthand would otherwise reset.
        .when(
            "glass",
            PaperDefaults::glass_sx()
                .when(
                    "colored",
                    PaperDefaults::glass_fill_sx(&PAPER_FILL.value(), Some(&PAPER_TINT_SHARE)),
                )
                .when("gradient", PaperDefaults::glass_gradient_sx()),
        )
}

// Own vars, not `PAPER_BACKGROUND`: an uncoloured `Paper` nested inside keeps the surface.
const PAPER_FILL: CssVar = CssVar::new("--lsx-paper-fill");
const PAPER_FILL_CONTRAST: CssVar = CssVar::new("--lsx-paper-fill-contrast");
/// A glass tint's share of the fill, raised past `glass_background` where the label needs it.
const PAPER_TINT_SHARE: CssVar = CssVar::new("--lsx-paper-tint-share");

/// A `color` fill; a literal has no computed label, so the text inherits.
fn colored_sx() -> Sx {
    sx().background(PAPER_FILL.value())
        .color(PAPER_FILL_CONTRAST.value_or("inherit"))
        .var(FOCUS_RING_HALO, PAPER_FILL.value())
}

/// The palette `color` a glass tint is measured for; a literal's label is the caller's.
fn glass_tint_color(props: &PaperProps) -> Option<ThemeAwareValue> {
    let glass = props.glass && draws_backdrop_filter() && props.gradient.is_none();
    let base = base_color(Some(props.color.as_ref().filter(|_| glass)?));
    contrast_color(&base).is_some().then_some(base)
}

/// The fill and label of `color`, unless a `gradient` takes it as its first stop.
/// `tint`: the label and share of a glass tint, which replace the solid fill's label.
fn paper_variables(props: &PaperProps, tint: Option<(String, u8)>) -> Option<String> {
    if props.gradient.is_some() {
        return None;
    }
    let base = base_color(Some(props.color.as_ref()?));
    let (contrast, share) = match tint {
        Some((label, share)) => (Some(label), Some(format!("{share}%"))),
        None => (contrast_color(&base).and_then(|c| c.resolve(None)), None),
    };
    Some(
        variables()
            .with(PAPER_FILL, fill_color(&base))
            .with(PAPER_FILL_CONTRAST, contrast.clone())
            .with(PAPER_TINT_SHARE, share)
            .with(
                CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
                contrast,
            )
            .render(),
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
        /// A fill: a palette colour paints its shade 6 under a computed label; a literal
        /// CSS colour is used as given, its label the caller's. The first stop under a `gradient`.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// A linear gradient fill from `color`: `("secondary", 45)` or a [`Gradient`];
        /// `Gradient::default()` is the theme's.
        #[props(default, into)]
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
    let gradient = use_gradient_style(
        props.gradient.as_ref(),
        props.color.as_ref(),
        props.gradient.is_some(),
        false,
    );
    let tint = use_glass_tint(glass_tint_color(&props).as_ref());
    // At most one is set: a gradient takes `color` as its first stop.
    let style = paper_variables(&props, tint).or(gradient);

    use_box()
        .framework_sx(props.framework_sx.unwrap_or(&PAPER_BASE_SX))
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&props.variables)
        .style(style)
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
    let colored = props.color.as_ref().is_some() && !gradient;

    if radius.is_none() && shadow.is_none() && !props.bordered && !glass && !gradient && !colored {
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
        .with("colored", colored)
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
            color: Input::None,
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

    /// A palette colour paints its fill shade under the computed label.
    #[test]
    fn a_palette_color_brings_its_fill_and_label() {
        let props = PaperProps {
            color: Input::Value("primary".into()),
            ..props()
        };
        let style = paper_variables(&props, None).unwrap();

        assert!(
            style.contains("--lsx-paper-fill:var(--lsx-primary-fill-6);"),
            "{style}"
        );
        assert!(
            style.contains("--lsx-paper-fill-contrast:var(--lsx-primary-contrast-6);"),
            "{style}"
        );
        assert!(style.contains("--lsx-focus-contrast:"), "{style}");
        assert_eq!(
            paper_states(&props)
                .as_ref()
                .and_then(States::data_state)
                .as_deref(),
            Some("colored")
        );
    }

    /// A literal is used as given; its label is the caller's.
    #[test]
    fn a_literal_color_is_used_as_given() {
        let props = PaperProps {
            color: Input::Value("#123456".into()),
            glass: true,
            ..props()
        };
        let style = paper_variables(&props, None).unwrap();

        assert!(style.contains("--lsx-paper-fill:#123456;"), "{style}");
        assert!(!style.contains("--lsx-paper-fill-contrast"), "{style}");
        assert_eq!(glass_tint_color(&props), None);
    }

    /// Under glass the tint's label and share replace the solid fill's label.
    #[test]
    fn a_glass_tint_brings_its_own_label_and_share() {
        let props = PaperProps {
            color: Input::Value("secondary".into()),
            glass: true,
            ..props()
        };
        let style = paper_variables(&props, Some(("#000000".into(), 95))).unwrap();

        assert!(
            style.contains("--lsx-paper-fill-contrast:#000000;"),
            "{style}"
        );
        assert!(style.contains("--lsx-paper-tint-share:95%;"), "{style}");
        assert!(style.contains("--lsx-focus-contrast:#000000;"), "{style}");
        assert_eq!(glass_tint_color(&props).is_some(), draws_backdrop_filter());
    }

    /// Under a gradient the colour is the first stop, not a flat fill.
    #[test]
    fn a_gradient_takes_the_color_as_its_first_stop() {
        let props = PaperProps {
            color: Input::Value("error".into()),
            gradient: Some(("info", 90).into()),
            ..props()
        };

        assert_eq!(paper_variables(&props, None), None);
        assert_eq!(
            paper_states(&props)
                .as_ref()
                .and_then(States::data_state)
                .as_deref(),
            Some("gradient")
        );
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
