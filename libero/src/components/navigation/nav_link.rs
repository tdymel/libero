use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss},
};

use super::InternalAnchor;

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
        // Keeps `scroll_into_view`'s `Nearest` off the container's edge.
        // Affects where a scroll lands, never whether one happens.
        .scroll_margin("8rem")
        // Hover is neutral grey, not `color`: it shouldn't preview the
        // selected look. Active gets the light color tint instead.
        .hover(sx().background("grey.2"))
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
            .map(|router| router.full_route_string() == *path)
            .unwrap_or(false),
        NavigationTarget::External(_) => false,
    });

    let variables = nav_link_variables(props.color.as_ref(), theme.nav_link.color);
    let states = props
        .states
        .unwrap_or_default()
        .with("disabled", disabled)
        .with("active", is_active);
    let aria_current = is_active.then_some("page");

    let scroll_into_view = props.scroll_into_view.unwrap_or(false);
    let mut mounted = use_signal(|| None::<MountedEvent>);

    // `is_active` is a plain bool, hence `use_reactive!`; `mounted` is a
    // signal and tracks itself. Together they cover both "active on mount"
    // and "became active later" in one path.
    use_effect(use_reactive!(|is_active| {
        if scroll_into_view
            && is_active
            && let Some(event) = mounted()
        {
            spawn(async move {
                let _ = event
                    .scroll_to_with_options(ScrollToOptions {
                        behavior: ScrollBehavior::Smooth,
                        vertical: ScrollLogicalPosition::Nearest,
                        horizontal: ScrollLogicalPosition::Nearest,
                    })
                    .await;
            });
        }
    }));

    if disabled {
        return rsx! {
            Box {
                component: "a",
                class: props.class,
                sx: props.sx,
                states,
                variables,
                framework_sx: &NAV_LINK_BASE_SX,
                "aria-disabled": "true",
                tabindex: "-1",
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    rsx! {
        InternalAnchor {
            to: props.to,
            target: props.target,
            class: props.class,
            sx: props.sx,
            framework_sx: &NAV_LINK_BASE_SX,
            states,
            variables,
            "aria-current": aria_current,
            onmounted: move |event| mounted.set(Some(event)),
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
