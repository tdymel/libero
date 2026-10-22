use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::nav_link::{nav_link_color, route_matches};
use crate::{
    CssLayer,
    components::{
        accessibility::visually_hidden_sx,
        common::{
            HtmlTag, Input, Part, States, Variables, attr, base_props, disabled_look_sx,
            forced_on_sx, input_from_str, inset_focus_ring_sx, is_empty_element, names_itself,
            on_tint_color, parts_enum, safe_area_padding, sx_source, use_name_warning,
            use_style_attributes, variables,
        },
        form::use_fieldset_disabled,
        layout::{Float, box_style, render_anchor, use_box},
    },
    css::Stylesheet,
    hooks::{use_css, use_theme},
    platform::{SCROLL_PADDING_VARS, clamps_lines},
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        Color, ColorShade, ColorValue, CssVar, PAPER_BACKGROUND, Size, TEXT_FONT_SIZE,
        Z_INDEX_HEADER,
    },
    utils::warn,
};

str_enum! {
    /// How a [`NavigationBar`] stays on screen.
    pub enum NavigationBarPosition {
        /// In the flow, where it is rendered.
        #[default]
        Static = "static",
        /// Held at the bottom of its scroller while its parent is in view.
        Sticky = "sticky",
        /// Docked to the viewport's bottom edge; pad the page by
        /// `var(--lsx-navigation-bar-height)`.
        Fixed = "fixed",
    }
}

input_from_str!(NavigationBarPosition);

str_enum! {
    /// The shape of a [`NavigationBar`].
    pub enum NavigationBarVariant {
        /// A row along the bottom edge, for a phone.
        #[default]
        Bar = "bar",
        /// A full-height column on the inline-start edge, for a tablet or a desktop.
        /// Sticky, or docked to the viewport with `position: "fixed"`; pad the page by
        /// `var(--lsx-navigation-rail-width)`.
        Rail = "rail",
    }
}

input_from_str!(NavigationBarVariant);

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
const NAVIGATION_BAR_HEIGHT_VAR: CssVar = CssVar::new("--lsx-navigation-bar-height");
/// The rail's width on `:root` while one is mounted.
const NAVIGATION_RAIL_WIDTH_VAR: CssVar = CssVar::new("--lsx-navigation-rail-width");
/// The rail's width, its border included (M3).
const RAIL_WIDTH: &str = "88px";

const PILL_BACKGROUND_VAR: CssVar = CssVar::new("--lsx-navigation-bar-pill");
const PILL_COLOR_VAR: CssVar = CssVar::new("--lsx-navigation-bar-pill-color");

/// An item's least height, past the 48px touch target.
const ITEM_HEIGHT: &str = "56px";
/// Above this many items the labels get too narrow to read (M3, iOS).
const MAX_ITEMS: usize = 5;

parts_enum! {
    /// [`NavigationBar`]'s inner parts, for its `parts` prop.
    pub enum NavigationBarPart {
        /// Every [`NavigationBarItem`].
        Item = "item" => "& [data-slot='item']",
        /// The pill holding an item's icon and badge.
        Icon = "icon" => "& [data-slot='item'] > [data-slot='icon']",
        Label = "label" => "& [data-slot='item'] > [data-slot='label']",
    }
}

