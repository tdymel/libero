use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, Part, ScaleOrCss, States, Variables, base_color, base_props,
            coarse_hit_area_sx, contrast_color, fill_color, focus_ring_sx, literal_contrast,
            on_state_sx, parts_enum, variables,
        },
        layout::{BoxStyle, use_box},
    },
    context::IconSlot,
    hooks::{ElementHandle, use_element, use_localization, use_silent_focus_out_of},
    localization::fill,
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        CssVar, PAGINATION_ACTIVE_BACKGROUND, PAGINATION_ACTIVE_COLOR, PAGINATION_BORDER,
        PAGINATION_CONTROL_SIZE, PAGINATION_GAP, PaginationDefaults, Size, SizeCss,
    },
    use_theme,
    utils::warn,
};

use super::range::{PaginationItem, pagination_range};

/// Which control a `label` closure names; no `Ellipsis`, which is hidden.
/// The current page is `Page 4`, every other `Go to page 4`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaginationLabel {
    Page { number: u32, current: bool },
    First,
    Previous,
    Next,
    Last,
}

/// Per instance, not on the theme: `radius` is a prop the controls read off the nav.
const PAGINATION_RADIUS: CssVar = CssVar::new("--lsx-pagination-radius");

/// Flex `<li>`s drop the line box whose descender set the digits 2.5px below the
/// arrows. Chromium still reports `list` / `listitem`.
static PAGINATION_LIST_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap(PAGINATION_GAP.value())
        // Eleven `md` controls are 353px: without wrapping they overflow a phone.
        .flex_wrap("wrap")
        .list_style("none")
        .margin("0")
        .padding("0")
        .selector("& > li", sx().display("flex"))
});

/// One style for every control, arrows included. The size vars sit here, not
/// on the `<nav>`, so all controls share one class.
static PAGINATION_CONTROL_SX: StaticSx = StaticSx::new(|| {
    PaginationDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .padding("0 0.35em")
        .border(format!("1px solid {}", PAGINATION_BORDER.value()))
        .border_radius(PAGINATION_RADIUS.value())
        .background("transparent")
        .color("inherit")
        .cursor("pointer")
        .font_family("inherit")
        .letter_spacing("inherit")
        .line_height("1")
        .position("relative")
        .and(coarse_hit_area_sx("::before"))
        // On-state ring, so the current page is not a fill alone (todo 631).
        .when(
            "current",
            sx().background(PAGINATION_ACTIVE_BACKGROUND.value())
                .color(PAGINATION_ACTIVE_COLOR.value())
                .border_color("transparent")
                .and(on_state_sx(None))
                .focus_visible(focus_ring_sx()),
        )
        .when("disabled", sx().opacity("0.5").cursor("default"))
        // Page buttons have no `disabled` state: `disabled` soft-disables them (todo 2418).
        .selector(
            "&[aria-disabled='true']",
            sx().opacity("0.5").cursor("default"),
        )
        // A disabled `Fieldset` disables them natively (todo 514).
        .selector("&:disabled", sx().opacity("0.5").cursor("default"))
        // The row runs right to left, so the arrows point the other way.
        .rtl(sx().selector("& svg", sx().transform("scaleX(-1)")))
});

static PAGINATION_ELLIPSIS_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .pointer_events("none")
        .user_select("none")
});

parts_enum! {
    /// [`Pagination`]'s inner parts, for its `parts` prop.
    pub enum PaginationPart {
        /// The `<ul>` of controls.
        List = "list" => "& > [data-slot='list']",
        /// A page number button; the current one has `aria-current="page"`.
        Page = "page" => "& > [data-slot='list'] > li > [data-slot='page']",
        /// The first, previous, next and last buttons.
        Arrow = "arrow" => "& > [data-slot='list'] > li > [data-slot='arrow']",
        /// The `…` between page ranges.
        Ellipsis = "ellipsis" => "& > [data-slot='list'] > li > [data-slot='ellipsis']",
    }
}

