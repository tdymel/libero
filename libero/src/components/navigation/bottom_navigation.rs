use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::nav_link::{nav_link_color, route_matches};
use crate::{
    CssLayer,
    components::{
        accessibility::visually_hidden_sx,
        common::{
            HtmlTag, Input, Part, States, Variables, attr, base_props, disabled_look_sx,
            forced_on_sx, input_from_str, inset_focus_ring_sx, names_itself, on_tint_color,
            parts_enum, safe_area_padding, sx_source, use_name_warning, use_style_attributes,
            variables,
        },
        layout::{Float, box_style, render_anchor, use_box},
    },
    css::Stylesheet,
    hooks::{use_css, use_theme},
    str_enum::str_enum,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        Color, ColorShade, ColorValue, CssVar, PAPER_BACKGROUND, Size, TEXT_FONT_SIZE,
        Z_INDEX_HEADER,
    },
    utils::warn,
};

str_enum! {
    /// How a [`BottomNavigation`] stays on screen.
    pub enum BottomNavigationPosition {
        /// In the flow, where it is rendered.
        #[default]
        Static = "static",
        /// Held at the bottom of its scroller while its parent is in view.
        Sticky = "sticky",
        /// Docked to the viewport's bottom edge; pad the page by
        /// `var(--lsx-bottom-navigation-height)`.
        Fixed = "fixed",
    }
}

input_from_str!(BottomNavigationPosition);

str_enum! {
    /// Which item labels show. A hidden label still names its item.
    #[non_exhaustive]
    pub enum LabelVisibility {
        #[default]
        Always = "always",
        /// Only the selected item's.
        Selected = "selected",
        Never = "never",
    }
}

input_from_str!(LabelVisibility);

/// The bar's height on `:root` while a sticky or fixed one is mounted.
const BOTTOM_NAVIGATION_HEIGHT_VAR: CssVar = CssVar::new("--lsx-bottom-navigation-height");

const PILL_BACKGROUND_VAR: CssVar = CssVar::new("--lsx-bottom-navigation-pill");
const PILL_COLOR_VAR: CssVar = CssVar::new("--lsx-bottom-navigation-pill-color");

/// An item's least height, past the 48px touch target.
const ITEM_HEIGHT: &str = "56px";
/// Above this many items the labels get too narrow to read (M3, iOS).
const MAX_ITEMS: usize = 5;

parts_enum! {
    /// [`BottomNavigation`]'s inner parts, for its `parts` prop.
    pub enum BottomNavigationPart {
        /// Every [`BottomNavigationItem`].
        Item = "item" => "& [data-slot='item']",
        /// The pill holding an item's icon and badge.
        Icon = "icon" => "& [data-slot='item'] > [data-slot='icon']",
        Label = "label" => "& [data-slot='item'] > [data-slot='label']",
    }
}

static BOTTOM_NAVIGATION_SX: StaticSx = StaticSx::new(|| {
    let label = BottomNavigationPart::Label.selector();
    sx().display("flex")
        .align_items("stretch")
        .width("100%")
        .min_width("0")
        .background(PAPER_BACKGROUND.value())
        .border_top("1px solid")
        .border_top_color("muted.4")
        .z_index(Z_INDEX_HEADER.overridable())
        // The insets are physical, so these stay physical in RTL too.
        .padding_bottom(safe_area_padding("0px", "bottom"))
        .padding_left(safe_area_padding("0px", "left"))
        .padding_right(safe_area_padding("0px", "right"))
        .when(
            BottomNavigationPosition::Sticky.state_name(),
            sx().position("sticky").with("inset-block-end", "0"),
        )
        .when(
            BottomNavigationPosition::Fixed.state_name(),
            sx().position("fixed")
                .with("inset-inline", "0")
                .with("inset-block-end", "0"),
        )
        .when("labels-never", sx().selector(label, visually_hidden_sx()))
        .when(
            "labels-selected",
            sx().selector(
                "& [data-slot='item']:not([data-state~=\"selected\"]) > [data-slot='label']",
                visually_hidden_sx(),
            ),
        )
});

