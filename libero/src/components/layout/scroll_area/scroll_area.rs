use std::rc::Rc;

use dioxus::{dioxus_core::AttributeValue, prelude::*};

use super::{
    handle::{ScrollAreaHandle, inline_x, scroll_to_percent},
    keys::scroll_on_key,
    scrollbars::{DrawnBars, ScrollAreaBars, ScrollMetrics},
    viewport::{ContentOffsets, ScrollGeometry, ScrollViewport, WindowSpec},
};
use crate::{
    components::{
        common::{
            FOCUSABLE_SELECTOR, HtmlTag, Input, States, Variables, base_props, input_from_str,
            names_itself, parts_enum, variables,
        },
        layout::use_box,
    },
    hooks::{ElementHandle, use_content_changes, use_element, use_resize_fallback, use_theme},
    platform::{
        Dimensions, ElementApi, OBSERVE_ATTR, PlatformError, Read, SCROLL_QUIET, TimerSubscription,
        clips_z_indexed, draws_own_scrollbars, fires_scroll_end, fires_scroll_on_scroll_to,
        has_match_by_tag, on_element_scroll, scroll, scroll_range, scrolls_on_keys, timer,
        when_free, when_laid_out,
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        ColorCss, ColorShade, CssVar, SKELETON_COLOR, ScrollAxis, ScrollbarSize,
        ScrollbarVisibility,
    },
    utils::warn,
};

input_from_str!(ScrollAxis);

input_from_str!(ScrollbarVisibility);

input_from_str!(ScrollbarSize);

/// Scroll position as a percent (0-100) of each axis's scrollable range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollPositionEvent {
    Start(f64, f64),
    Change(f64, f64),
    End(f64, f64),
}

const SCROLL_AREA_THUMB_VAR: CssVar = CssVar::new("--lsx-scroll-area-thumb-color");
/// Rows a `Virtualize` child skipped, as padding so the range spans the whole list.
const SCROLL_AREA_LEADING_VAR: CssVar = CssVar::new("--lsx-scroll-area-leading");
const SCROLL_AREA_TRAILING_VAR: CssVar = CssVar::new("--lsx-scroll-area-trailing");
/// A windowed row's pitch, for the placeholder rows in the padding.
const SCROLL_AREA_PITCH_VAR: CssVar = CssVar::new("--lsx-scroll-area-pitch");

/// A reported position, and whether it ended the scroll.
type ScrollHandler = Box<dyn FnMut(&ScrollData, bool)>;

/// The drawn bars' `aria-hidden` layer, the root's first child. Plumbing, not a part.
pub(super) const SCROLLBARS_SLOT: &str = "scrollbars";

parts_enum! {
    /// [`ScrollArea`]'s inner parts, for its `parts` prop. They exist only while the
    /// area draws its own bars: `always` in a browser.
    pub enum ScrollAreaPart {
        /// A track; `data-orientation` is `vertical` or `horizontal`.
        Scrollbar = "scrollbar" => "& > [data-slot='scrollbars'] > * > [data-slot='scrollbar']",
        /// The thumb inside a track.
        Thumb = "thumb" => "& > [data-slot='scrollbars'] > * > [data-slot='scrollbar'] > [data-slot='thumb']",
    }
}

/// `Scroll` behaves as `Hover`: nothing here can yet fade the scrollbar out
/// after an idle timeout. See `ScrollbarVisibility`.
fn visibility_token(visibility: ScrollbarVisibility) -> &'static str {
    match visibility {
        ScrollbarVisibility::Scroll => ScrollbarVisibility::Hover.state_name(),
        visibility => visibility.state_name(),
    }
}