base_props! {
    parts(PaginationPart);
    pub struct PaginationProps {
        /// Page count. `0` renders nothing.
        total: u32,
        /// 1-based and clamped into range. Strictly controlled.
        page: u32,
        /// Asks for a new page. Without it the page can never change.
        #[props(default)]
        onchange: Option<EventHandler<u32>>,
        /// Names the `<nav>` landmark, so two paginations are distinguishable.
        aria_label: String,
        /// Pages shown beside the current one.
        #[props(default)]
        siblings: Option<u8>,
        #[props(default)]
        boundaries: Option<u8>,
        #[props(default, into)]
        size: Input<Size>,
        /// A size step on the radius scale, or any CSS, as `radius: "0"`.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Fill of the current page.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Disables every control at once, as `aria-disabled`: focus stays where it is.
        #[props(default)]
        disabled: Option<bool>,
        /// Previous and next. On by default.
        #[props(default)]
        with_controls: Option<bool>,
        /// First and last. Off by default.
        #[props(default)]
        with_edges: Option<bool>,
        /// Every accessible name, replacing `PaginationLabels`. Runs during render.
        #[props(default)]
        label: Option<Callback<PaginationLabel, String>>,
    }
}

/// A row of page buttons in a named `<nav>`, not links; wire `?page=2` URLs yourself.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Pagination;
/// # fn app() -> Element {
/// let mut page = use_signal(|| 1);
/// rsx! {
///     Pagination {
///         total: 10,
///         page: page(),
///         onchange: move |next| page.set(next),
///         aria_label: "Results pages",
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/pagination>
#[component]
pub fn Pagination(props: PaginationProps) -> Element {
    let theme = use_theme();
    let defaults = theme.pagination;
    let labels = use_localization().pagination;

    let list = use_element();

    let size = props.size.copied_or(defaults.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), defaults.radius).resolve(SizeCss::RADIUS);
    let siblings = props.siblings.unwrap_or(defaults.siblings);
    let boundaries = props.boundaries.unwrap_or(defaults.boundaries);
    let disabled = props.disabled.unwrap_or(false);
    let with_controls = props.with_controls.unwrap_or(true);
    let with_edges = props.with_edges.unwrap_or(false);

    if props.onchange.is_none() {
        warn("Pagination: without `onchange` the page can never change.");
    }

    // Fill and text from one ramp, so no `autoContrast` knob is needed.
    let picked = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or(ThemeAwareValue::Color(defaults.color));
    let active = base_color(Some(&picked));
    // A literal fill has no ramp, so it gets black or white (todo 1587).
    let on_active = contrast_color(&active)
        .and_then(|on_active| on_active.resolve(None))
        .or_else(|| literal_contrast(&active));

    let variables: Variables = variables()
        .with(PAGINATION_ACTIVE_BACKGROUND, fill_color(&active))
        .with(PAGINATION_RADIUS, radius.clone())
        .with(PAGINATION_ACTIVE_COLOR, on_active);

    let total = props.total;
    let page = props.page.clamp(1, total.max(1));

    let owed_focus = use_pagination_focus_repair(list, page);
    // Leaving before the page moved (a rejected or slow `onchange`) forgives
    // the debt; the blur of the control the move disabled comes after (todo 1589).
    let forgive = move || {
        let mut owed_focus = owed_focus;
        if *owed_focus.peek() == Some(page) {
            owed_focus.set(None);
        }
    };
    // Blitz's Tab and libero's `focus()` fire no `focusout` (todo 1639).
    use_silent_focus_out_of(list, forgive);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .into();

    let nav = use_box()
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&variables.into())
        .prepare();

    let list_box = use_box().framework_sx(&PAGINATION_LIST_SX).prepare();
    // Two frames, as `use_box` is a hook and can't run in the loop; a hand-written
    // `data-state` would be a second attribute `when("current")` never sees.
    let control_states: Input<States> = States::default().with(size.state_name(), true).into();
    let current_states: Input<States> = States::default()
        .with(size.state_name(), true)
        .with("current", true)
        .into();
    let control = use_box()
        .framework_sx(&PAGINATION_CONTROL_SX)
        .states(&control_states)
        .prepare();
    let control_current = use_box()
        .framework_sx(&PAGINATION_CONTROL_SX)
        .states(&current_states)
        .prepare();
    let ellipsis = use_box().framework_sx(&PAGINATION_ELLIPSIS_SX).prepare();

    let onchange = props.onchange;
    // Copying the `Signal` inside keeps this an `Fn` for several handlers.
    let go = move |target: u32, will_disable: bool| {
        if will_disable && onchange.is_some() {
            let mut owed_focus = owed_focus;
            owed_focus.set(Some(page));
        }
        if let Some(onchange) = onchange {
            onchange.call(target);
        }
    };

    // The target is worked out at the press, so an arrow's props hold still
    // while the page moves and it skips the render.
    let onarrow = use_callback(move |label: PaginationLabel| {
        let target = match label {
            PaginationLabel::First => 1,
            PaginationLabel::Previous => page.saturating_sub(1).max(1),
            PaginationLabel::Next => page.saturating_add(1).min(total),
            PaginationLabel::Last | PaginationLabel::Page { .. } => total,
        };
        go(target, target == 1 || target == total)
    });

    // Not one disabled `1`: an empty result set has no pages.
    if total == 0 {
        return rsx! {};
    }

    let name = |label: PaginationLabel| match props.label {
        Some(custom) => custom.call(label),
        None => match label {
            PaginationLabel::Page {
                number,
                current: true,
            } => fill(labels.current_page, &[("n", &number)]),
            PaginationLabel::Page { number, .. } => fill(labels.page, &[("n", &number)]),
            PaginationLabel::First => labels.first.to_string(),
            PaginationLabel::Previous => labels.previous.to_string(),
            PaginationLabel::Next => labels.next.to_string(),
            PaginationLabel::Last => labels.last.to_string(),
        },
    };

    let arrow = |label: PaginationLabel, at_end: bool| {
        rsx! {
            li {
                PaginationArrow {
                    label,
                    name: name(label),
                    states: control_states.clone(),
                    size,
                    radius: radius.clone(),
                    disabled: disabled || at_end,
                    // An end arrow stays natively disabled; the focus repair covers it.
                    focusable: disabled,
                    onarrow,
                }
            }
        }
    };

    // Keyed by value: an index key would re-label the focused button as the range shifts.
    let items: Vec<(String, PaginationItem)> = pagination_range(total, page, siblings, boundaries)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let key = match item {
                PaginationItem::Page(number) => format!("page-{number}"),
                PaginationItem::Ellipsis => format!("ellipsis-{index}"),
            };
            (key, item)
        })
        .collect();

    let list_element = list_box
        .clone()
        .element(&list)
        .attr("data-slot", PaginationPart::List.slot())
        // Safari with VoiceOver drops list semantics from a `list-style: none` list.
        .attr("role", "list")
        .event("onfocusout", move |_: Event<FocusData>| forgive())
        .render(
            HtmlTag::Ul,
            vec![],
            rsx! {
                if with_edges {
                    {arrow(PaginationLabel::First, page == 1)}
                }
                if with_controls {
                    {arrow(PaginationLabel::Previous, page == 1)}
                }
                for (key , item) in items.iter() {
                    li {
                        key: "{key}",
                        // Hidden at the `<li>`: an empty list item would still be counted.
                        "aria-hidden": matches!(item, PaginationItem::Ellipsis)
                            .then_some("true"),
                        {
                            match item {
                                PaginationItem::Ellipsis => {
                                    ellipsis
                                        .clone()
                                        .attr("data-slot", PaginationPart::Ellipsis.slot())
                                        .render(HtmlTag::Span, vec![], rsx! { "…" })
                                }
                                PaginationItem::Page(number) => {
                                    let number = *number;
                                    let current = number == page;
                                    page_button(
                                        match current {
                                            true => control_current.clone(),
                                            false => control.clone(),
                                        },
                                        number,
                                        name(PaginationLabel::Page { number, current }),
                                        current,
                                        disabled,
                                        go,
                                    )
                                }
                            }
                        }
                    }
                }
                if with_controls {
                    {arrow(PaginationLabel::Next, page == total)}
                }
                if with_edges {
                    {arrow(PaginationLabel::Last, page == total)}
                }
            },
        );
    nav.attr("aria-label", props.aria_label)
        .render(HtmlTag::Nav, props.attributes, list_element)
}