static NAVIGATION_BAR_SX: StaticSx = StaticSx::new(|| {
    let label = NavigationBarPart::Label.selector();
    sx().display("flex")
        .align_items("stretch")
        .width("100%")
        .min_width("0")
        .background(PAPER_BACKGROUND.value())
        .border_top("1px solid")
        .border_top_color("muted.4")
        .z_index(Z_INDEX_HEADER.overridable())
        .with("padding-block-end", safe_area_padding("0px", "bottom"))
        // Physical, as `env()` is: clear of a landscape phone's notch in either direction.
        .with("padding-left", safe_area_padding("0px", "left"))
        .with("padding-right", safe_area_padding("0px", "right"))
        .when(
            NavigationBarPosition::Sticky.state_name(),
            sx().position("sticky").with("inset-block-end", "0"),
        )
        .when(
            NavigationBarPosition::Fixed.state_name(),
            sx().position("fixed")
                .with("inset-inline", "0")
                .with("inset-block-end", "0"),
        )
        .when(NavigationBarVariant::Rail.state_name(), rail_sx())
        .when(
            "rail-fixed",
            sx().position("fixed")
                .with("inset-block", "0")
                .with("inset-inline-start", "0")
                .height("auto"),
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

/// A sticky column on the inline-start edge, padded clear of the top, bottom and that
/// edge's safe areas; a 200% label wraps and the column scrolls, never widens.
fn rail_sx() -> Sx {
    let side = |left: &str, right: &str| {
        sx().width(safe_area_padding(RAIL_WIDTH, left))
            .with(format!("padding-{left}"), safe_area_padding("0px", left))
            .with(format!("padding-{right}"), "0")
    };
    // A clamped label's ellipsis hangs past its last line: room inside the box, or it is cut.
    let label = match clamps_lines() {
        true => sx()
            .box_sizing("border-box")
            .with("padding-inline", "0.35em"),
        false => sx(),
    };
    sx().position("sticky")
        .with("inset-block-start", "0")
        .align_self("flex-start")
        .flex_direction("column")
        .flex_shrink("0")
        .gap("4px")
        .height("100dvh")
        .overflow_y("auto")
        .border_top("0")
        .with(
            "border-inline-end",
            format!(
                "1px solid {}",
                ColorValue::Shade(Color::Muted, ColorShade::S4).value()
            ),
        )
        .with("padding-block-start", safe_area_padding("12px", "top"))
        .with("padding-block-end", safe_area_padding("12px", "bottom"))
        .and(side("left", "right"))
        .rtl(side("right", "left"))
        .selector(NavigationBarPart::Item.selector(), sx().flex("0 0 auto"))
        .selector(NavigationBarPart::Label.selector(), label)
}

static NAVIGATION_BAR_ITEM_SX: StaticSx = StaticSx::new(|| {
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
        .letter_spacing("inherit")
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
        // Blitz has no line clamp: a two-line height cap cuts it without the ellipsis.
        .selector("& > [data-slot='label']", {
            let label = sx()
                .max_width("100%")
                .font_size(TEXT_FONT_SIZE.value(Size::Xs))
                .line_height("1.25")
                .text_align("center")
                .with("overflow-wrap", "anywhere")
                .overflow("hidden");
            match clamps_lines() {
                true => label
                    .display("-webkit-box")
                    .with("-webkit-box-orient", "vertical")
                    .with("-webkit-line-clamp", "2"),
                false => label.display("block").max_height("2.5em"),
            }
        })
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
/// Until measured, a bar of one-line labels: the item's content, plus the top border.
/// A pane of its own that a sticky bar closes sets that padding itself, and
/// `--lsx-scroll-padding-bottom` beside it for Blitz.
fn publish_css(measured: Option<f64>) -> String {
    let height = match measured {
        Some(height) => format!("{height}px"),
        None => safe_area_padding("64px", "bottom"),
    };
    format!(
        ":root{{{}:{height};scroll-padding-bottom:{padding};{}:{padding};}}",
        NAVIGATION_BAR_HEIGHT_VAR.name(),
        SCROLL_PADDING_VARS[2],
        padding = NAVIGATION_BAR_HEIGHT_VAR.value()
    )
}

/// The rail's width, for a page beside a fixed rail to pad by.
fn publish_rail_css(measured: Option<f64>) -> String {
    let width = match measured {
        Some(width) => format!("{width}px"),
        None => RAIL_WIDTH.to_string(),
    };
    format!(":root{{{}:{width};}}", NAVIGATION_RAIL_WIDTH_VAR.name())
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
                "NavigationBar: more than {MAX_ITEMS} items. Their labels get too \
                 narrow to read; move the rest into a menu or a drawer."
            ));
        }
    }
}

base_props! {
    parts(NavigationBarPart);
    pub struct NavigationBarProps {
        /// `static` (default), `sticky` or `fixed`; both of the latter publish
        /// `--lsx-navigation-bar-height` and pad focus clear of the bar.
        #[props(default, into)]
        position: Input<NavigationBarPosition>,
        /// `bar` (default) along the bottom edge, or `rail`, a column on the inline-start
        /// edge that publishes `--lsx-navigation-rail-width`. Switch with `use_media_query`.
        #[props(default, into)]
        variant: Input<NavigationBarVariant>,
        /// `always` (default), `selected` or `never`.
        #[props(default, into)]
        show_labels: Input<LabelVisibility>,
        /// The selected item's pill. Only the color family counts: the pill is its lightest shade.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Three to five [`NavigationBarItem`]s.
        children: Element,
    }
}

/// A phone's bar of three to five top-level destinations, a `<nav>` landmark.
/// Each item is its own Tab stop. Name it with an `aria-label`. It pads the
/// bottom, left and right safe-area insets, for a `viewport-fit=cover` page.
/// From a tablet up, `variant: "rail"` turns it into a column on the inline-start edge.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{NavigationBar, NavigationBarItem};
/// # fn app() -> Element {
/// rsx! {
///     NavigationBar { "aria-label": "Main", position: "fixed",
///         NavigationBarItem { to: "/", icon: rsx! { "⌂" }, "Home" }
///         NavigationBarItem { to: "/search", icon: rsx! { "⌕" }, "Search" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/navigation-bar>
#[component]
pub fn NavigationBar(props: NavigationBarProps) -> Element {
    let theme = use_theme();
    let position = props.position.copied_or_default();
    let labels = props.show_labels.copied_or_default();

    let mut attributes = props.attributes;
    let named = names_itself(&attributes);
    use_name_warning(
        named,
        "NavigationBar: no `aria-label` or `aria-labelledby`, so it is announced as just \"navigation\".",
    );
    if !named {
        attributes.retain(|attribute| !matches!(attribute.name, "aria-label" | "aria-labelledby"));
    }
    // Fixed per build, so the hook order holds.
    if cfg!(debug_assertions) {
        use_context_provider(ItemCount::default);
    }

    let rail = props.variant.copied_or_default() == NavigationBarVariant::Rail;
    let publishes = rail || position != NavigationBarPosition::Static;
    // Measured, so a two-line label or larger text still clears the page.
    // Keyed by the variant: a switch must not publish the bar's height as the rail's width.
    let mut measured = use_signal(|| None::<(bool, f64)>);
    let onresize = publishes.then_some(move |event: Event<ResizeData>| {
        if let Ok(size) = event.get_border_box_size() {
            let extent = Some((rail, if rail { size.width } else { size.height }));
            if *measured.peek() != extent {
                measured.set(extent);
            }
        }
    });
    use_css(
        publishes.then(|| {
            let extent = measured().filter(|(of_rail, _)| *of_rail == rail);
            let extent = extent.map(|(_, extent)| extent);
            let css = match rail {
                true => publish_rail_css(extent),
                false => publish_css(extent),
            };
            Stylesheet::from(css.as_str())
        }),
        CssLayer::Framework,
    );

    let variables: Input<Variables> =
        pill_variables(props.color.as_ref(), theme.navigation_bar.color)
            .with(Z_INDEX_HEADER.override_var(), props.z_index.resolve(None))
            .into();
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(position.state_name(), !rail)
        .with(NavigationBarVariant::Rail.state_name(), rail)
        .with(
            "rail-fixed",
            rail && position == NavigationBarPosition::Fixed,
        )
        .with("labels-never", labels == LabelVisibility::Never)
        .with("labels-selected", labels == LabelVisibility::Selected)
        .into();

    use_box()
        .framework_sx(&NAVIGATION_BAR_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&variables)
        .prepare()
        .event("onresize", onresize)
        .render(HtmlTag::Nav, attributes, props.children)
}

base_props! {
    pub struct NavigationBarItemProps {
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

/// One destination of a [`NavigationBar`]: an icon over a label, with
/// `aria-current="page"` while selected.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{NavigationBar, NavigationBarItem, Indicator};
/// # fn app() -> Element {
/// rsx! {
///     NavigationBar { "aria-label": "Main",
///         NavigationBarItem { to: "/inbox", "aria-label": "Inbox, 3 unread",
///             icon: rsx! { "✉" },
///             badge: rsx! { Indicator { label: 3u32 } },
///             "Inbox"
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/navigation-bar>
#[component]
pub fn NavigationBarItem(props: NavigationBarItemProps) -> Element {
    if cfg!(debug_assertions) {
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
                    "NavigationBarItem: both `to` and `onclick`; `to` wins and `onclick` is not called.",
                );
            }
        });
        // The icon span is aria-hidden, so the badge is only heard through the item's name.
        use_name_warning(
            props.badge.is_none() || names_itself(&props.attributes),
            "NavigationBarItem: `badge` without an `aria-label` or `aria-labelledby` carrying the count, so it is not announced.",
        );
        // The icon is aria-hidden, so an item without a label or a spread name is announced as just "link" or "button".
        use_name_warning(
            !is_empty_element(&props.children) || names_itself(&props.attributes),
            "NavigationBarItem: no label and no `aria-label` or `aria-labelledby`, so it is announced as just \"link\" or \"button\".",
        );
    }

    // A disabled `Fieldset` reaches the button form only, as a native one (todo 3873).
    let in_disabled_group = use_fieldset_disabled() && props.to.as_ref().is_none();
    let disabled = props.disabled.unwrap_or(false) || in_disabled_group;
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
        Some(&NAVIGATION_BAR_ITEM_SX),
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
    // An explicit `selected: false` overrides the router `Link`'s own `aria-current` on its route.
    let link_current = aria_current.or_else(|| props.selected.map(|_| "false"));
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
            if let Some(link_current) = link_current {
                attributes.push(attr("aria-current", link_current));
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
            // Soft-disabled as `Button`: a native `disabled` would drop its focus (todo 3838).
            box_style(style_attributes)
                .attr("type", "button")
                .attr("aria-disabled", disabled.then_some("true"))
                .attr("tabindex", disabled.then_some("-1"))
                .attr("aria-current", aria_current)
                .event("onclick", move |event: MouseEvent| {
                    if disabled {
                        event.prevent_default();
                    } else if let Some(onclick) = &onclick {
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
            crate::components::common::part_table::<NavigationBarPart>(),
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
            publish_css(None),
            ":root{--lsx-navigation-bar-height:calc(64px + env(safe-area-inset-bottom, 0px));\
             scroll-padding-bottom:var(--lsx-navigation-bar-height);\
             --lsx-scroll-padding-bottom:var(--lsx-navigation-bar-height);}"
        );
        assert_eq!(
            publish_css(Some(79.5)),
            ":root{--lsx-navigation-bar-height:79.5px;\
             scroll-padding-bottom:var(--lsx-navigation-bar-height);\
             --lsx-scroll-padding-bottom:var(--lsx-navigation-bar-height);}"
        );
    }

    /// Todo 1942: the browser harness cannot emulate a notch, so the rule itself.
    #[test]
    fn the_bar_pads_the_side_safe_areas() {
        let css = Stylesheet::from(&NAVIGATION_BAR_SX);
        for side in ["left", "right"] {
            assert!(
                css.as_str().contains(&format!(
                    "padding-{side}:calc(0px + env(safe-area-inset-{side}, 0px))"
                )),
                "{}",
                css.as_str()
            );
        }
    }

    #[test]
    fn a_rail_publishes_its_width_and_not_the_bar_height() {
        assert_eq!(
            publish_rail_css(None),
            ":root{--lsx-navigation-rail-width:88px;}"
        );
        assert_eq!(
            publish_rail_css(Some(96.0)),
            ":root{--lsx-navigation-rail-width:96px;}"
        );
    }

    /// Todo 3890: the rail pads the top, bottom and inline-start safe areas, mirrored in RTL.
    #[test]
    fn the_rail_pads_its_own_safe_areas() {
        let css = Stylesheet::from(&NAVIGATION_BAR_SX);
        let css = css.as_str();
        for padding in [
            "padding-block-start:calc(12px + env(safe-area-inset-top, 0px))",
            "padding-block-end:calc(12px + env(safe-area-inset-bottom, 0px))",
            "padding-left:calc(0px + env(safe-area-inset-left, 0px))",
            "padding-right:calc(0px + env(safe-area-inset-right, 0px))",
            "width:calc(88px + env(safe-area-inset-right, 0px))",
        ] {
            assert!(css.contains(padding), "{padding}: {css}");
        }
        assert!(css.contains(":dir(rtl)"), "{css}");
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
