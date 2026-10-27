use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::nav_link::{nav_link_color, route_matches};
use crate::{
    CssLayer,
    components::{
        accessibility::visually_hidden_absolute_sx,
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
        Color, ColorShade, ColorValue, CssVar, PAPER_BACKGROUND, Responsive, Size, TEXT_FONT_SIZE,
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

/// One shape, or one per breakpoint: `variant: responsive(NavigationBarVariant::Bar).sm(NavigationBarVariant::Rail)`.
impl From<&str> for Input<Responsive<NavigationBarVariant>> {
    fn from(value: &str) -> Self {
        Self::Value(Responsive::new(value.into()))
    }
}

impl From<String> for Input<Responsive<NavigationBarVariant>> {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<NavigationBarVariant> for Input<Responsive<NavigationBarVariant>> {
    fn from(value: NavigationBarVariant) -> Self {
        Self::Value(Responsive::new(value))
    }
}

impl From<Option<NavigationBarVariant>> for Input<Responsive<NavigationBarVariant>> {
    fn from(value: Option<NavigationBarVariant>) -> Self {
        value.map_or(Self::None, Into::into)
    }
}

impl From<Responsive<NavigationBarVariant>> for Input<Responsive<NavigationBarVariant>> {
    fn from(value: Responsive<NavigationBarVariant>) -> Self {
        Self::Value(value)
    }
}

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
        .when(NavigationBarPosition::Sticky.state_name(), sticky_bar_sx())
        .when(NavigationBarPosition::Fixed.state_name(), fixed_bar_sx())
        .when(NavigationBarVariant::Rail.state_name(), rail_sx())
        .when("rail-fixed", fixed_rail_sx())
        .when(
            "labels-never",
            sx().selector(label, visually_hidden_absolute_sx()),
        )
        .when(
            "labels-selected",
            sx().selector(
                "& [data-slot='item']:not([data-state~=\"selected\"]) > [data-slot='label']",
                visually_hidden_absolute_sx(),
            ),
        )
});

/// Each run of one shape across the breakpoints, with the media query that bounds it.
fn layout_runs(variant: &Responsive<NavigationBarVariant>) -> Vec<(String, NavigationBarVariant)> {
    let mut runs = vec![(None, variant.base())];
    for (size, value) in variant.breakpoints() {
        if runs.last().is_some_and(|(_, last)| *last != value) {
            runs.push((Some(size), value));
        }
    }
    let ends: Vec<_> = runs
        .iter()
        .skip(1)
        .map(|(from, _)| *from)
        .chain([None])
        .collect();
    runs.into_iter()
        .zip(ends)
        .map(|((from, value), until)| (range_query(from, until), value))
        .collect()
}

/// From `from`'s breakpoint to just under `until`'s; `max-width` is inclusive, so 0.02px under.
fn range_query(from: Option<Size>, until: Option<Size>) -> String {
    let rem = |size: Size| {
        let value = size.breakpoint_value().trim_end_matches("rem");
        value.parse::<f64>().unwrap_or_default()
    };
    let from = from.map(|size| format!("(min-width: {})", size.breakpoint_value()));
    let until = until.map(|size| format!("(max-width: {}rem)", rem(size) - 0.00125));
    from.into_iter()
        .chain(until)
        .collect::<Vec<_>>()
        .join(" and ")
}

/// One shape with its position, as the states select it for a plain `variant`.
fn shape_sx(variant: NavigationBarVariant, position: NavigationBarPosition) -> Sx {
    match (variant, position) {
        (NavigationBarVariant::Rail, NavigationBarPosition::Fixed) => {
            rail_sx().and(fixed_rail_sx())
        }
        (NavigationBarVariant::Rail, _) => rail_sx(),
        (NavigationBarVariant::Bar, NavigationBarPosition::Sticky) => sticky_bar_sx(),
        (NavigationBarVariant::Bar, NavigationBarPosition::Fixed) => fixed_bar_sx(),
        (NavigationBarVariant::Bar, NavigationBarPosition::Static) => sx(),
    }
}

fn sticky_bar_sx() -> Sx {
    sx().position("sticky").with("inset-block-end", "0")
}

fn fixed_bar_sx() -> Sx {
    sx().position("fixed")
        .with("inset-inline", "0")
        .with("inset-block-end", "0")
}

/// `stretch`: the rail's `flex-start` would shrink a fixed box to its content in Chrome.
fn fixed_rail_sx() -> Sx {
    sx().position("fixed")
        .align_self("stretch")
        .with("inset-block", "0")
        .with("inset-inline-start", "0")
        .height("auto")
}

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
            .with("padding-inline", "calc(0.35em + 2px)"),
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
        .justify_content("flex-start")
        .gap("4px")
        .padding("6px 0")
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
        // Only with a hovering pointer: a tap on touch must not leave the pill lit.
        .media(
            "(hover: hover)",
            sx().selector(
                "&:hover:not(:where([data-state~=\"disabled\"], [data-state~=\"selected\"])) > [data-slot='icon']",
                sx().background("muted.2"),
            ),
        )
        // Two lines at most, then an ellipsis: a long label must not grow the bar.
        // Blitz has no line clamp: a two-line height cap cuts it without the ellipsis.
        .selector("& > [data-slot='label']", {
            let label = sx()
                .box_sizing("border-box")
                .max_width("100%")
                .font_size(TEXT_FONT_SIZE.value(Size::Xs))
                .line_height("1.25")
                .text_align("center")
                .with("overflow-wrap", "break-word")
                .overflow("hidden");
            match clamps_lines() {
                // Padding inside the box keeps the clamped ellipsis from hanging past it (3943).
                true => label
                    .with("padding-inline", "max(2px, 0.2em)")
                    .display("-webkit-box")
                    .with("-webkit-box-orient", "vertical")
                    .with("-webkit-line-clamp", "2"),
                false => label
                    .with("padding-inline", "2px")
                    .display("block")
                    .max_height("2.5em"),
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

/// Each run's extent under its query: the rail's width, a docked bar's height.
fn publish_runs_css(
    runs: &[(String, NavigationBarVariant)],
    docked: bool,
    bar: Option<f64>,
    rail: Option<f64>,
) -> String {
    runs.iter()
        .filter_map(|(query, variant)| {
            let css = match variant {
                NavigationBarVariant::Rail => publish_rail_css(rail),
                NavigationBarVariant::Bar if docked => publish_css(bar),
                NavigationBarVariant::Bar => return None,
            };
            Some(format!("@media {query}{{{css}}}"))
        })
        .collect()
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
        /// edge that publishes `--lsx-navigation-rail-width`. One per breakpoint, as
        /// `responsive(NavigationBarVariant::Bar).sm(NavigationBarVariant::Rail)`, switches in CSS.
        #[props(default, into)]
        variant: Input<Responsive<NavigationBarVariant>>,
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
/// From a tablet up, `variant: "rail"` turns it into a column on the inline-start edge;
/// a `variant` per breakpoint switches between the two with a media query, without a script.
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

    let variant = props
        .variant
        .as_ref()
        .copied()
        .unwrap_or_else(|| Responsive::new(NavigationBarVariant::Bar));
    let runs = layout_runs(&variant);
    // One shape throughout goes through the states; several through media queries.
    let plain = (runs.len() == 1).then_some(variant.base());
    let rail = plain == Some(NavigationBarVariant::Rail);
    let any_rail = runs
        .iter()
        .any(|(_, shape)| *shape == NavigationBarVariant::Rail);
    let docked = position != NavigationBarPosition::Static;
    let publishes = any_rail || docked;
    // Measured, so a two-line label or larger text still clears the page.
    // Keyed by the shape: a switch must not publish the bar's height as the rail's width.
    let mut measured = use_signal(|| None::<(bool, f64)>);
    let onresize = publishes.then_some(move |event: Event<ResizeData>| {
        if let Ok(size) = event.get_border_box_size() {
            // Switched by a query, the shape shows in the box: a rail is taller than wide, a bar never.
            let as_rail = match plain {
                Some(shape) => shape == NavigationBarVariant::Rail,
                None => size.height > size.width,
            };
            let extent = Some((as_rail, if as_rail { size.width } else { size.height }));
            if *measured.peek() != extent {
                measured.set(extent);
            }
        }
    });
    let switched = plain.is_none().then(|| runs.clone());
    use_css(
        publishes.then(|| {
            let extent_of = |of: bool| {
                measured()
                    .filter(|(of_rail, _)| *of_rail == of)
                    .map(|(_, extent)| extent)
            };
            let css = match (&switched, rail) {
                (Some(runs), _) => {
                    publish_runs_css(runs, docked, extent_of(false), extent_of(true))
                }
                (None, true) => publish_rail_css(extent_of(true)),
                (None, false) => publish_css(extent_of(false)),
            };
            Stylesheet::from(css.as_str())
        }),
        CssLayer::Framework,
    );
    // Breakpoints go in the user layer, or the framework base rule wins; before `props.sx`, so it still overrides.
    let sx: Input<Sx> = match &switched {
        None => props.sx.clone(),
        Some(runs) => runs
            .iter()
            .fold(sx(), |sx, (query, shape)| {
                sx.media(query.clone(), shape_sx(*shape, position))
            })
            .and(props.sx.clone().unwrap_or_default())
            .into(),
    };

    let variables: Input<Variables> =
        pill_variables(props.color.as_ref(), theme.navigation_bar.color)
            .with(Z_INDEX_HEADER.override_var(), props.z_index.resolve(None))
            .into();
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(position.state_name(), true)
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
        .sx(&sx)
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

    /// Todo 4127: one run per shape, each bounded just under the next one's breakpoint.
    #[test]
    fn a_variant_per_breakpoint_runs_between_queries() {
        use NavigationBarVariant::{Bar, Rail};
        let runs = layout_runs(&crate::theme::responsive(Bar).sm(Rail).md(Rail).xl(Bar));
        assert_eq!(
            runs,
            [
                ("(max-width: 47.99875rem)".to_string(), Bar),
                (
                    "(min-width: 48rem) and (max-width: 87.99875rem)".to_string(),
                    Rail
                ),
                ("(min-width: 88rem)".to_string(), Bar),
            ]
        );
        assert_eq!(layout_runs(&Responsive::new(Rail).sm(Rail)).len(), 1);
    }

    /// Todo 4127: each run publishes its own extent under its query; a static bar none.
    #[test]
    fn each_run_publishes_its_extent_under_its_query() {
        use NavigationBarVariant::{Bar, Rail};
        let runs = layout_runs(&crate::theme::responsive(Bar).sm(Rail));
        let css = publish_runs_css(&runs, true, Some(70.0), None);
        assert_eq!(
            css,
            "@media (max-width: 47.99875rem){:root{--lsx-navigation-bar-height:70px;\
             scroll-padding-bottom:var(--lsx-navigation-bar-height);\
             --lsx-scroll-padding-bottom:var(--lsx-navigation-bar-height);}}\
             @media (min-width: 48rem){:root{--lsx-navigation-rail-width:88px;}}"
        );
        let css = publish_runs_css(&runs, false, None, Some(96.0));
        assert_eq!(
            css,
            "@media (min-width: 48rem){:root{--lsx-navigation-rail-width:96px;}}"
        );
    }

    /// Todo 4127: a fixed rail run is docked and stretched under its query.
    #[test]
    fn a_rail_run_carries_its_rules_under_its_query() {
        let runs = layout_runs(
            &crate::theme::responsive(NavigationBarVariant::Bar).sm(NavigationBarVariant::Rail),
        );
        let sx = runs.iter().fold(sx(), |sx, (query, shape)| {
            sx.media(
                query.clone(),
                shape_sx(*shape, NavigationBarPosition::Fixed),
            )
        });
        let css = Stylesheet::from(&sx);
        let css = css.as_str();
        let wide = css.find("@media (min-width: 48rem)").expect(css);
        assert!(css[wide..].contains("position:fixed;"), "{css}");
        assert!(css[wide..].contains("align-self:stretch;"), "{css}");
        assert!(css[..wide].contains("inset-inline:0;"), "{css}");
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
