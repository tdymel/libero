use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, HtmlTag, Input, States, Variables,
        common::{base_color, base_props, contrast_color, fill_color},
        layout::use_box,
        variables,
    },
    hooks::use_element,
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        CssVar, PAGINATION_ACTIVE_BACKGROUND, PAGINATION_ACTIVE_COLOR, PAGINATION_BORDER,
        PAGINATION_CONTROL_SIZE, PAGINATION_GAP, PaginationDefaults, Size, SizeCss,
    },
    use_theme,
    utils::warn,
};

use super::glyphs::{FirstIcon, LastIcon, NextIcon, PreviousIcon};
use super::range::{PaginationItem, pagination_range};

/// Which control a caller's `label` closure is naming.
///
/// Deliberately **not** [`PaginationItem`]: that carries `Ellipsis`, which is
/// `aria-hidden` and never focusable, so it has no name to override and every
/// caller writing the exhaustive match would get a dead branch.
///
/// `current` rides on [`Page`](Self::Page) rather than being a sixth variant
/// because it is the same control with a different name, which is the whole of
/// MUI's asymmetry: the current page is `Page 4`, every other `Go to page 4`.
/// `aria-current` already says "current", so repeating "go to" on the page you
/// are on is a lie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaginationLabel {
    Page { number: u32, current: bool },
    First,
    Previous,
    Next,
    Last,
}

/// Per instance, not on the theme: `radius` is a prop, and the controls read it
/// off the nav they are inside.
const PAGINATION_RADIUS: CssVar = CssVar::new("--lsx-pagination-radius");

/// The `<li>`s are flex containers, and that is what keeps the digits level
/// with the arrows.
///
/// A control is `inline-flex`, so a `display: list-item` `<li>` wraps it in a
/// **line box** - and a line box reserves the strut's descender below the
/// baseline whether anything sits there or not. Where that baseline falls
/// differs between the two kinds of control: a page button's first flex item is
/// its digit, so the button's baseline is the digit's and the descender space
/// lands *inside* the button's own height; an arrow's only item is an `<svg>`,
/// which has no baseline, so the button's baseline is synthesized at its bottom
/// edge and the descender is added *underneath* it. The arrow's `<li>` comes
/// out one strut-descender taller than the page button's, `align-items: center`
/// on this list centres the short one against the tall one, and every digit
/// sits half a descender lower than every arrow - 2.5px at `md`, and 3.75px at
/// `xs`, where the button is shorter than the strut's ascent as well.
///
/// `display: flex` on the `<li>` removes the line box, so both kinds are
/// exactly their control's height. Measured in Chromium: the list goes from
/// 37px to 32px at `md` and the offset from 2.5px to 0. `list-item` is what is
/// being given up, and the AX tree was checked rather than assumed - Chromium
/// still reports `list` / `listitem` for the `<ul>` and its children.
static PAGINATION_LIST_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("center")
        .gap(PAGINATION_GAP.value())
        .list_style("none")
        .margin("0")
        .padding("0")
        .selector("& > li", sx().display("flex"))
});

/// One style for every control, arrows included.
///
/// The per-size block is [`PaginationDefaults::theme_vars`], and it belongs
/// **here rather than on the `<nav>`**: the nav is not the thing being sized,
/// and a per-size rule on it would set a min-width and a font-size on a wrapper
/// that has neither. Each control carries the size state instead, so every
/// control on the page shares one recycled class.
///
/// `min-width` rather than `width`, because `"100"` has to grow past the square
/// an `ActionIcon` would otherwise be.
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
        .line_height("1")
        .when(
            "current",
            sx().background(PAGINATION_ACTIVE_BACKGROUND.value())
                .color(PAGINATION_ACTIVE_COLOR.value())
                .border_color("transparent"),
        )
        .when("disabled", sx().opacity("0.5").cursor("default"))
});

static PAGINATION_ELLIPSIS_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .pointer_events("none")
        .user_select("none")
});

base_props! {
    pub struct PaginationProps {
        /// Page count. `0` renders nothing.
        total: u32,
        /// 1-based and clamped into range. Strictly controlled, like
        /// `Tabs::value`.
        page: u32,
        /// Asks for a new page. Without it the selection can never change.
        #[props(default)]
        onchange: Option<EventHandler<u32>>,
        /// Names the `<nav>` landmark. Required: two paginations on one page
        /// have to be distinguishable.
        aria_label: String,
        /// `Option`, not `Input`: `Input` exists for values that chain through
        /// `sx`/theme resolution, and these are plain scalars with a theme
        /// default - the `disabled`/`with_controls` shape. It also keeps `u8`
        /// out of `input_from!`, which is a shared file this does not need to
        /// touch.
        #[props(default)]
        siblings: Option<u8>,
        #[props(default)]
        boundaries: Option<u8>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        radius: Input<Size>,
        /// Fill of the current page.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Disables every control at once.
        #[props(default)]
        disabled: Option<bool>,
        /// Previous and next. On by default.
        #[props(default)]
        with_controls: Option<bool>,
        /// First and last. Off by default.
        #[props(default)]
        with_edges: Option<bool>,
        /// Overrides every accessible name. Runs during render, so it can read
        /// a locale the theme's `PaginationLabels` cannot express - a language
        /// that does not put the number last.
        #[props(default)]
        label: Option<Callback<PaginationLabel, String>>,
    }
}

