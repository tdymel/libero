use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{attr, base_props, use_style_attributes},
        layout::box_style,
        variables,
    },
    hooks::{use_cache, use_element, use_theme},
    platform::{ElementApi, prefers_reduced_motion},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss},
};

use super::render_anchor;

// A literal hex/css color has no derivable "lighter shade", so it falls
// through to the theme's default.
fn nav_link_color(value: Option<&ThemeAwareValue>) -> Option<Color> {
    match value {
        Some(ThemeAwareValue::Color(color)) => Some(*color),
        Some(ThemeAwareValue::ColorValue(
            ColorValue::Shade(color, _) | ColorValue::Contrast(color, _),
        )) => Some(*color),
        _ => None,
    }
}

/// An in-page anchor adds a `#fragment` to the route without leaving the page,
/// so it's ignored - unless `to` names a fragment itself, then it must match.
fn route_matches(route: &str, to: &str) -> bool {
    if to.contains('#') {
        return route == to;
    }
    route.split('#').next() == Some(to)
}

const NAV_LINK_ACTIVE_BACKGROUND_VAR: CssVar = CssVar::new("--lsx-nav-link-active-background");

fn nav_link_variables(color: Option<&ThemeAwareValue>, default_color: Color) -> Variables {
    let base = nav_link_color(color).unwrap_or(default_color);
    let background = ThemeAwareValue::ColorValue(ColorValue::Shade(base, ColorShade::S1));

    variables().with(NAV_LINK_ACTIVE_BACKGROUND_VAR, background.resolve(None))
}

static NAV_LINK_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .width("100%")
        .padding("8px 12px")
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .color("inherit")
        .text_decoration("none")
        .cursor("pointer")
        // A long unbreakable label wraps instead of widening the page (1.4.10).
        .with("overflow-wrap", "anywhere")
        // Keeps `scroll_into_view`'s `Nearest` off the container's edge.
        // Affects where a scroll lands, never whether one happens.
        .scroll_margin("8rem")
        // Hover is neutral grey, not `color`: it shouldn't preview the
        // selected look. Active gets the light color tint instead.
        .hover(sx().background("muted.2"))
        .when(
            "active",
            sx().background(NAV_LINK_ACTIVE_BACKGROUND_VAR.value())
                .hover(sx().background(NAV_LINK_ACTIVE_BACKGROUND_VAR.value())),
        )
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
});

base_props! {
    pub struct NavLinkProps {
        /// A plain path/URL or a typed route, same as `Anchor::to`.
        #[props(into)]
        to: NavigationTarget,
        #[props(default)]
        target: Option<String>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Tints the link. Unset, it compares `to` against the current route,
        /// so it is only ever true for an `Internal` target with a router
        /// mounted. Set it explicitly for a section-level parent item, or
        /// anywhere auto-detection has nothing to compare against.
        #[props(default)]
        active: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// Scrolls this link into view when it becomes active, if it isn't
        /// already visible. Off by default - it acts on whatever scrollable
        /// ancestor happens to exist, which only suits a sidebar.
        #[props(default)]
        scroll_into_view: Option<bool>,
        children: Element,
    }
}

/// A navigation list item - `Anchor` plus a themed active/hover background
/// and `aria-current`, for a sidebar or nav bar link.
#[component]
pub fn NavLink(props: NavLinkProps) -> Element {
    let theme = use_theme();
    let disabled = props.disabled.unwrap_or(false);
    let is_active = props.active.unwrap_or_else(|| match &props.to {
        NavigationTarget::Internal(path) => try_router()
            .map(|router| route_matches(&router.full_route_string(), path))
            .unwrap_or(false),
        NavigationTarget::External(_) => false,
    });

    let style = use_cache(
        (props.color.clone(), theme.nav_link.color),
        |(color, default_color)| nav_link_variables(color.as_ref(), *default_color).render(),
    );
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("disabled", disabled)
        .with("active", is_active)
        .into();
    let aria_current = is_active.then_some("page");

    let scroll_into_view = props.scroll_into_view.unwrap_or(false);
    let element = use_element();

    // `is_active` is a plain bool, hence `use_reactive!`; `element` is read
    // through a signal and tracks itself. Together they cover both "active
    // on mount" and "became active later" in one path.
    use_effect(use_reactive!(|is_active| {
        if scroll_into_view && is_active && element.is_mounted() {
            // Not the DOM's `scrollIntoView`: in Chromium it moves the Tab
            // starting point onto this link, so a fresh page's first Tab
            // skipped everything above the nav. Instant under reduced motion:
            // an explicit smooth scroll overrides the stylesheet.
            let _ = element.scroll_into_view(!prefers_reduced_motion());
        }
    }));

    // One hook for every path, above the branch, and no `InternalAnchor`
    // scope: that would resolve the styling a second time.
    let style_attributes = use_style_attributes(
        &props.class,
        Some(&NAV_LINK_BASE_SX),
        &props.sx,
        &states,
        &Input::None,
        Some(style),
        true,
    );

    // An `<a>` without `href` is `generic`, so the role comes back by hand.
    if disabled {
        return box_style(style_attributes)
            .attr_default("role", "link")
            .attr("aria-disabled", "true")
            .attr("tabindex", "-1")
            .attr("aria-current", aria_current)
            .render(HtmlTag::A, props.attributes, props.children);
    }

    let mut attributes = props.attributes;
    if let Some(aria_current) = aria_current {
        attributes.push(attr("aria-current", aria_current));
    }

    render_anchor(
        style_attributes,
        props.to,
        props.target,
        Some(element.mount()),
        attributes,
        props.children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fragment_on_the_route_still_matches_the_page() {
        assert!(route_matches("/form", "/form"));
        assert!(route_matches("/form#summary", "/form"));
        assert!(!route_matches("/forms#summary", "/form"));
    }

    #[test]
    fn a_fragment_in_to_must_match_exactly() {
        assert!(route_matches("/form#summary", "/form#summary"));
        assert!(!route_matches("/form#other", "/form#summary"));
        assert!(!route_matches("/form", "/form#summary"));
    }

    #[test]
    fn the_active_background_is_the_lightest_shade_of_the_color() {
        let color = ThemeAwareValue::Color(Color::Info);
        let variables = nav_link_variables(Some(&color), Color::Primary);

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                NAV_LINK_ACTIVE_BACKGROUND_VAR.name(),
                ColorValue::Shade(Color::Info, ColorShade::S1).value()
            )
        );
    }

    /// Only the color family matters; the active background is always `S1`.
    #[test]
    fn a_shaded_color_contributes_only_its_family() {
        let color = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Info, ColorShade::S9));
        let variables = nav_link_variables(Some(&color), Color::Primary);

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                NAV_LINK_ACTIVE_BACKGROUND_VAR.name(),
                ColorValue::Shade(Color::Info, ColorShade::S1).value()
            )
        );
    }

    #[test]
    fn an_unrecognisable_color_falls_back_to_the_themed_default() {
        let raw = ThemeAwareValue::String("gold".to_string());
        let variables = nav_link_variables(Some(&raw), Color::Primary);

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                NAV_LINK_ACTIVE_BACKGROUND_VAR.name(),
                ColorValue::Shade(Color::Primary, ColorShade::S1).value()
            )
        );
    }
}