/// Crate-internal base styles replacing `ScrollArea`'s own on the framework
/// layer, below a caller's `sx`. Build it from [`scroll_area_base`].
#[doc(hidden)]
#[allow(unnameable_types)]
#[derive(Clone, Copy, PartialEq)]
pub struct ScrollAreaBase(pub(crate) &'static StaticSx);

/// `ScrollArea`'s own base styles with `sx` merged on top: a property `sx`
/// declares again replaces the area's.
pub(crate) fn scroll_area_base(sx: crate::sx::Sx) -> crate::sx::Sx {
    SCROLL_AREA_BASE_SX.clone().and(sx)
}

/// The thumb's colour, native or drawn.
pub(super) fn thumb_color() -> String {
    // Shade 6: shade 5 was 2.07:1 on the light page, under WCAG 1.4.11's 3:1.
    SCROLL_AREA_THUMB_VAR.value_or(ColorCss::MUTED.value(ColorShade::S6))
}

static SCROLL_AREA_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .display("block")
        .width("100%")
        .height("100%")
        .when(
            "axis-vertical",
            sx().overflow_y("auto").overflow_x("hidden"),
        )
        .when(
            "axis-horizontal",
            sx().overflow_x("auto").overflow_y("hidden"),
        )
        .when("axis-both", sx().overflow_x("auto").overflow_y("auto"))
        .when("axis-none", sx().overflow_x("hidden").overflow_y("hidden"))
        .scrollbar_color(format!("{} transparent", thumb_color()))
        .when("visible-hidden", sx().scrollbar_width("none"));
    // A stacking context of its own, so it clips a z-indexed row natively too.
    let base = match clips_z_indexed() {
        true => base,
        false => base.position("relative").z_index("0"),
    };

    ScrollbarSize::ALL.iter().fold(base, |acc, &size| {
        let token = size.state_name();
        let width = size.as_str();
        // Drawn by `ScrollAreaBars` instead: an overlay bar fades out. The
        // area holds their layer and stacks it above its rows.
        let always = match draws_own_scrollbars() {
            true => sx()
                .scrollbar_width("none")
                .position("relative")
                .z_index("0"),
            false => sx().scrollbar_width(width),
        };
        acc.when(format!("visible-always && {token}"), always).when(
            format!("visible-hover && {token}"),
            sx().scrollbar_width("none")
                .hover(sx().scrollbar_width(width))
                .selector(":focus-within", sx().scrollbar_width(width)),
        )
    })
});

/// Reserves the rows a `Virtualize` skipped; `display: contents` until then,
/// so an ordinary area lays out as without it.
static SCROLL_AREA_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().display("contents")
        .when(
            "virtualized",
            sx().display("block")
                .padding_top(SCROLL_AREA_LEADING_VAR.value_or("0px"))
                .padding_bottom(SCROLL_AREA_TRAILING_VAR.value_or("0px")),
        )
        .when("windowed", placeholder_rows_sx())
});

/// Skeleton bars, one per pitch, in the padding only: a fast fling outruns the
/// rows' render, and shows these instead of a blank pane (todo 2131). Forced
/// colours compute the image to `none`, so no bars stray there.
fn placeholder_rows_sx() -> Sx {
    let pitch = SCROLL_AREA_PITCH_VAR.value();
    let bar = format!("min(6px, {pitch} / 4)");
    let color = SKELETON_COLOR.value();
    let rows = format!(
        "repeating-linear-gradient(to bottom, transparent 0 calc({pitch} / 2 - {bar}), \
         {color} 0 calc({pitch} / 2 + {bar}), transparent 0 {pitch})"
    );
    sx().background_image(format!("{rows}, {rows}"))
        .background_size(format!(
            "calc(100% - 2rem) {}, calc(100% - 2rem) {}",
            SCROLL_AREA_LEADING_VAR.value_or("0px"),
            SCROLL_AREA_TRAILING_VAR.value_or("0px")
        ))
        .background_position("1rem top, 1rem bottom")
        .background_repeat("no-repeat")
}

fn scroll_area_variables(color: Option<&ThemeAwareValue>) -> Variables {
    variables().with(SCROLL_AREA_THUMB_VAR, color.and_then(|v| v.resolve(None)))
}

/// Both always, `0px` included: the style attribute is patched per property,
/// so a dropped one would keep the last window's reserve.
fn scroll_area_content_variables(offsets: ContentOffsets, pitch: Option<f64>) -> Variables {
    variables()
        .with(SCROLL_AREA_LEADING_VAR, format!("{}px", offsets.leading))
        .with(SCROLL_AREA_TRAILING_VAR, format!("{}px", offsets.trailing))
        .with(
            SCROLL_AREA_PITCH_VAR,
            pitch.map(|pitch| format!("{pitch}px")),
        )
}

/// The edges the last position rested against, so `on*reached` fires on the
/// rising edge. Inline start and end, so RTL shares the origin.
#[derive(Clone, Copy, Debug, PartialEq)]
struct EdgeState {
    top: bool,
    bottom: bool,
    start: bool,
    end: bool,
    /// The scroll ranges, so a far edge the content grew past counts again.
    max_x: f64,
    max_y: f64,
}

impl EdgeState {
    /// Where a scroll container starts; all-`false` would report the top and
    /// start as newly reached on the first scroll.
    const AT_ORIGIN: Self = Self {
        top: true,
        bottom: false,
        start: true,
        end: false,
        max_x: 0.0,
        max_y: 0.0,
    };

