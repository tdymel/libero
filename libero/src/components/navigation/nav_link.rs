use dioxus::{dioxus_core::AttributeValue, prelude::*};

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{
            ChevronDownIcon, StyleAttributes, attr, base_props, disabled_look_sx, forced_on_sx,
            inset_focus_ring_sx, on_start_bar_sx, use_style_attributes,
        },
        layout::{Collapse, box_style, use_box},
        variables,
    },
    hooks::{
        ElementHandle, use_cache, use_element, use_id, use_localization, use_root_id, use_theme,
    },
    platform::{ElementApi, prefers_reduced_motion},
    sx::{REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, TEXT_FONT_SIZE},
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
        // Inset: a full-width link in a scrolling sidebar would clip an outset ring.
        .focus_visible(inset_focus_ring_sx("-2px"))
        // Hover is neutral grey, not `color`: it shouldn't preview the
        // selected look. Active gets the light color tint instead.
        // Skips a disabled link, which needs no `pointer-events: none` then and
        // shows `not-allowed` (todo 596). `:where` keeps `:hover`'s specificity.
        .selector(
            "&:hover:not(:where([data-state~=\"disabled\"]))",
            sx().background("muted.2"),
        )
        // The house on-state line at the start edge only: a full ring would
        // read as this link's inset focus ring. After each `background`, which resets it.
        .when(
            "active",
            sx().background(NAV_LINK_ACTIVE_BACKGROUND_VAR.value())
                .and(on_start_bar_sx("0px"))
                .hover(
                    sx().background(NAV_LINK_ACTIVE_BACKGROUND_VAR.value())
                        .and(on_start_bar_sx("0px")),
                )
                // Clear of the focus ring's 2px stripe.
                .focus_visible(on_start_bar_sx("2px"))
                .and(forced_on_sx()),
        )
        // Without `href` it has nothing to follow.
        .when("disabled", disabled_look_sx("not-allowed"))
        .selector(
            "& > [data-nav-body]",
            sx().display("flex").flex_direction("column").min_width("0"),
        )
        .selector(
            "& [data-nav-description]",
            sx().font_size(TEXT_FONT_SIZE.value(Size::Xs))
                .color("text-dimmed"),
        )
});

/// The link and its toggle in one row, the nested links indented under it.
static NAV_GROUP_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .selector(
            "& > [data-nav-row]",
            sx().display("flex").align_items("stretch"),
        )
        .selector(
            "& > [data-nav-row] > button",
            sx().display("inline-flex")
                .align_items("center")
                .justify_content("center")
                .flex_shrink("0")
                .min_width("32px")
                .min_height("32px")
                .padding("0")
                .border("0")
                .border_radius(SizeCss::RADIUS.value(Size::Sm))
                .background("transparent")
                .color("inherit")
                .cursor("pointer")
                .focus_visible(inset_focus_ring_sx("-2px"))
                .hover(sx().background("muted.2"))
                .selector("&:disabled", disabled_look_sx("not-allowed"))
                .selector(
                    "& > svg",
                    sx().width("16px")
                        .height("16px")
                        .transition("transform 150ms ease")
                        .media(REDUCED_MOTION, sx().transition("none")),
                ),
        )
        .selector(
            "& > [data-nav-row] > button[aria-expanded=\"true\"] > svg",
            sx().transform("rotate(180deg)"),
        )
        .selector(
            "& [data-nav-children]",
            sx().padding_left("lg")
                .selector("&:dir(rtl)", sx().padding_left("0").padding_right("lg")),
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
        /// A dimmed line under the label, read as the link's description.
        #[props(default, into)]
        description: Option<String>,
        /// Child `NavLink`s, shown under this one by a toggle button beside
        /// it. The link itself still goes to `to`.
        #[props(default)]
        nested: Option<Element>,
        /// Whether `nested` shows. Set, it is controlled: pair it with
        /// `onchange`.
        #[props(default)]
        opened: Option<bool>,
        /// Whether `nested` shows at first, when `opened` is unset.
        #[props(default)]
        default_opened: bool,
        /// The toggle asks for this `opened`.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        children: Element,
    }
}