/// First, previous, next or last. Its own scope: `ActionIcon` takes the icon
/// as children, so inline it redrew on every page.
#[component]
fn PaginationArrow(
    label: PaginationLabel,
    name: String,
    states: Input<States>,
    size: Size,
    radius: String,
    disabled: bool,
    focusable: bool,
    onarrow: Callback<PaginationLabel>,
) -> Element {
    let (slot, icon) = match label {
        PaginationLabel::First => (IconSlot::ChevronFirst, lucide::chevron_first::outlined),
        PaginationLabel::Previous => (IconSlot::ChevronLeft, lucide::chevron_left::outlined),
        PaginationLabel::Next => (IconSlot::ChevronRight, lucide::chevron_right::outlined),
        PaginationLabel::Last | PaginationLabel::Page { .. } => {
            (IconSlot::ChevronLast, lucide::chevron_last::outlined)
        }
    };
    let icon = rsx! { Glyph { slot, icon } };
    rsx! {
        ActionIcon {
            "data-slot": PaginationPart::Arrow.slot(),
            aria_label: name,
            // The page buttons' style; `size` and `radius` via `ActionIcon`'s props.
            sx: &PAGINATION_CONTROL_SX,
            states,
            size: PAGINATION_CONTROL_SIZE.value(size),
            radius,
            disabled,
            focusable_when_disabled: focusable,
            onclick: move |_| {
                if !disabled {
                    onarrow.call(label)
                }
            },
            {icon}
        }
    }
}

