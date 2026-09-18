use dioxus::{dioxus_core::AttributeValue, prelude::*};

use super::{
    handle::{ScrollAreaHandle, inline_x, scroll_to_percent},
    viewport::{ContentOffsets, ScrollGeometry, ScrollViewport},
};
use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{FOCUSABLE_SELECTOR, base_props, input_from_str, names_itself, variables},
        layout::use_box,
    },
    hooks::{ElementHandle, use_content_changes, use_element, use_resize_fallback, use_theme},
    platform::{Dimensions, ElementApi, PlatformError, when_laid_out},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, CssVar, ScrollAxis, ScrollbarSize, ScrollbarVisibility},
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
/// Rows a `Virtualize` child skipped, standing in as padding so the scroll
/// range still spans the whole list.
const SCROLL_AREA_LEADING_VAR: CssVar = CssVar::new("--lsx-scroll-area-leading");
const SCROLL_AREA_TRAILING_VAR: CssVar = CssVar::new("--lsx-scroll-area-trailing");

/// `Scroll` behaves as `Hover`: nothing here can yet fade the scrollbar out
/// after an idle timeout. See `ScrollbarVisibility`.
fn visibility_token(visibility: ScrollbarVisibility) -> &'static str {
    match visibility {
        ScrollbarVisibility::Scroll => ScrollbarVisibility::Hover.state_name(),
        visibility => visibility.state_name(),
    }
}

/// The base styles of a component built on `ScrollArea`, standing in for
/// `ScrollArea`'s own on the framework layer. Build it from
/// [`scroll_area_base`], so it keeps everything the area needs.
///
/// Crate-internal: the field is `pub(crate)` and the type is not exported, so
/// nothing outside the crate can make one. It exists so the component's CSS
/// stays below a caller's `sx`, which passing it as `sx` would not.
#[doc(hidden)]
#[allow(unnameable_types)]
#[derive(Clone, Copy, PartialEq)]
pub struct ScrollAreaBase(pub(crate) &'static StaticSx);

/// `ScrollArea`'s own base styles with `sx` merged on top: a property `sx`
/// declares again replaces the area's.
pub(crate) fn scroll_area_base(sx: crate::sx::Sx) -> crate::sx::Sx {
    SCROLL_AREA_BASE_SX.clone().and(sx)
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
        // Shade 6: shade 5 was 2.07:1 on the light page, under WCAG 1.4.11's 3:1.
        .scrollbar_color(format!(
            "{} transparent",
            SCROLL_AREA_THUMB_VAR.value_or(ColorCss::MUTED.value(ColorShade::S6))
        ))
        .when("visible-hidden", sx().scrollbar_width("none"));

    ScrollbarSize::ALL.iter().fold(base, |acc, &size| {
        let token = size.state_name();
        let width = size.as_str();
        acc.when(
            format!("visible-always && {token}"),
            sx().scrollbar_width(width),
        )
        .when(
            format!("visible-hover && {token}"),
            sx().scrollbar_width("none")
                .hover(sx().scrollbar_width(width))
                .selector(":focus-within", sx().scrollbar_width(width)),
        )
    })
});

/// Wraps the content so the rows a `Virtualize` skipped have something to be
/// reserved on. `display: contents` until then, so an ordinary scroll area
/// lays out exactly as it did without it.
static SCROLL_AREA_CONTENT_SX: StaticSx = StaticSx::new(|| {
    sx().display("contents").when(
        "virtualized",
        sx().display("block")
            .padding_top(SCROLL_AREA_LEADING_VAR.value_or("0px"))
            .padding_bottom(SCROLL_AREA_TRAILING_VAR.value_or("0px")),
    )
});

fn scroll_area_variables(color: Option<&ThemeAwareValue>) -> Variables {
    variables().with(SCROLL_AREA_THUMB_VAR, color.and_then(|v| v.resolve(None)))
}

/// Both are always written, `0px` included. Dropping one leaves the previous
/// declaration in place - the style attribute is patched property by property,
/// not replaced - and the last window's reserve outlives the window.
fn scroll_area_content_variables(offsets: ContentOffsets) -> Variables {
    variables()
        .with(SCROLL_AREA_LEADING_VAR, format!("{}px", offsets.leading))
        .with(SCROLL_AREA_TRAILING_VAR, format!("{}px", offsets.trailing))
}

/// Which edges the last scroll position rested against, so `on*reached`
/// fires on the rising edge rather than every event. Inline start and end,
/// so the origin is the same under RTL.
#[derive(Clone, Copy, Debug, PartialEq)]
struct EdgeState {
    top: bool,
    bottom: bool,
    start: bool,
    end: bool,
}

impl EdgeState {
    /// Where a scroll container starts. All-`false` would make the first
    /// scroll event report leaving-and-reaching the top and start edges it
    /// was already resting against.
    const AT_ORIGIN: Self = Self {
        top: true,
        bottom: false,
        start: true,
        end: false,
    };