    /// From one scroll position, the x offset counted from the inline start.
    fn at(x: f64, y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            top: y <= 0.0,
            bottom: max_y <= 0.0 || y >= max_y - 1.0,
            start: x <= 0.0,
            end: max_x <= 0.0 || x >= max_x - 1.0,
            max_x,
            max_y,
        }
    }

    /// The edges reached since `previous`; the bottom and end also count when
    /// the content grew, say rows appended, and the scroll followed to its new end.
    fn reached_since(self, previous: Self) -> Self {
        Self {
            top: self.top && !previous.top,
            bottom: self.bottom && (!previous.bottom || self.max_y > previous.max_y + 1.0),
            start: self.start && !previous.start,
            end: self.end && (!previous.end || self.max_x > previous.max_x + 1.0),
            ..self
        }
    }
}

base_props! {
    parts(ScrollAreaPart);
    pub struct ScrollAreaProps {
        /// `"vertical"` (default), `"horizontal"`, `"both"` or `"none"`.
        #[props(default, into)]
        scrollbars: Input<ScrollAxis>,
        /// `"always"` (default), `"hover"`, `"hidden"`, or `"scroll"` (as `"hover"` for now).
        #[props(default, into)]
        scrollbar_visibility: Input<ScrollbarVisibility>,
        /// CSS `scrollbar-width`: `"thin"` (default) or `"auto"`.
        #[props(default, into)]
        scrollbar_size: Input<ScrollbarSize>,
        /// Scrollbar thumb color - track stays transparent.
        #[props(default, into)]
        scrollbar_color: Input<ThemeAwareValue>,
        /// Percent (0-100) to scroll to; re-applied on every change of a bound signal.
        scroll_position_x: Option<f64>,
        /// Percent (0-100) along the vertical axis - see `scroll_position_x`.
        scroll_position_y: Option<f64>,
        /// Always a tab stop, not only while overflowing with nothing focusable
        /// inside. A tab stop needs an `aria-label` or `aria-labelledby`.
        #[props(default)]
        focusable: bool,
        /// From [`use_scroll_area`](super::use_scroll_area), to scroll from an event handler.
        #[props(default)]
        handle: Option<ScrollAreaHandle>,
        /// Crate-internal, see [`ScrollAreaBase`].
        #[doc(hidden)]
        #[props(default)]
        framework_sx: Option<ScrollAreaBase>,
        /// Crate-internal: px the drawn vertical track starts down, below a sticky header.
        #[doc(hidden)]
        #[props(default)]
        bar_inset_top: Option<f64>,
        /// The position as it scrolls; a WebView (desktop, mobile) coalesces it to about once a frame.
        #[props(default)]
        onscroll: Option<EventHandler<ScrollPositionEvent>>,
        /// After the area resized and re-measured itself. A prop, so it doesn't
        /// replace the area's own listener.
        #[props(default)]
        onresize: Option<EventHandler<Event<ResizeData>>>,
        #[props(default)]
        ontopreached: Option<EventHandler<()>>,
        #[props(default)]
        onbottomreached: Option<EventHandler<()>>,
        #[props(default)]
        onleftreached: Option<EventHandler<()>>,
        #[props(default)]
        onrightreached: Option<EventHandler<()>>,
        children: Element,
    }
}

/// Position as a percent of each axis's scrollable range, plus that range
/// in px - `(x%, y%, max_x, max_y)`.
fn scroll_metrics(data: &ScrollData) -> (f64, f64, f64, f64) {
    let (max_x, max_y) = scroll_range(data);
    let percent = |offset: f64, max: f64| if max > 0.0 { offset / max * 100.0 } else { 0.0 };

    (
        percent(inline_x(data.scroll_left()), max_x),
        percent(data.scroll_top(), max_y),
        max_x,
        max_y,
    )
}

/// Measurements of no height asked again: Blitz may not have laid out a box
/// mounted this frame.
pub(super) const UNLAID_TRIES: u8 = 3;

/// Natively an effect runs before layout, with the document borrowed.
fn measure_area(root: ElementHandle, mut geometry: Signal<Option<ScrollGeometry>>, tries: u8) {
    when_laid_out(move || {
        let (size, offset) = (root.dimensions(), root.scroll_offset());
        spawn(async move {
            let measured = match (size.await, offset.await) {
                (Ok(size), _) if size.height <= 0.0 && tries > 0 => {
                    return measure_area(root, geometry, tries - 1);
                }
                (Ok(size), Ok((_, top))) => ScrollGeometry {
                    offset: top,
                    viewport: size.height,
                    ..ScrollGeometry::default()
                },
                _ => ScrollGeometry::default(),
            };
            if *geometry.peek() != Some(measured) {
                geometry.set(Some(measured));
            }
        });
    });
}