static BOTTOM_NAVIGATION_ITEM_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .flex("1 1 0")
        .min_width("0")
        .min_height(ITEM_HEIGHT)
        .display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .gap("4px")
        .padding("6px 2px")
        .margin("0")
        .border("0")
        .background("transparent")
        .color("text-dimmed")
        .font("inherit")
        .text_decoration("none")
        .cursor("pointer")
        .with("-webkit-tap-highlight-color", "transparent")
        .focus_visible(inset_focus_ring_sx("-2px"))
        .selector(
            "& > [data-slot='icon']",
            sx().position("relative")
                .display("inline-flex")
                .align_items("center")
                .justify_content("center")
                .width("56px")
                .max_width("100%")
                .height("32px")
                .flex_shrink("0")
                .border_radius("16px")
                .selector("& > svg", sx().width("24px").height("24px")),
        )
        .selector(
            "&:hover:not(:where([data-state~=\"disabled\"], [data-state~=\"selected\"])) > [data-slot='icon']",
            sx().background("muted.2"),
        )
        // Two lines at most, then an ellipsis: a long label must not grow the bar.
        .selector(
            "& > [data-slot='label']",
            sx().max_width("100%")
                .font_size(TEXT_FONT_SIZE.value(Size::Xs))
                .line_height("1.25")
                .text_align("center")
                .with("overflow-wrap", "anywhere")
                .display("-webkit-box")
                .with("-webkit-box-orient", "vertical")
                .with("-webkit-line-clamp", "2")
                .overflow("hidden"),
        )
        .when(
            "selected",
            sx().color("inherit")
                .selector("& > [data-slot='label']", sx().font_weight("600"))
                .selector(
                    "& > [data-slot='icon']",
                    sx().background(PILL_BACKGROUND_VAR.value())
                        .color(PILL_COLOR_VAR.value())
                        .and(forced_on_sx()),
                ),
        )
        .when("disabled", disabled_look_sx("not-allowed"))
});

/// The pill's tint and the icon colour that reads on it, as `NavLink`'s.
fn pill_variables(color: Option<&ThemeAwareValue>, default_color: Color) -> Variables {
    let base = nav_link_color(color).unwrap_or(default_color);
    let background = ThemeAwareValue::ColorValue(ColorValue::Shade(base, ColorShade::S1));
    let on_tint = on_tint_color(&ThemeAwareValue::ColorValue(ColorValue::Shade(
        base,
        ColorShade::S6,
    )));
    variables()
        .with(PILL_BACKGROUND_VAR, background.resolve(None))
        .with(PILL_COLOR_VAR, on_tint)
}

/// The bar's height and the scroll padding that keeps focus clear of it (2.4.11).
fn publish_css() -> String {
    format!(
        ":root{{{}:{};scroll-padding-bottom:{};}}",
        BOTTOM_NAVIGATION_HEIGHT_VAR.name(),
        safe_area_padding(ITEM_HEIGHT, "bottom"),
        BOTTOM_NAVIGATION_HEIGHT_VAR.value()
    )
}

/// Counts mounted items, to warn once past [`MAX_ITEMS`].
#[derive(Clone, Default)]
struct ItemCount {
    count: Rc<Cell<usize>>,
    warned: Rc<Cell<bool>>,
}

impl ItemCount {
    fn add(&self) {
        self.count.set(self.count.get() + 1);
        if self.count.get() > MAX_ITEMS && !self.warned.replace(true) {
            warn(&format!(
                "BottomNavigation: more than {MAX_ITEMS} items. Their labels get too \
                 narrow to read; move the rest into a menu or a drawer."
            ));
        }
    }
}

base_props! {
    parts(BottomNavigationPart);
    pub struct BottomNavigationProps {
        /// `static` (default), `sticky` or `fixed`; both of the latter publish
        /// `--lsx-bottom-navigation-height` and pad focus clear of the bar.
        #[props(default, into)]
        position: Input<BottomNavigationPosition>,
        /// `always` (default), `selected` or `never`.
        #[props(default, into)]
        show_labels: Input<LabelVisibility>,
        /// The selected item's pill. Only the color family counts: the pill is its lightest shade.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Three to five [`BottomNavigationItem`]s.
        children: Element,
    }
}

/// A phone's bar of three to five top-level destinations, a `<nav>` landmark.
/// Each item is its own Tab stop. Name it with an `aria-label`.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{BottomNavigation, BottomNavigationItem};
/// # fn app() -> Element {
/// rsx! {
///     BottomNavigation { "aria-label": "Main", position: "fixed",
///         BottomNavigationItem { to: "/", icon: rsx! { "⌂" }, "Home" }
///         BottomNavigationItem { to: "/search", icon: rsx! { "⌕" }, "Search" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/bottom-navigation>
#[component]
pub fn BottomNavigation(props: BottomNavigationProps) -> Element {
    let theme = use_theme();
    let position = props.position.copied_or_default();
    let labels = props.show_labels.copied_or_default();

    use_name_warning(
        names_itself(&props.attributes),
        "BottomNavigation: no `aria-label` or `aria-labelledby`, so it is announced as just \"navigation\".",
    );
    use_context_provider(ItemCount::default);

    let publishes = position != BottomNavigationPosition::Static;
    use_css(
        publishes.then(|| Stylesheet::from(publish_css().as_str())),
        CssLayer::Framework,
    );

    let variables: Input<Variables> =
        pill_variables(props.color.as_ref(), theme.bottom_navigation.color)
            .with(Z_INDEX_HEADER.override_var(), props.z_index.resolve(None))
            .into();
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(position.state_name(), true)
        .with("labels-never", labels == LabelVisibility::Never)
        .with("labels-selected", labels == LabelVisibility::Selected)
        .into();

    use_box()
        .framework_sx(&BOTTOM_NAVIGATION_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Nav, props.attributes, props.children)
}