    /// From one scroll position, the x offset counted from the inline start.
    fn at(x: f64, y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            top: y <= 0.0,
            bottom: max_y <= 0.0 || y >= max_y - 1.0,
            start: x <= 0.0,
            end: max_x <= 0.0 || x >= max_x - 1.0,
        }
    }
}

base_props! {
    pub struct ScrollAreaProps {
        /// `"vertical"` (default), `"horizontal"`, `"both"` or `"none"`.
        #[props(default, into)]
        scrollbars: Input<ScrollAxis>,
        /// `"always"` (default), `"hover"`, `"hidden"`, or `"scroll"`
        /// (currently identical to `"hover"`).
        #[props(default, into)]
        scrollbar_visibility: Input<ScrollbarVisibility>,
        /// CSS `scrollbar-width`: `"thin"` (default) or `"auto"`.
        #[props(default, into)]
        scrollbar_size: Input<ScrollbarSize>,
        /// Scrollbar thumb color - track stays transparent.
        #[props(default, into)]
        scrollbar_color: Input<ThemeAwareValue>,
        /// Percent (0-100) to scroll to. Bound to a signal it re-applies on
        /// every change; a literal applies once, at mount.
        scroll_position_x: Option<f64>,
        /// Percent (0-100) along the vertical axis - see `scroll_position_x`.
        scroll_position_y: Option<f64>,
        /// Makes the viewport a tab stop always. Without it the area is one
        /// only while it overflows and holds nothing focusable, so plain
        /// content can still be scrolled with the arrow keys.
        ///
        /// A tab stop gets `role="region"` and needs a name: pass
        /// `aria-label` or `aria-labelledby` (a debug build warns without).
        #[props(default)]
        focusable: bool,
        /// From [`use_scroll_area`](super::use_scroll_area), to scroll the
        /// area from an event handler. Unlike `scroll_position_x`/`_y`, each
        /// call scrolls, including one that asks for the same position again.
        #[props(default)]
        handle: Option<ScrollAreaHandle>,
        /// Crate-internal, see [`ScrollAreaBase`].
        #[doc(hidden)]
        #[props(default)]
        framework_sx: Option<ScrollAreaBase>,
        #[props(default)]
        onscroll: Option<EventHandler<ScrollPositionEvent>>,
        /// After the area resized, once it has re-measured itself. A prop, not
        /// a spread attribute: a spread `onresize` would replace the area's own.
        ///
        /// ```no_run
        /// # use dioxus::prelude::*;
        /// # use libero::components::ScrollArea;
        /// # fn app() -> Element {
        /// # let mut resized = use_signal(|| 0);
        /// # rsx! {
        /// ScrollArea { onresize: move |_: Event<ResizeData>| resized += 1, "Rows" }
        /// # } }
        /// ```
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
    let max_x = (data.scroll_width() - data.client_width()).max(0) as f64;
    let max_y = (data.scroll_height() - data.client_height()).max(0) as f64;
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
const UNLAID_TRIES: u8 = 3;

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
                },
                _ => ScrollGeometry::default(),
            };
            if *geometry.peek() != Some(measured) {
                geometry.set(Some(measured));
            }
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

/// Re-reads [`needs_tab_stop`] once laid out. A renderer that cannot query the
/// subtree leaves the area as it was.
fn check_tab_stop(root: ElementHandle, axis: ScrollAxis, mut stop: Signal<bool>, tries: u8) {
    when_laid_out(move || {
        let inner = match root.query_selector(FOCUSABLE_SELECTOR) {
            Ok(_) => true,
            Err(PlatformError::NotFound) => false,
            Err(_) => return,
        };
        let (view, content) = (root.dimensions(), root.scroll_size());
        spawn(async move {
            if let (Ok(view), Ok(content)) = (view.await, content.await) {
                if view.height <= 0.0 && tries > 0 {
                    return check_tab_stop(root, axis, stop, tries - 1);
                }
                let next = needs_tab_stop(axis, view, content, inner);
                if *stop.peek() != next {
                    stop.set(next);
                }
            }
        });
    });
}