/// A row of page controls: a named `<nav>` landmark around a `<ul>` of real
/// buttons, with an ellipsis range that never reflows as you click through it.
///
/// Strictly controlled - `page` is the caller's and `onchange` asks for a new
/// one. **It renders buttons, not links**: a pagination is state, and whether a
/// navigation happens is not part of its contract. A content listing that wants
/// shareable `?page=2` URLs wires its own links around this component's state.
#[component]
pub fn Pagination(props: PaginationProps) -> Element {
    let theme = use_theme();
    let defaults = theme.pagination;
    let labels = theme.pagination_labels;

    let list = use_element();
    // Set when a click is about to disable the control under the pointer, spent
    // by the effect below. `FileField`'s `FocusDebt` shape.
    let mut owed_focus = use_signal(|| false);

    let size = props.size.copied_or(defaults.size);
    let radius = props.radius.copied_or(defaults.radius);
    let siblings = props.siblings.unwrap_or(defaults.siblings);
    let boundaries = props.boundaries.unwrap_or(defaults.boundaries);
    let disabled = props.disabled.unwrap_or(false);
    let with_controls = props.with_controls.unwrap_or(true);
    let with_edges = props.with_edges.unwrap_or(false);

    if props.onchange.is_none() {
        warn("Pagination: without `onchange` the page can never change.");
    }

    // A themed `Color` becomes a shade, and a prop-supplied value wins. Through
    // `base_color`/`contrast_color` rather than by hand, so the fill and its
    // readable text come from the same ramp - which is why no `autoContrast`
    // knob is needed.
    let picked = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or(ThemeAwareValue::Color(defaults.color));
    let active = base_color(Some(&picked));
    let on_active = contrast_color(&active);

    let mut variables: Variables = variables()
        .with(PAGINATION_ACTIVE_BACKGROUND, fill_color(&active))
        .with(PAGINATION_RADIUS, SizeCss::RADIUS.value(radius));
    if let Some(on_active) = on_active {
        variables = variables.with(PAGINATION_ACTIVE_COLOR, on_active.resolve(None));
    }

    let total = props.total;
    let page = props.page.clamp(1, total.max(1));

    // The repair: the control the user just clicked disables in place, so focus
    // falls to `<body>` with nothing removed. Spent on the render where `page`
    // has actually changed, and skipped if focus already went somewhere inside.
    use_effect(use_reactive!(|(page,)| {
        // `page` is read only to make it the dependency: the repair has to run
        // on the render where the control actually disabled, not on the one
        // where the click happened.
        let _ = page;
        // `peek`, not a read. Reading would subscribe this effect to the debt
        // as well, so setting the flag in the click handler would run the
        // repair immediately - while the control is still enabled and still
        // holds focus. It would find focus inside, return early, and clear the
        // debt; then the caller's `onchange` resolves, the control disables,
        // and focus falls to the document with nothing left owing.
        //
        // That is invisible when `onchange` is synchronous, because both
        // signals land in one batch, and it is the normal case for the table
        // pagination this component is mostly for - the caller fetches, then
        // sets the page. Found by Karen3.
        if !*owed_focus.peek() {
            return;
        }
        owed_focus.set(false);
        if list.query_selector(":focus").is_ok() {
            return;
        }
        let _ = list
            .query_selector("[aria-current=\"page\"]")
            .and_then(|current| current.focus());
    }));

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .into();

    let nav = use_box()
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables.into())
        .prepare();

    let list_box = use_box().framework_sx(&PAGINATION_LIST_SX).prepare();
    // Two frames rather than one plus a hand-written `data-state`: the
    // `when("current", ..)` above matches the box's own state mechanism, and an
    // `attr("data-state", ..)` would render a second attribute it never sees.
    // `use_box` is a hook, so this cannot be done inside the loop.
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

    // `0` renders nothing at all rather than one disabled `1`: an empty result
    // set has no pages, and a lone control implies otherwise.
    if total == 0 {
        return rsx! {};
    }

    let name = |label: PaginationLabel| match props.label {
        Some(custom) => custom.call(label),
        None => match label {
            PaginationLabel::Page {
                number,
                current: true,
            } => format!("{} {number}", labels.current_page_label),
            PaginationLabel::Page { number, .. } => format!("{} {number}", labels.page_label),
            PaginationLabel::First => labels.first_label.to_string(),
            PaginationLabel::Previous => labels.previous_label.to_string(),
            PaginationLabel::Next => labels.next_label.to_string(),
            PaginationLabel::Last => labels.last_label.to_string(),
        },
    };

    let onchange = props.onchange;
    // `Signal` is `Copy`, so the copy inside addresses the very same value and
    // keeps this an `Fn` - it is called from several handlers.
    let go = move |target: u32, will_disable: bool| {
        // No handler means no page change, so nothing disables and nothing owes
        // focus anywhere.
        if will_disable && onchange.is_some() {
            let mut owed_focus = owed_focus;
            owed_focus.set(true);
        }
        if let Some(onchange) = onchange {
            onchange.call(target);
        }
    };

    let arrow_states = control_states.clone();
    let arrow = |label: PaginationLabel, target: u32, at_end: bool, icon: Element| {
        let control_disabled = disabled || at_end;
        rsx! {
            li {
                ActionIcon {
                    aria_label: name(label),
                    // The same style the page buttons use, so an arrow and a
                    // number are one row rather than two shapes. `size` and
                    // `radius` go through `ActionIcon`'s own props so its
                    // geometry agrees rather than being overridden.
                    sx: &PAGINATION_CONTROL_SX,
                    states: arrow_states.clone(),
                    size: PAGINATION_CONTROL_SIZE.value(size),
                    radius: SizeCss::RADIUS.value(radius),
                    disabled: control_disabled,
                    onclick: move |_| {
                        if !control_disabled {
                            go(target, target == 1 || target == total)
                        }
                    },
                    {icon}
                }
            }
        }
    };

    // Keyed by *value*, not position: the range shifts as you page, so an index
    // key would keep the focused node and re-label it, leaving the user on a
    // button they did not click. By value the clicked node stays the page it
    // was, which is also the node that becomes `aria-current`. Built here
    // because rsx only takes a key as a formatted string.
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

    rsx! {
        {
            nav.attr("aria-label", props.aria_label)
                .render(
                    HtmlTag::Nav,
                    props.attributes,
                    rsx! {
                        {
                            list_box
                                .clone()
                                .element(&list)
                                .render(
                                    HtmlTag::Ul,
                                    vec![],
                                    rsx! {
                                        if with_edges {
                                            {arrow(PaginationLabel::First, 1, page == 1, rsx! { FirstIcon {} })}
                                        }
                                        if with_controls {
                                            {arrow(PaginationLabel::Previous, page.saturating_sub(1).max(1), page == 1, rsx! { PreviousIcon {} })}
                                        }
                                        for (key , item) in items.iter() {
                                            li {
                                                key: "{key}",
                                                // The gap is hidden at the `<li>`, not just on
                                                // the text inside it. Hiding only the span
                                                // silences the `…` but leaves an empty list
                                                // item in the accessibility tree, so the list
                                                // announces more entries than it has.
                                                "aria-hidden": matches!(item, PaginationItem::Ellipsis)
                                                    .then_some("true"),
                                                {
                                                    match item {
                                                        PaginationItem::Ellipsis => {
                                                            ellipsis
                                                                .clone()
                                                                .render(HtmlTag::Span, vec![], rsx! { "…" })
                                                        }
                                                        PaginationItem::Page(number) => {
                                                            let number = *number;
                                                            let current = number == page;
                                                            let frame = match current {
                                                                true => control_current.clone(),
                                                                false => control.clone(),
                                                            };
                                                            frame
                                                                .attr("type", "button")
                                                                .attr("aria-label", name(PaginationLabel::Page { number, current }))
                                                                .attr("aria-current", current.then_some("page"))
                                                                .attr("disabled", disabled)
                                                                .event(
                                                                    "onclick",
                                                                    move |_: Event<MouseData>| {
                                                                        if !disabled {
                                                                            go(number, false)
                                                                        }
                                                                    },
                                                                )
                                                                .render(HtmlTag::Button, vec![], rsx! { "{number}" })
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if with_controls {
                                            {arrow(PaginationLabel::Next, (page + 1).min(total), page == total, rsx! { NextIcon {} })}
                                        }
                                        if with_edges {
                                            {arrow(PaginationLabel::Last, total, page == total, rsx! { LastIcon {} })}
                                        }
                                    },
                                )
                        }
                    },
                )
        }
    }
}
