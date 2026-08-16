use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{Box, Input, States, common::class_list},
    hooks::{use_css, use_theme},
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

// Active gets a light tint of the resolved color, same convention as
// `Mark`/`ActionIcon`'s outlined hover. A non-active link only tints on
// hover, and with a neutral grey rather than `color` - hovering shouldn't
// preview the "selected" look before it's actually selected.
fn nav_link_dynamic_sx(
    is_active: bool,
    color: Option<&ThemeAwareValue>,
    default_color: Color,
) -> Sx {
    if is_active {
        let base = nav_link_color(color).unwrap_or(default_color);
        sx().background(ThemeAwareValue::ColorValue(ColorValue::Shade(
            base,
            ColorShade::S1,
        )))
    } else {
        sx().hover(sx().background("grey.1"))
    }
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
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
});

#[derive(Props, Clone, PartialEq)]
pub struct NavLinkProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
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
    children: Element,
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

    let dynamic_sx = nav_link_dynamic_sx(is_active, props.color.as_ref(), theme.nav_link.color);
    let dynamic_class = use_css(&dynamic_sx, CssLayer::UserDynamic);
    let class = class_list([props.class, dynamic_class]);
    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("disabled", disabled)
        .with("active", is_active);
    let aria_current = is_active.then_some("page");

    if disabled {
        return rsx! {
            Box {
                component: "a",
                class,
                sx: props.sx,
                states,
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
            class,
            sx: props.sx,
            framework_sx: &NAV_LINK_BASE_SX,
            states,
            "aria-current": aria_current,
            attributes: props.attributes,
            {props.children}
        }
    }
}