base_props! {
    pub struct BottomNavigationItemProps {
        /// A path, a URL or a typed route: the item is a link. Wins over `onclick`.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        /// Without `to`, the item is a button that calls this.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Marks the current destination. Unset, `to` is compared against the router's route.
        #[props(default)]
        selected: Option<bool>,
        /// Decorative: the label names the item.
        icon: Element,
        /// A count or dot over the icon, such as an `Indicator`. Put the count in
        /// the item's `aria-label` too, as `"Inbox, 3 unread"`.
        #[props(default)]
        badge: Option<Element>,
        #[props(default)]
        disabled: Option<bool>,
        /// The label.
        children: Element,
    }
}

/// One destination of a [`BottomNavigation`]: an icon over a label, with
/// `aria-current="page"` while selected.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{BottomNavigation, BottomNavigationItem, Indicator};
/// # fn app() -> Element {
/// rsx! {
///     BottomNavigation { "aria-label": "Main",
///         BottomNavigationItem { to: "/inbox", "aria-label": "Inbox, 3 unread",
///             icon: rsx! { "✉" },
///             badge: rsx! { Indicator { label: 3u32 } },
///             "Inbox"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/bottom-navigation>
#[component]
pub fn BottomNavigationItem(props: BottomNavigationItemProps) -> Element {
    let count = use_hook(|| {
        let count = try_consume_context::<ItemCount>();
        if let Some(count) = &count {
            count.add();
        }
        count
    });
    use_drop(move || {
        if let Some(count) = &count {
            count.count.set(count.count.get().saturating_sub(1));
        }
    });
    use_hook(|| {
        if props.to.as_ref().is_some() && props.onclick.is_some() {
            warn(
                "BottomNavigationItem: both `to` and `onclick`; `to` wins and `onclick` is not called.",
            );
        }
    });

    let disabled = props.disabled.unwrap_or(false);
    let selected = props.selected.unwrap_or_else(|| match props.to.as_ref() {
        Some(NavigationTarget::Internal(path)) => try_router()
            .map(|router| route_matches(&router.full_route_string(), path))
            .unwrap_or(false),
        _ => false,
    });
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("selected", selected)
        .with("disabled", disabled)
        .into();
    let style_attributes = use_style_attributes(
        &props.class,
        Some(&BOTTOM_NAVIGATION_ITEM_SX),
        sx_source(&props.sx),
        &states,
        &Input::None,
        None,
        true,
    );

    let body = rsx! {
        span { "data-slot": "icon", "aria-hidden": "true",
            {props.icon}
            if let Some(badge) = props.badge {
                Float { placement: "top-end", {badge} }
            }
        }
        span { "data-slot": "label", {props.children} }
    };
    let aria_current = selected.then_some("page");
    let mut attributes = props.attributes;
    attributes.push(attr("data-slot", "item"));

    match props.to.as_ref().cloned() {
        // An `<a>` without `href` is `generic`, so the role comes back by hand.
        Some(_) if disabled => box_style(style_attributes)
            .attr_default("role", "link")
            .attr("aria-disabled", "true")
            .attr("tabindex", "-1")
            .attr("aria-current", aria_current)
            .render(HtmlTag::A, attributes, body),
        Some(to) => {
            if let Some(aria_current) = aria_current {
                attributes.push(attr("aria-current", aria_current));
            }
            render_anchor(
                style_attributes,
                to,
                None,
                None::<fn(MountedEvent)>,
                attributes,
                body,
            )
        }
        None => {
            let onclick = props.onclick;
            box_style(style_attributes)
                .attr("type", "button")
                .attr("disabled", disabled.then_some(true))
                .attr("aria-current", aria_current)
                .event("onclick", move |event: MouseEvent| {
                    if let Some(onclick) = &onclick {
                        onclick.call(event);
                    }
                })
                .render(HtmlTag::Button, attributes, body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            crate::components::common::part_table::<BottomNavigationPart>(),
            [
                ("item", "& [data-slot='item']"),
                ("icon", "& [data-slot='item'] > [data-slot='icon']"),
                ("label", "& [data-slot='item'] > [data-slot='label']"),
            ]
        );
    }

    #[test]
    fn a_docked_bar_publishes_its_height_with_the_safe_area() {
        assert_eq!(
            publish_css(),
            ":root{--lsx-bottom-navigation-height:calc(56px + env(safe-area-inset-bottom, 0px));\
             scroll-padding-bottom:var(--lsx-bottom-navigation-height);}"
        );
    }

    #[test]
    fn the_sixth_item_warns_once() {
        let count = ItemCount::default();
        crate::utils::take_warnings();
        for _ in 0..7 {
            count.add();
        }
        let warnings = crate::utils::take_warnings();
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("more than 5"));
    }
}
