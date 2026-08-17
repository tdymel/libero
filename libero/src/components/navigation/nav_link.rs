use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, Variables, common::base_props, variables},
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, Size, SizeCss},
};

use super::InternalAnchor;

// Only meaningful for values grounded in the named palette - an arbitrary
// literal (hex/css) has no well-defined "lighter shade" to derive, so it
// falls through to the theme's default color instead.
fn nav_link_color(value: Option<&ThemeAwareValue>) -> Option<Color> {
    match value {
        Some(ThemeAwareValue::Color(color)) => Some(*color),
        Some(ThemeAwareValue::ColorValue(
            ColorValue::Shade(color, _) | ColorValue::Contrast(color, _),
        )) => Some(*color),
        _ => None,
    }
}

const NAV_LINK_ACTIVE_BACKGROUND_VAR: &str = "--lsx-nav-link-active-background";

fn nav_link_variables(color: Option<&ThemeAwareValue>, default_color: Color) -> Variables {
    let base = nav_link_color(color).unwrap_or(default_color);
    let background = ThemeAwareValue::ColorValue(ColorValue::Shade(base, ColorShade::S1));

    variables().with(NAV_LINK_ACTIVE_BACKGROUND_VAR, background.resolved())
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
        // Gives `scroll_into_view`'s `Nearest` some breathing room instead of
        // stopping flush against the scroll container's edge - respected by
        // the native `scrollIntoView` call under the hood, so it only ever
        // affects *where* a scroll lands, never whether one happens at all.
        .scroll_margin("8rem")
        // A non-active link only tints on hover, and with a neutral grey
        // rather than `color` - hovering shouldn't preview the "selected"
        // look before it's actually selected. Active gets a light tint of
        // the resolved color instead, same convention as `Mark`/
        // `ActionIcon`'s outlined hover.
        .hover(sx().background("grey.1"))
        .when(
            "active",
            sx().background(format!("var({NAV_LINK_ACTIVE_BACKGROUND_VAR})"))
                .hover(sx().background(format!("var({NAV_LINK_ACTIVE_BACKGROUND_VAR})"))),
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
        /// Highlights this link with a light background tint. Unset auto-detects
        /// by comparing `to` against the current route - only ever true for an
        /// `Internal` target with a router mounted. Set explicitly to override
        /// (e.g. a parent nav item that should read as active for a whole
        /// section, or for an `External` target/routerless usage, where
        /// auto-detection has nothing to compare against and always reads
        /// `false`).
        #[props(default)]
        active: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// Scrolls this link into view (only if it isn't already visible)
        /// whenever it becomes active - on mount, or later if a different link
        /// was active first. Off by default: it's a side effect on whatever
        /// scrollable container happens to be an ancestor, which only makes
        /// sense for a handful of call sites (e.g. a sidebar), not every place a
        /// `NavLink` might get used.
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
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("disabled", disabled)
        .with("active", is_active);
    let aria_current = is_active.then_some("page");

    let scroll_into_view = props.scroll_into_view.unwrap_or(false);
    let mut mounted = use_signal(|| None::<MountedEvent>);

    // Re-runs whenever `is_active` changes (`use_reactive!` - it's a plain
    // bool, not a signal) and whenever `mounted` first becomes available
    // (a real signal read, auto-tracked) - between the two, this covers
    // both "already active on mount" and "became active later" without
    // needing two separate code paths.
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