/// Moves focus to the current page when the clicked control disables itself.
/// The signal holds the page that click left, and is spent once `page` changes.
fn use_pagination_focus_repair(list: ElementHandle, page: u32) -> Signal<Option<u32>> {
    let mut owed_focus = use_signal(|| None);

    use_effect(use_reactive!(|(page,)| {
        // Runs on the render where `page` changed, not the click's.
        let _ = page;
        // `peek`: subscribing would spend the debt before an async `onchange` disables the control.
        if owed_focus.peek().is_none() {
            return;
        }
        owed_focus.set(None);
        // Chromium keeps a just-disabled button as `activeElement` until its
        // next focus fixup, so it still matches `:focus` here.
        if list.query_selector(":focus:not(:disabled)").is_ok() {
            return;
        }
        let _ = list
            .query_selector("[aria-current=\"page\"]")
            .and_then(|current| current.focus());
    }));

    owed_focus
}

/// One page number; `frame` is prepared outside the loop, as `use_box` is a hook.
fn page_button(
    frame: BoxStyle,
    number: u32,
    label: String,
    current: bool,
    disabled: bool,
    go: impl Fn(u32, bool) + 'static,
) -> Element {
    frame
        .attr("data-slot", PaginationPart::Page.slot())
        .attr("type", "button")
        .attr("aria-label", label)
        .attr("aria-current", current.then_some("page"))
        // Soft: a native `disabled` would drop the focused button's focus to the page.
        .attr("aria-disabled", disabled.then_some("true"))
        .event("onclick", move |_: Event<MouseData>| {
            // The current page is no change: no `onchange`, like Select's same-value pick.
            if !disabled && !current {
                go(number, false)
            }
        })
        .render(HtmlTag::Button, vec![], rsx! { "{number}" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<PaginationPart>(),
            [
                ("list", "& > [data-slot='list']"),
                ("page", "& > [data-slot='list'] > li > [data-slot='page']"),
                ("arrow", "& > [data-slot='list'] > li > [data-slot='arrow']"),
                (
                    "ellipsis",
                    "& > [data-slot='list'] > li > [data-slot='ellipsis']"
                ),
            ]
        );
    }
}