/// A scroll moves only the offset: read it once the document is free, not a
/// layout later, so a `Virtualize` follows in the same poll (todo 1873).
fn measure_offset(root: ElementHandle, mut geometry: Signal<Option<ScrollGeometry>>) {
    when_free(move || {
        let offset = root.scroll_offset();
        spawn(async move {
            let (Ok((_, top)), Some(known)) = (offset.await, *geometry.peek()) else {
                return;
            };
            if known.offset != top {
                geometry.set(Some(known.scrolled(top, known.viewport)));
            }
        });
    });
}

/// Reads the area's position once laid out, for `check` (`x`, `y`, `max_x`, `max_y`).
fn measure_edges(root: ElementHandle, check: impl Fn(f64, f64, f64, f64) + 'static) {
    when_laid_out(move || {
        let (size, view, offset) = (root.scroll_size(), root.dimensions(), root.scroll_offset());
        spawn(async move {
            let (Ok(size), Ok(view), Ok((x, y))) = (size.await, view.await, offset.await) else {
                return;
            };
            let max_x = (size.width - view.width).max(0.0);
            let max_y = (size.height - view.height).max(0.0);
            check(inline_x(x), y, max_x, max_y);
        });
    });
}

/// Whether the area has to be a tab stop itself: it overflows on an axis it
/// scrolls, and nothing inside can take focus and scroll it instead.
fn needs_tab_stop(axis: ScrollAxis, view: Dimensions, content: Dimensions, inner: bool) -> bool {
    // A pixel of slack: the view is a rounded border box, the content integer.
    let over = |content: f64, view: f64| content > view + 1.0;
    let (x, y) = (
        over(content.width, view.width),
        over(content.height, view.height),
    );
    !inner
        && match axis {
            ScrollAxis::Vertical => y,
            ScrollAxis::Horizontal => x,
            ScrollAxis::Both => x || y,
            ScrollAxis::None => false,
        }
}

/// Re-reads [`needs_tab_stop`] once laid out. A WebView queries the subtree by
/// the root's `tag`; a renderer that cannot leaves the area as it was.
fn check_tab_stop(
    root: ElementHandle,
    tag: Option<u64>,
    axis: ScrollAxis,
    mut stop: Signal<bool>,
    tries: u8,
) {
    when_laid_out(move || {
        let inner: Read<bool> = match (root.query_selector(FOCUSABLE_SELECTOR), tag) {
            (Ok(_), _) => Box::pin(std::future::ready(Ok(true))),
            (Err(PlatformError::NotFound), _) => Box::pin(std::future::ready(Ok(false))),
            (Err(PlatformError::Unsupported), Some(tag)) => {
                has_match_by_tag(tag, FOCUSABLE_SELECTOR)
            }
            _ => return,
        };
        let (view, content) = (root.dimensions(), root.scroll_size());
        spawn(async move {
            if let (Ok(inner), Ok(view), Ok(content)) = (inner.await, view.await, content.await) {
                if view.height <= 0.0 && tries > 0 {
                    return check_tab_stop(root, tag, axis, stop, tries - 1);
                }
                let next = needs_tab_stop(axis, view, content, inner);
                if *stop.peek() != next {
                    stop.set(next);
                }
            }
        });
    });
}

/// The box around the content. A scope of its own, so a `Virtualize` step
/// redraws only this box's padding, not the whole area (todo 843).
#[component]
fn ScrollAreaContent(content: ElementHandle, children: Element) -> Element {
    let viewport = use_context::<ScrollViewport>();
    // Unvirtualized content is `display: contents` and reserves nothing.
    let (states, variables): (Input<States>, Input<Variables>) = if (viewport.virtualized)() {
        let spec = (viewport.spec)();
        let offsets = spec
            .map(|spec| spec.at((viewport.geometry)()).0.offsets)
            .unwrap_or_default();
        (
            States::default()
                .with("virtualized", true)
                .with("windowed", spec.is_some())
                .into(),
            scroll_area_content_variables(offsets, spec.map(|spec| spec.pitch)).into(),
        )
    } else {
        (Input::None, Input::None)
    };
    use_box()
        .framework_sx(&SCROLL_AREA_CONTENT_SX)
        .states(&states)
        .variables(&variables)
        .prepare()
        .element(&content)
        .render(HtmlTag::Div, Vec::new(), children)
}