/// A navigation list item - `Anchor` plus a themed active/hover background
/// and `aria-current`, for a sidebar or nav bar link. With `nested` it is a
/// disclosure too: a toggle beside the link shows the child links.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::NavLink;
/// # fn app() -> Element {
/// rsx! {
///     NavLink { to: "/docs", description: "Guides and API",
///         nested: rsx! {
///             NavLink { to: "/docs/install", "Install" }
///             NavLink { to: "/docs/theming", "Theming" }
///         },
///         "Docs"
///     }
/// }
/// # }
/// ```
#[component]
pub fn NavLink(props: NavLinkProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().nav_link;
    let link_id = use_root_id(&props.attributes);
    let (toggle_id, panel_id, description_id) = (use_id(), use_id(), use_id());
    let mut own_opened = use_signal(|| props.default_opened);
    let opened = props.opened.unwrap_or(own_opened());
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

    let mut attributes = props.attributes;
    // Hidden from the link's name, which the label alone gives; the
    // description reaches it through `aria-describedby`.
    let body = match &props.description {
        Some(description) => {
            join_described_by(&mut attributes, &description_id());
            rsx! {
                span { "data-nav-body": true,
                    span { {props.children} }
                    span {
                        id: description_id(),
                        "data-nav-description": true,
                        "aria-hidden": "true",
                        "{description}"
                    }
                }
            }
        }
        None => props.children,
    };
    // A hook, so above the branch.
    let group_style = use_box().framework_sx(&NAV_GROUP_SX).prepare();
    let Some(nested) = props.nested else {
        return nav_link(
            style_attributes,
            props.to,
            props.target,
            disabled,
            aria_current,
            element,
            attributes,
            body,
        );
    };
    if !attributes.iter().any(|attribute| attribute.name == "id") {
        attributes.push(attr("id", link_id()));
    }
    let link = nav_link(
        style_attributes,
        props.to,
        props.target,
        disabled,
        aria_current,
        element,
        attributes,
        body,
    );

    let controlled = props.opened.is_some();
    let onchange = props.onchange;
    let toggle = move |_: MouseEvent| {
        if !controlled {
            own_opened.set(!opened);
        }
        if let Some(onchange) = &onchange {
            onchange.call(!opened);
        }
    };
    group_style.render(
        HtmlTag::Div,
        Vec::new(),
        rsx! {
            div { "data-nav-row": true,
                {link}
                // APG disclosure. Named "Show links" plus the link's own name.
                button {
                    r#type: "button",
                    id: toggle_id(),
                    "aria-label": labels.show_links,
                    "aria-labelledby": "{toggle_id} {link_id}",
                    "aria-expanded": opened.to_string(),
                    "aria-controls": panel_id(),
                    disabled: disabled.then_some(true),
                    onclick: toggle,
                    ChevronDownIcon {}
                }
            }
            Collapse { id: panel_id(), open: opened,
                div { "data-nav-children": true, {nested} }
            }
        },
    )
}

/// The link itself: an `<a>` to `to`, or with `disabled` one that goes nowhere.
#[allow(clippy::too_many_arguments)]
fn nav_link(
    style_attributes: StyleAttributes,
    to: NavigationTarget,
    target: Option<String>,
    disabled: bool,
    aria_current: Option<&'static str>,
    element: ElementHandle,
    mut attributes: Vec<Attribute>,
    body: Element,
) -> Element {
    // An `<a>` without `href` is `generic`, so the role comes back by hand.
    if disabled {
        return box_style(style_attributes)
            .attr_default("role", "link")
            .attr("aria-disabled", "true")
            .attr("tabindex", "-1")
            .attr("aria-current", aria_current)
            .render(HtmlTag::A, attributes, body);
    }

    if let Some(aria_current) = aria_current {
        attributes.push(attr("aria-current", aria_current));
    }

    render_anchor(
        style_attributes,
        to,
        target,
        Some(element.mount()),
        attributes,
        body,
    )
}

/// `aria-describedby` is a list: `id` joins a caller's rather than replacing it.
fn join_described_by(attributes: &mut Vec<Attribute>, id: &str) {
    let caller = attributes.iter_mut().find(|attribute| {
        attribute.name == "aria-describedby" && matches!(attribute.value, AttributeValue::Text(_))
    });
    match caller {
        Some(attribute) => {
            if let AttributeValue::Text(ids) = &attribute.value {
                attribute.value = AttributeValue::Text(format!("{ids} {id}"));
            }
        }
        None => attributes.push(attr("aria-describedby", id.to_string())),
    }
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
