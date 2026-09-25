use dioxus::{dioxus_core::AttributeValue, prelude::*};
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        common::{
            Glyph, HtmlTag, Input, Part, States, StyleAttributes, Variables, attr, base_props,
            disabled_look_sx, forced_on_sx, inset_focus_ring_sx, on_start_bar_sx, on_tint_color,
            parts_enum, parts_source, sx_source, use_style_attributes, variables, with_parts,
        },
        layout::{Collapse, box_style, use_box},
        navigation::{NewTabHint, wants_new_tab_hint},
    },
    context::IconSlot,
    hooks::{
        ElementHandle, use_cache, use_element, use_id, use_localization, use_root_id, use_theme,
    },
    platform::{ElementApi, SCROLL_MARGIN_VAR, prefers_reduced_motion},
    sx::{REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, TEXT_FONT_SIZE},
};

use crate::components::layout::render_anchor;

// A literal CSS color has no lighter shade, so it falls back to the theme's.
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
const NAV_LINK_ACTIVE_BAR_VAR: CssVar = CssVar::new("--lsx-nav-link-active-bar");

fn nav_link_variables(color: Option<&ThemeAwareValue>, default_color: Color) -> Variables {
    let base = nav_link_color(color).unwrap_or(default_color);
    let background = ThemeAwareValue::ColorValue(ColorValue::Shade(base, ColorShade::S1));
    // The label colour made to read on the tints: 4.5:1, well past 1.4.11's 3:1.
    let bar = on_tint_color(&ThemeAwareValue::ColorValue(ColorValue::Shade(
        base,
        ColorShade::S6,
    )));

    variables()
        .with(NAV_LINK_ACTIVE_BACKGROUND_VAR, background.resolve(None))
        .with(NAV_LINK_ACTIVE_BAR_VAR, bar)
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
        // Keeps `scroll_into_view`'s `Nearest` off the container's edge; Blitz reads the var.
        .scroll_margin("8rem")
        .with(SCROLL_MARGIN_VAR, "8rem")
        // Inset: a full-width link in a scrolling sidebar would clip an outset ring.
        .focus_visible(inset_focus_ring_sx("-2px"))
        // Neutral grey, so hover doesn't preview the active tint. Skips a disabled
        // link (todo 596); `:where` keeps `:hover`'s specificity.
        .selector(
            "&:hover:not(:where([data-state~=\"disabled\"]))",
            sx().background("muted.2"),
        )
        // The tint is too faint to mark the state alone, so a coloured start bar
        // does; a full ring would read as the focus ring. After each `background`, which resets it.
        .when(
            "active",
            sx().background(NAV_LINK_ACTIVE_BACKGROUND_VAR.value())
                .and(on_start_bar_sx("0px", &NAV_LINK_ACTIVE_BAR_VAR.value()))
                .hover(
                    sx().background(NAV_LINK_ACTIVE_BACKGROUND_VAR.value())
                        .and(on_start_bar_sx("0px", &NAV_LINK_ACTIVE_BAR_VAR.value())),
                )
                // Clear of the focus ring's 2px stripe.
                .focus_visible(on_start_bar_sx("2px", &NAV_LINK_ACTIVE_BAR_VAR.value()))
                .and(forced_on_sx()),
        )
        // Without `href` it has nothing to follow.
        .when("disabled", disabled_look_sx("not-allowed"))
        .selector(
            NavLinkPart::Body.selector(),
            sx().display("flex").flex_direction("column").min_width("0"),
        )
        .selector(
            NavLinkPart::Description.selector(),
            sx().font_size(TEXT_FONT_SIZE.value(Size::Xs))
                .color("text-dimmed"),
        )
});

/// The link and its toggle in one row, the nested links indented under it.
static NAV_GROUP_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .selector(
            "& > [data-slot='row']",
            sx().display("flex").align_items("stretch"),
        )
        .selector(
            "& > [data-slot='row'] > button",
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
            "& > [data-slot='row'] > button[aria-expanded=\"true\"] > svg",
            sx().transform("rotate(180deg)"),
        )
        .selector(
            "& [data-slot='children']",
            sx().padding_left("lg")
                .rtl(sx().padding_left("0").padding_right("lg")),
        )
});

parts_enum! {
    /// [`NavLink`]'s inner parts, for its `parts` prop. All but `NewTab` only with a
    /// `description`. The toggle and the nested links sit beside the link, out of reach.
    pub enum NavLinkPart {
        /// The column holding the label and the description.
        Body = "body" => "& > [data-slot='body']",
        /// The `children`.
        Label = "label" => "& > [data-slot='body'] > [data-slot='label']",
        Description = "description" => "& > [data-slot='body'] > [data-slot='description']",
        /// The new-tab icon after the label, with `target: "_blank"` only.
        NewTab = "new-tab" => "& > [data-slot='new-tab']",
    }
}