/// Scrolls its content with themed scrollbars, filling the parent by default.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::ScrollArea;
/// # fn app() -> Element {
/// rsx! {
///     div { style: "height: 200px",
///         ScrollArea { aria_label: "Log", onbottomreached: move |_| {}, "Rows" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/scroll-area>
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    let theme = use_theme();
    // Always called, for a stable hook order; unmounted under a caller's handle.
    let own = use_element();
    let root = props.handle.map_or(own, |handle| handle.element);

    let scrollbars = props.scrollbars.copied_or(theme.scroll_area.scrollbars);
    let visibility = props
        .scrollbar_visibility
        .copied_or(theme.scroll_area.visibility);
    let size = props.scrollbar_size.copied_or(theme.scroll_area.size);
    let own_bars = visibility == ScrollbarVisibility::Always
        && scrollbars != ScrollAxis::None
        && draws_own_scrollbars();
    let drawn_bars = DrawnBars {
        root,
        layer: use_element(),
        metrics: use_signal(|| None::<ScrollMetrics>),
        latest: use_hook(|| CopyValue::new(None)),
    };
    let measure_bars_now = move |own_bars: bool| {
        if own_bars && root.is_mounted() && drawn_bars.layer.is_mounted() {
            drawn_bars.measure(1);
        }
    };

    // A caller's own `tabindex`, or a role like `listbox`, brings its own
    // keyboard model: no automatic stop then.
    let owned = props
        .attributes
        .iter()
        .any(|attribute| match attribute.name {
            "tabindex" => true,
            "role" => !matches!(&attribute.value, AttributeValue::Text(role) if role == "region"),
            _ => false,
        });
    let auto_stop = use_signal(|| false);
    let automatic = !props.focusable && !owned;
    // A WebView finds the root by its tag: a caller's spread one wins, never a second.
    let spread_tag = props
        .attributes
        .iter()
        .find(|attribute| attribute.name == OBSERVE_ATTR)
        .map(|attribute| match &attribute.value {
            AttributeValue::Text(tag) => tag.parse().ok(),
            _ => None,
        });
    let tag = spread_tag.unwrap_or(root.tag());
    let check_stop = move || {
        if automatic {
            check_tab_stop(root, tag, scrollbars, auto_stop, UNLAID_TRIES);
        }
    };
    let content = use_element();
    // Only while the area picks its own stop: a change inside a child component
    // re-renders nothing here (todo 681).
    let changes = use_content_changes(content, automatic || own_bars);
    // After the DOM has the content: on mount (the effect reads the mount), on
    // new content, on a change deeper down, and on resize (`onresize`).
    let children = props.children.clone();
    let mut seen = use_hook(|| CopyValue::new(None::<(Option<usize>, u64, bool)>));
    use_effect(use_reactive!(|children, own_bars| {
        let _ = &children;
        let now = (root.mount_token(), changes.count.cloned(), own_bars);
        // Only new children: a watched subtree that changed bumps `count` anyway, so
        // a parent's redraw (a theme switch) forces no layout (todo 2094).
        let redraw_only = seen.replace(Some(now)) == Some(now) && changes.watching();
        if now.0.is_some() && !redraw_only {
            check_stop();
            measure_bars_now(own_bars);
        }
    }));
    let tab_stop = props.focusable || (automatic && auto_stop());
    let mut warned = use_hook(|| CopyValue::new(false));
    if tab_stop && !warned() && !names_itself(&props.attributes) {
        warned.set(true);
        warn(
            "ScrollArea: a tab stop with no `aria-label` or `aria-labelledby`, so the \
             region has no name.",
        );
    }

    let mut is_scrolling = use_signal(|| false);
    let edges = use_signal(|| EdgeState::AT_ORIGIN);

    let mut geometry = use_signal(|| None::<ScrollGeometry>);
    let spec = use_signal(|| None::<WindowSpec>);
    let virtualized = use_signal(|| false);
    use_context_provider(|| ScrollViewport::new(content, geometry, spec, virtualized));
    // A `Virtualize` child needs the height before any scroll and on every resize.
    let measure = move || {
        if root.is_mounted() {
            measure_area(root, geometry, UNLAID_TRIES);
        }
    };
    let watches_edges = [
        props.ontopreached,
        props.onbottomreached,
        props.onleftreached,
        props.onrightreached,
    ]
    .iter()
    .any(Option::is_some);
    // Read by the platform scroll report, outside any render.
    let mut reports_edges = use_hook(|| CopyValue::new(false));
    reports_edges.set(watches_edges);
    // Blitz's `scroll_to` and scroll-into-view fire no `scroll` event: its
    // platform scroll report stands in for them.
    let platform_scrolls = use_signal(|| 0u64);
    use_hook(|| {
        let api = scroll().filter(|_| !fires_scroll_on_scroll_to());
        Rc::new(api.map(|api| {
            api.on_scroll(Box::new(move || {
                if *virtualized.peek() || *reports_edges.peek() {
                    crate::utils::bump(platform_scrolls);
                }
            }))
        }))
    });
    let onscroll = props.onscroll;
    let onscroll_prop = onscroll.is_some();
    let scrolled = move |event: ScrollPositionEvent| {
        if let Some(onscroll) = &onscroll {
            onscroll.call(event);
        }
    };
    let reached = |handler: Option<EventHandler<()>>| {
        if let Some(handler) = handler {
            handler.call(());
        }
    };
    let ontopreached = props.ontopreached;
    let onbottomreached = props.onbottomreached;
    let onleftreached = props.onleftreached;
    let onrightreached = props.onrightreached;

    // One check, and `edges` its one latch, for `scroll` events and the platform
    // report alike: whichever comes second finds no new edge. `x` from the inline start.
    let check_edges = move |x: f64, y: f64, max_x: f64, max_y: f64| {
        let mut edges = edges;
        let new_edges = EdgeState::at(x, y, max_x, max_y);
        let now = new_edges.reached_since(*edges.peek());
        if now.top {
            reached(ontopreached);
        }
        if now.bottom {
            reached(onbottomreached);
        }
        let (start, end) = (now.start, now.end);
        if start || end {
            // Left and right stay physical: the start is the right under RTL.
            let (onstart, onend) = match root.is_rtl() {
                true => (onrightreached, onleftreached),
                false => (onleftreached, onrightreached),
            };
            if start {
                reached(onstart);
            }
            if end {
                reached(onend);
            }
        }
        edges.set(new_edges);
    };

    // Re-runs once the root is mounted, and once a `Virtualize` asks: only then
    // do scroll and resize events keep the geometry current.
    let mut seen_reports = use_hook(|| CopyValue::new(0u64));
    use_effect(move || {
        let reports = platform_scrolls();
        let scrolled = reports != *seen_reports.peek();
        seen_reports.set(reports);
        if virtualized() {
            match scrolled && geometry.peek().is_some() && root.is_mounted() {
                true => measure_offset(root, geometry),
                false => measure(),
            }
        }
        if reports > 0 && *reports_edges.peek() && root.is_mounted() {
            measure_edges(root, check_edges);
        }
    });

    let mut ended = move |(x_pct, y_pct): (f64, f64)| {
        if is_scrolling() {
            is_scrolling.set(false);
            scrolled(ScrollPositionEvent::End(x_pct, y_pct));
        }
    };
    // No `scrollend` natively: a quiet spell after the last scroll ends it.
    let mut quiet = use_signal(|| 0u64);
    let mut quiet_wait = use_hook(|| CopyValue::new(None::<Box<dyn TimerSubscription>>));
    let mut last_at = use_hook(|| CopyValue::new((0.0, 0.0)));
    use_effect(move || {
        if quiet() > 0 {
            // A task: what `onscroll` reads must not subscribe this effect.
            spawn(async move { ended(*last_at.peek()) });
        }
    });

    let mut scroll_data = move |data: &ScrollData| {
        if own_bars {
            drawn_bars.scrolled(data);
        }
        let (x_pct, y_pct, max_x, max_y) = scroll_metrics(data);
        let known = geometry.peek().unwrap_or_default();
        geometry.set(Some(
            known.scrolled(data.scroll_top(), data.client_height() as f64),
        ));
        if onscroll_prop && !fires_scroll_end() {
            last_at.set((x_pct, y_pct)); // Replacing the wait drops the one before, which cancels it.
            quiet_wait.set(timer().map(|timer| {
                timer.after(
                    SCROLL_QUIET,
                    Box::new(move || {
                        let next = *quiet.peek() + 1;
                        quiet.set(next);
                    }),
                )
            }));
        }

        if is_scrolling() {
            scrolled(ScrollPositionEvent::Change(x_pct, y_pct));
        } else {
            is_scrolling.set(true);
            scrolled(ScrollPositionEvent::Start(x_pct, y_pct));
        }

        check_edges(
            inline_x(data.scroll_left()),
            data.scroll_top(),
            max_x,
            max_y,
        );
    };
    let onscroll = move |event: Event<ScrollData>| scroll_data(&event.data());
    // The latest render's handler, for the WebView's report below.
    let mut latest = use_hook(|| CopyValue::new(None::<ScrollHandler>));
    // On a WebView a `scroll` listener held the page up a round trip per step,
    // most of a fast fling's frames (todo 2131): it reports by message instead.
    let reported = use_hook(|| {
        Rc::new(tag.and_then(|tag| {
            on_element_scroll(
                tag,
                Box::new(move |data, end| {
                    let mut latest = latest;
                    if let Some(handler) = latest.write().as_mut() {
                        handler(&data, end);
                    }
                }),
            )
        }))
    })
    .is_some();

    let onscrollend = move |event: Event<ScrollData>| {
        let (x_pct, y_pct, ..) = scroll_metrics(&event.data());
        ended((x_pct, y_pct));
    };

    let onresize = props.onresize;
    let scroll_position_x = props.scroll_position_x;
    let scroll_position_y = props.scroll_position_y;
    use_effect(use_reactive!(|scroll_position_x, scroll_position_y| {
        scroll_to_percent(root, scroll_position_x, scroll_position_y);
    }));

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(scrollbars.state_name(), true)
        .with(visibility_token(visibility), true)
        .with(size.state_name(), true)
        .into();

    let variables: Input<Variables> = scroll_area_variables(props.scrollbar_color.as_ref()).into();

    // ARIA prohibits naming a generic: the name waits for a role.
    let has_role = tab_stop
        || props
            .attributes
            .iter()
            .any(|attribute| attribute.name == "role");
    let mut attributes = props.attributes;
    if !has_role {
        attributes.retain(|attribute| !matches!(attribute.name, "aria-label" | "aria-labelledby"));
    }
    if (automatic || reported) && spread_tag.is_none() {
        attributes.extend(root.attributes());
    }

    let virtualized = virtualized();
    let body = rsx! {
        if own_bars {
            ScrollAreaBars { state: drawn_bars, scrollbars, size, inset_top: props.bar_inset_top.unwrap_or(0.0) }
        }
        ScrollAreaContent { content, {props.children} }
    };

    // A listener costs a render and, for `onresize`, an observer: attach each
    // only while something reads it.
    let tracks_scroll = virtualized
        || own_bars
        || onscroll_prop
        || [ontopreached, onbottomreached, onleftreached, onrightreached]
            .iter()
            .any(Option::is_some);
    // `scrollend` comes by the same message: as its own event it would beat the last step.
    let handler: Option<ScrollHandler> = match tracks_scroll && reported {
        true => Some(Box::new(move |data: &ScrollData, end: bool| {
            scroll_data(data);
            if end && onscroll_prop {
                let (x_pct, y_pct, ..) = scroll_metrics(data);
                ended((x_pct, y_pct));
            }
        })),
        false => None,
    };
    latest.set(handler);

    // `ResizeObserver` reports once on observe: the mount-time check.
    let resized = move |event: Event<ResizeData>| {
        check_stop();
        measure_bars_now(own_bars);
        if virtualized || onresize.is_some() {
            measure();
        }
        if let Some(onresize) = &onresize {
            onresize.call(event);
        }
    };
    use_resize_fallback(root, resized);

    use_box()
        .framework_sx(
            props
                .framework_sx
                .map_or(&SCROLL_AREA_BASE_SX, |base| base.0),
        )
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&variables)
        .prepare()
        .element(&root)
        // `-1` opts out of Chromium's implicit stop, so every browser gets the
        // same one: this, named as a region (APG scrollable region).
        .attr_default("tabindex", if tab_stop { "0" } else { "-1" })
        .attr_default("role", tab_stop.then_some("region"))
        .event("onscroll", (tracks_scroll && !reported).then_some(onscroll))
        .event(
            "onscrollend",
            (onscroll_prop && !reported).then_some(onscrollend),
        )
        .event(
            "onkeydown",
            (!owned && !scrolls_on_keys()).then_some(move |event| scroll_on_key(root, event)),
        )
        .event("onresize", resized)
        .render(HtmlTag::Div, attributes, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::common::part_table,
        tokens::{Color, ColorShade, ColorValue, Size},
    };

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<ScrollAreaPart>(),
            [
                (
                    "scrollbar",
                    "& > [data-slot='scrollbars'] > * > [data-slot='scrollbar']"
                ),
                (
                    "thumb",
                    "& > [data-slot='scrollbars'] > * > [data-slot='scrollbar'] > [data-slot='thumb']"
                ),
            ]
        );
    }

    #[test]
    fn a_thumb_color_resolves_with_no_scale() {
        let color = ThemeAwareValue::Color(Color::Primary);
        let variables = scroll_area_variables(Some(&color));

        assert_eq!(
            variables.to_string(),
            format!(
                "{}:{};",
                SCROLL_AREA_THUMB_VAR.name(),
                ColorValue::Shade(Color::Primary, ColorShade::DEFAULT).value()
            )
        );
    }

    #[test]
    fn no_thumb_color_emits_no_variable() {
        assert_eq!(scroll_area_variables(None).to_string(), "");
    }

    /// A bare `Size` has no scale here, so it drops out.
    #[test]
    fn an_unresolvable_value_emits_no_variable() {
        let size = ThemeAwareValue::Size(Size::Md);

        assert_eq!(scroll_area_variables(Some(&size)).to_string(), "");
    }

    fn dims(width: f64, height: f64) -> Dimensions {
        Dimensions { width, height }
    }

    #[test]
    fn only_overflow_on_a_scrolled_axis_with_nothing_focusable_is_a_tab_stop() {
        let (view, tall, wide) = (dims(100.0, 100.0), dims(100.0, 300.0), dims(300.0, 100.0));

        assert!(needs_tab_stop(ScrollAxis::Vertical, view, tall, false));
        assert!(!needs_tab_stop(ScrollAxis::Vertical, view, tall, true));
        assert!(!needs_tab_stop(ScrollAxis::Vertical, view, wide, false));
        assert!(needs_tab_stop(ScrollAxis::Horizontal, view, wide, false));
        assert!(needs_tab_stop(ScrollAxis::Both, view, wide, false));
        assert!(!needs_tab_stop(ScrollAxis::None, view, tall, false));
    }

    /// A fractional border box against integer content is not overflow.
    #[test]
    fn a_pixel_of_rounding_is_not_overflow() {
        let view = dims(100.0, 100.4);

        assert!(!needs_tab_stop(
            ScrollAxis::Vertical,
            view,
            dims(100.0, 101.0),
            false
        ));
    }

    const NATIVE_RANGE: i32 = cfg!(all(not(target_arch = "wasm32"), feature = "native")) as i32;

    struct FakeScroll {
        top: f64,
        left: f64,
        scroll: (i32, i32),
        client: (i32, i32),
    }

    impl dioxus::html::HasScrollData for FakeScroll {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn scroll_top(&self) -> f64 {
            self.top
        }
        fn scroll_left(&self) -> f64 {
            self.left
        }
        // Blitz reports the range as `scroll_width`/`scroll_height`, the web the content's size.
        fn scroll_width(&self) -> i32 {
            self.scroll.0 - NATIVE_RANGE * self.client.0
        }
        fn scroll_height(&self) -> i32 {
            self.scroll.1 - NATIVE_RANGE * self.client.1
        }
        fn client_width(&self) -> i32 {
            self.client.0
        }
        fn client_height(&self) -> i32 {
            self.client.1
        }
    }

    #[test]
    fn metrics_are_a_percent_of_the_scrollable_range() {
        let data = ScrollData::new(FakeScroll {
            top: 100.0,
            left: 50.0,
            scroll: (300, 500),
            client: (100, 300),
        });

        assert_eq!(scroll_metrics(&data), (25.0, 50.0, 200.0, 200.0));
    }

    /// Under RTL `scrollLeft` runs negative; the percent still counts from the start.
    #[test]
    fn an_rtl_offset_is_a_percent_from_the_start() {
        let data = ScrollData::new(FakeScroll {
            top: 0.0,
            left: -50.0,
            scroll: (300, 100),
            client: (100, 100),
        });

        assert_eq!(scroll_metrics(&data).0, 25.0);
    }

    #[test]
    fn edges_count_from_the_inline_start() {
        let at = |x| EdgeState::at(x, 0.0, 200.0, 0.0);

        assert!(at(0.0).start);
        assert!(!at(0.0).end);
        assert!(at(199.5).end);
        assert!(!at(100.0).start);
        assert!(!at(100.0).end);
    }

    /// Rows appended at the bottom: End to the new bottom reports it once more.
    #[test]
    fn a_grown_bottom_is_reached_again() {
        let at = |y, max_y| EdgeState::at(0.0, y, 0.0, max_y);
        let bottom = at(500.0, 500.0);
        assert!(bottom.reached_since(at(400.0, 500.0)).bottom);

        // Appended: still at the old offset, then End jumps to the new bottom.
        let grown = at(1000.0, 1000.0);
        assert!(grown.reached_since(bottom).bottom);
        assert!(
            !grown.reached_since(grown).bottom,
            "a repeat at the same bottom"
        );

        let shrunk = at(300.0, 300.0);
        assert!(!shrunk.reached_since(bottom).bottom, "content that shrank");
        assert!(!grown.reached_since(bottom).top);
    }

    /// Content that fits scrolls nowhere - the percent would divide by zero.
    #[test]
    fn an_unscrollable_axis_reports_zero() {
        let data = ScrollData::new(FakeScroll {
            top: 0.0,
            left: 0.0,
            scroll: (100, 100),
            client: (100, 100),
        });

        assert_eq!(scroll_metrics(&data), (0.0, 0.0, 0.0, 0.0));
    }
}
