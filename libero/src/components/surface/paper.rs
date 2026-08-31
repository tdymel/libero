use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, Variables, common::base_props, layout::use_box},
    sx::{StaticSx, Sx, sx},
    theme::{PAPER_BORDER_COLOR, PaperDefaults, Size},
};

/// The library's one definition of a surface, as an `Sx` to build on.
///
/// Public because [`PaperProps::framework_sx`] is: that prop **replaces**
/// `Paper`'s own base rather than layering on it, so without this a caller
/// outside the crate could only use it to delete the surface.
///
/// A component that renders a surface in bulk, or that would rather not pay
/// for a `Paper` scope, starts its own base static here and chains its own
/// declarations on top - `Dialog` is the worked example. Everything else
/// renders a [`Paper`].
///
/// The `radius`/`shadow` steps arrive as `data-state` tokens and the themed
/// defaults as plain declarations, so chaining `border_radius`/`box_shadow`
/// on top of this overrides the default without fighting a `[data-state]`
/// block's specificity.
pub fn paper_sx() -> Sx {
    PaperDefaults::theme_vars()
        // Mantine's two base declarations: a surface is a block, and it drops
        // the underline so `component: "a"` reads as a card, not as a link.
        .display("block")
        .text_decoration("none")
        .when(
            "bordered",
            sx().border(format!("1px solid {}", PAPER_BORDER_COLOR.value())),
        )
}

static PAPER_BASE_SX: StaticSx = StaticSx::new(paper_sx);

base_props! {
    // `href`/`target`: a surface rendered as an `<a>` is Mantine's clickable
    // card, and the base is written for it.
    extends(a);
    pub struct PaperProps {
        /// Corner radius, a step on the shared radius scale. An off-scale
        /// value goes through `sx` instead: `sx: sx().border_radius("2px")`.
        #[props(default, into)]
        radius: Input<Size>,
        /// Elevation, a step on the shared shadow scale. A flat surface is
        /// `sx: sx().box_shadow("none")`, the way `Image` spells it.
        #[props(default, into)]
        shadow: Input<Size>,
        /// A hairline border in the themed surface border colour. Legal
        /// together with a shadow - that is a design choice, not a misuse.
        #[props(default)]
        bordered: bool,
        /// Which element to render as - `div` by default. `section`,
        /// `article`, `aside` and `a` are the ones worth naming; a surface
        /// that becomes a landmark owns its own `aria-label`.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Per-instance CSS custom properties on the `style` attribute, for a
        /// component built on `Paper`.
        #[props(default, into)]
        variables: Input<Variables>,
        /// Base styles of a component built on `Paper`, on the framework
        /// layer. It **replaces** `Paper`'s own base rather than layering on
        /// it, so build it from [`paper_sx`].
        #[props(default)]
        framework_sx: Option<&'static StaticSx>,
        children: Element,
    }
}

/// A surface: a background, a corner radius, an elevation, optionally a
/// border. No role, no ARIA and nothing focusable - a surface is
/// presentational, and the contents are what a reader interacts with.
///
/// ```ignore
/// Paper { shadow: "sm", radius: "md", sx: sx().padding("lg"),
///     Title { size: "md", "Invoice #4021" }
///     Text { "Due 30 September." }
/// }
/// ```
#[component]
pub fn Paper(props: PaperProps) -> Element {
    let states = paper_states(&props);

    use_box()
        .framework_sx(props.framework_sx.unwrap_or(&PAPER_BASE_SX))
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&props.variables)
        .prepare()
        .render(
            props.component.copied_or_default(),
            props.attributes,
            props.children,
        )
}

/// The caller's states plus a token per axis the caller actually set.
///
/// **This deliberately diverges from `Button`**, which always emits its
/// `size`/`radius` token and defaults it from the theme. `Paper` emits one
/// only for a step the caller named; with the prop unset there is no token at
/// all and the themed default arrives as a plain `border-radius:
/// var(--lsx-paper-radius)` declaration from [`paper_sx`].
///
/// The difference is not an omission. A fold is a `[data-state~="radius-lg"]`
/// block at 0-2-0 and beats a top-level declaration whatever the source order,
/// so a component building its own base static from [`paper_sx`] - `Dialog`,
/// and every later surface - could never override the radius or the shadow if
/// `Paper` always emitted one. The rule: a gate others derive their base from
/// emits a token only for an explicit step; a leaf component can always emit.
fn paper_states(props: &PaperProps) -> Input<States> {
    let radius = props.radius.as_ref();
    let shadow = props.shadow.as_ref();

    if radius.is_none() && shadow.is_none() && !props.bordered {
        return props.states.clone();
    }

    let mut states = props.states.clone().unwrap_or_default();
    if let Some(radius) = radius {
        states = states.active(radius.radius_state_name());
    }
    if let Some(shadow) = shadow {
        states = states.active(shadow.shadow_state_name());
    }
    states.with("bordered", props.bordered).into()
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
        // `background()` publishes this for free only for a colour it can
        // read; a `var()` is opaque to it, so the surface owes it by hand.
        assert!(
            css.contains("--lsx-focus-contrast:var(--lsx-paper-contrast);"),
            "{css}"
        );
    }
}