base_props! {
    parts(NavLinkPart);
    pub struct NavLinkProps {
        /// A plain path/URL or a typed route, same as `Anchor::to`.
        #[props(into)]
        to: NavigationTarget,
        #[props(default)]
        target: Option<String>,
        /// With `target: "_blank"`, an icon and a hidden "(opens in a new tab)".
        #[props(default = true)]
        new_tab_hint: bool,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Marks the current page. Unset, `to` is compared against the router's route.
        #[props(default)]
        active: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// Scrolls the link into view when it becomes active; suits a sidebar.
        #[props(default)]
        scroll_into_view: Option<bool>,
        /// A dimmed line under the label, read as the link's description.
        #[props(default, into)]
        description: Option<String>,
        /// Child `NavLink`s, shown by a toggle beside this one.
        #[props(default)]
        nested: Option<Element>,
        /// Whether `nested` shows; controlled when set, paired with `onchange`.
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

/// A sidebar or nav bar link that marks the current page, optionally with
/// nested links behind a toggle.
///
/// ```
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
///
/// Docs: <https://libero-ui.dev/navigation/nav-link>
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

    // Covers both active on mount and active later: `element` tracks itself.
    use_effect(use_reactive!(|is_active| {
        if scroll_into_view && is_active && element.is_mounted() {
            // Not the DOM's `scrollIntoView`: Chromium moves the Tab start onto
            // the link. Instant under reduced motion, which the stylesheet can't force.
            let _ = element.scroll_into_view(!prefers_reduced_motion());
        }
    }));

    // Above the branch, and no `InternalAnchor` scope, which would resolve it twice.
    let mut merged = None;
    let style_attributes = use_style_attributes(
        &props.class,
        Some(&NAV_LINK_BASE_SX),
        with_parts(
            parts_source(&props.parts),
            sx_source(&props.sx),
            &mut merged,
        ),
        &states,
        &Input::None,
        Some(style),
        true,
    );

    let mut attributes = props.attributes;
    // Out of the link's name; it arrives through `aria-describedby`.
    let body = match &props.description {
        Some(description) => {
            join_described_by(&mut attributes, &description_id());
            rsx! {
                span { "data-slot": NavLinkPart::Body.slot(),
                    span { "data-slot": NavLinkPart::Label.slot(), {props.children} }
                    span {
                        id: description_id(),
                        "data-slot": NavLinkPart::Description.slot(),
                        "aria-hidden": "true",
                        "{description}"
                    }
                }
            }
        }
        None => props.children,
    };
    // A disabled link opens nothing.
    let hint = !disabled && wants_new_tab_hint(props.target.as_deref(), props.new_tab_hint);
    let body = rsx! {
        {body}
        if hint {
            NewTabHint {}
        }
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
            div { "data-slot": "row",
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
                    Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
                }
            }
            Collapse { id: panel_id(), open: opened,
                div { "data-slot": "children", {nested} }
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

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            crate::components::common::part_table::<NavLinkPart>(),
            [
                ("body", "& > [data-slot='body']"),
                ("label", "& > [data-slot='body'] > [data-slot='label']"),
                (
                    "description",
                    "& > [data-slot='body'] > [data-slot='description']"
                ),
                ("new-tab", "& > [data-slot='new-tab']"),
            ]
        );
    }

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

    /// The `S1` tint and, as the bar, the colour's label on that tint.
    fn expected(color: Color) -> String {
        let on_tint = ColorValue::Shade(color, ColorShade::S6);
        format!(
            "{}:{};{}:var({}, {});",
            NAV_LINK_ACTIVE_BACKGROUND_VAR.name(),
            ColorValue::Shade(color, ColorShade::S1).value(),
            NAV_LINK_ACTIVE_BAR_VAR.name(),
            on_tint.on_tint_name().unwrap(),
            on_tint.as_text().value()
        )
    }

    #[test]
    fn the_active_background_is_the_lightest_shade_of_the_color() {
        let color = ThemeAwareValue::Color(Color::Info);
        let variables = nav_link_variables(Some(&color), Color::Primary);

        assert_eq!(variables.to_string(), expected(Color::Info));
    }

    /// Only the color family matters; the active background is always `S1`.
    #[test]
    fn a_shaded_color_contributes_only_its_family() {
        let color = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Info, ColorShade::S9));
        let variables = nav_link_variables(Some(&color), Color::Primary);

        assert_eq!(variables.to_string(), expected(Color::Info));
    }

    #[test]
    fn an_unrecognisable_color_falls_back_to_the_themed_default() {
        let raw = ThemeAwareValue::String("gold".to_string());
        let variables = nav_link_variables(Some(&raw), Color::Primary);

        assert_eq!(variables.to_string(), expected(Color::Primary));
    }
}
