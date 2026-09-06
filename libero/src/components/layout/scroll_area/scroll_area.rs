use dioxus::prelude::*;

use super::{
    handle::{ScrollAreaHandle, scroll_to_percent},
    viewport::{ContentOffsets, ScrollGeometry, ScrollViewport},
};
use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, input_from_str, variables},
        layout::use_box,
    },
    hooks::{use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, CssVar, ScrollAxis, ScrollbarSize, ScrollbarVisibility},
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
        .scrollbar_color(format!(
            "{} transparent",
            SCROLL_AREA_THUMB_VAR.value_or(ColorCss::GREY.value(ColorShade::S5))
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
/// fires on the rising edge rather than every event.
#[derive(Clone, Copy, Debug, PartialEq)]
struct EdgeState {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
}

impl EdgeState {
    /// Where a scroll container starts. All-`false` would make the first
    /// scroll event report leaving-and-reaching the top and left edges it was
    /// already resting against.
    const AT_ORIGIN: Self = Self {
        top: true,
        bottom: false,
        left: true,
        right: false,
    };
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
        /// Makes the viewport itself a tab stop, so content that carries no
        /// focusable elements of its own can still be reached and scrolled
        /// with the arrow keys. Off by default, and only correct when the
        /// content is genuinely not reachable otherwise - a tab stop that
        /// does nothing is worse than none.
        ///
        /// APG's scrollable-region pattern also wants a `role="region"` (or
        /// `"group"`) and an accessible name. `ScrollArea` sets no `role` of
        /// its own, so both go through `attributes` as usual.
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
        percent(data.scroll_left(), max_x),
        percent(data.scroll_top(), max_y),
        max_x,
        max_y,
    )
}

/// Scrolls its content, filling the parent by default. Read the scroll
/// position via `onscroll`/`on*reached`; set it imperatively via
/// `scroll_position_x`/`scroll_position_y` (reactive if bound to a signal,
/// initial-only if a literal).
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

    let mut is_scrolling = use_signal(|| false);
    let mut edges = use_signal(|| EdgeState::AT_ORIGIN);

    let content = use_element();
    let mut geometry = use_signal(|| None::<ScrollGeometry>);
    let offsets = use_signal(ContentOffsets::default);
    let virtualized = use_signal(|| false);
    use_context_provider(|| ScrollViewport::new(content, geometry, offsets, virtualized));
    // A `Virtualize` child needs the viewport height before anything has been
    // scrolled, and re-runs once the root is actually mounted.
    use_effect(move || {
        if !root.is_mounted() {
            return;
        }
        let (size, offset) = (root.dimensions(), root.scroll_offset());
        spawn(async move {
            let measured = match (size.await, offset.await) {
                (Ok(size), Ok((_, top))) => ScrollGeometry {
                    offset: top,
                    viewport: size.height,
                },
                _ => ScrollGeometry::default(),
            };
            geometry.set(Some(measured));
        });
    });

    let onscroll = props.onscroll;
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

        let new_edges = EdgeState {
            top: data.scroll_top() <= 0.0,
            bottom: max_y <= 0.0 || data.scroll_top() >= max_y - 1.0,
            left: data.scroll_left() <= 0.0,
            right: max_x <= 0.0 || data.scroll_left() >= max_x - 1.0,
        };
        let previous = edges();
        if new_edges.top && !previous.top {
            reached(ontopreached);
        }
        if new_edges.bottom && !previous.bottom {
            reached(onbottomreached);
        }
        if new_edges.left && !previous.left {
            reached(onleftreached);
        }
        if new_edges.right && !previous.right {
            reached(onrightreached);
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

    let content_states: Input<States> = States::default().with("virtualized", virtualized()).into();
    let content_variables: Input<Variables> = scroll_area_content_variables(offsets()).into();
    let body = use_box()
        .framework_sx(&SCROLL_AREA_CONTENT_SX)
        .states(&content_states)
        .variables(&content_variables)
        .prepare()
        .element(&content)
        .render(HtmlTag::Div, Vec::new(), props.children);

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
        // Chromium makes an overflowing `overflow: auto` region an implicit
        // tab stop unless opted out, and the usual case is content that
        // carries its own focusable elements. `focusable` is the other case:
        // a strip of plain images or text, where the viewport is the only
        // thing there is to focus.
        .attr("tabindex", if props.focusable { "0" } else { "-1" })
        .event("onscroll", onscroll)
        .event("onscrollend", onscrollend)
        .render(HtmlTag::Div, props.attributes, body)
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