/// Scrolls its content, filling the parent by default. Read the scroll
/// position via `onscroll`/`on*reached`; set it imperatively via
/// `scroll_position_x`/`scroll_position_y` (reactive if bound to a signal,
/// initial-only if a literal). A caller's `aria-label`/`aria-labelledby`
/// applies only while the area has a role: its own region stop, or yours.
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    let theme = use_theme();
    // Always called, so the hook order does not depend on the prop. A caller's
    // handle takes over the element when there is one; the area's own is
    // then simply never mounted.
    let own = use_element();
    let root = props.handle.map_or(own, |handle| handle.element);

    let scrollbars = props.scrollbars.copied_or(theme.scroll_area.scrollbars);
    let visibility = props
        .scrollbar_visibility
        .copied_or(theme.scroll_area.visibility);
    let size = props.scrollbar_size.copied_or(theme.scroll_area.size);

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
    let check_stop = move || {
        if automatic {
            check_tab_stop(root, scrollbars, auto_stop, UNLAID_TRIES);
        }
    };
    let content = use_element();
    // Only while the area picks its own stop: a change inside a child component
    // re-renders nothing here (todo 681).
    let changes = use_content_changes(content, automatic);
    // After the DOM has the content: on mount (the effect reads the mount), on
    // new content, on a change deeper down, and on resize (`onresize`).
    let children = props.children.clone();
    use_effect(use_reactive!(|children| {
        let _ = (&children, changes());
        if root.is_mounted() {
            check_stop();
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
    let mut edges = use_signal(|| EdgeState::AT_ORIGIN);

    let mut geometry = use_signal(|| None::<ScrollGeometry>);
    let offsets = use_signal(ContentOffsets::default);
    let virtualized = use_signal(|| false);
    use_context_provider(|| ScrollViewport::new(content, geometry, offsets, virtualized));
    // A `Virtualize` child needs the viewport height before anything has been
    // scrolled, and again whenever a pane around it resizes.
    let measure = move || {
        if root.is_mounted() {
            measure_area(root, geometry, UNLAID_TRIES);
        }
    };
    // Re-runs once the root is mounted, and once a `Virtualize` asks: only then
    // do scroll and resize events keep the geometry current.
    use_effect(move || {
        if virtualized() {
            measure();
        }
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

    let onscroll = move |event: Event<ScrollData>| {
        let data = event.data();
        let (x_pct, y_pct, max_x, max_y) = scroll_metrics(&data);
        geometry.set(Some(ScrollGeometry {
            offset: data.scroll_top(),
            viewport: data.client_height() as f64,
        }));

        if is_scrolling() {
            scrolled(ScrollPositionEvent::Change(x_pct, y_pct));
        } else {
            is_scrolling.set(true);
            scrolled(ScrollPositionEvent::Start(x_pct, y_pct));
        }

        let new_edges = EdgeState::at(
            inline_x(data.scroll_left()),
            data.scroll_top(),
            max_x,
            max_y,
        );
        let previous = edges();
        if new_edges.top && !previous.top {
            reached(ontopreached);
        }
        if new_edges.bottom && !previous.bottom {
            reached(onbottomreached);
        }
        let (start, end) = (
            new_edges.start && !previous.start,
            new_edges.end && !previous.end,
        );
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

    let onscrollend = move |event: Event<ScrollData>| {
        if is_scrolling() {
            is_scrolling.set(false);
            let (x_pct, y_pct, ..) = scroll_metrics(&event.data());
            scrolled(ScrollPositionEvent::End(x_pct, y_pct));
        }
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

    // Unvirtualized content is `display: contents` and reserves nothing.
    let virtualized = virtualized();
    let (content_states, content_variables): (Input<States>, Input<Variables>) = if virtualized {
        (
            States::default().with("virtualized", true).into(),
            scroll_area_content_variables(offsets()).into(),
        )
    } else {
        (Input::None, Input::None)
    };
    let body = use_box()
        .framework_sx(&SCROLL_AREA_CONTENT_SX)
        .states(&content_states)
        .variables(&content_variables)
        .prepare()
        .element(&content)
        .render(HtmlTag::Div, Vec::new(), props.children);

    // A listener costs a render and, for `onresize`, an observer: attach each
    // only while something reads it.
    let tracks_scroll = virtualized
        || onscroll_prop
        || [ontopreached, onbottomreached, onleftreached, onrightreached]
            .iter()
            .any(Option::is_some);

    // `ResizeObserver` reports once on observe: the mount-time check.
    let resized = move |event: Event<ResizeData>| {
        check_stop();
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
        .states(&states)
        .variables(&variables)
        .prepare()
        .element(&root)
        // `-1` opts out of Chromium's implicit stop, so every browser gets the
        // same one: this, named as a region (APG scrollable region).
        .attr_default("tabindex", if tab_stop { "0" } else { "-1" })
        .attr_default("role", tab_stop.then_some("region"))
        .event("onscroll", tracks_scroll.then_some(onscroll))
        .event("onscrollend", onscroll_prop.then_some(onscrollend))
        .event("onresize", resized)
        .render(HtmlTag::Div, attributes, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Color, ColorShade, ColorValue, Size};

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
        fn scroll_width(&self) -> i32 {
            self.scroll.0
        }
        fn scroll_height(&self) -> i32 {
            self.scroll.1
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

    /// Under RTL `scrollLeft` runs negative; the percent still counts from the
    /// start.
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

        assert!(at(0.0).start && !at(0.0).end);
        assert!(at(199.5).end);
        assert!(!at(100.0).start && !at(100.0).end);
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
