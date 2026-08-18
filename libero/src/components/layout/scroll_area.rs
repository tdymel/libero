use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States, Variables,
        common::{base_props, dom_api, variables},
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ScrollAxis, ScrollbarSize, ScrollbarVisibility},
};

impl From<&str> for Input<ScrollAxis> {
    fn from(value: &str) -> Self {
        Input::Value(ScrollAxis::from(value))
    }
}

impl From<String> for Input<ScrollAxis> {
    fn from(value: String) -> Self {
        Input::Value(ScrollAxis::from(value))
    }
}

impl From<&str> for Input<ScrollbarVisibility> {
    fn from(value: &str) -> Self {
        Input::Value(ScrollbarVisibility::from(value))
    }
}

impl From<String> for Input<ScrollbarVisibility> {
    fn from(value: String) -> Self {
        Input::Value(ScrollbarVisibility::from(value))
    }
}

impl From<&str> for Input<ScrollbarSize> {
    fn from(value: &str) -> Self {
        Input::Value(ScrollbarSize::from(value))
    }
}

impl From<String> for Input<ScrollbarSize> {
    fn from(value: String) -> Self {
        Input::Value(ScrollbarSize::from(value))
    }
}

/// Scroll position as a percent (0-100) of each axis's scrollable range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollPositionEvent {
    Start(f64, f64),
    Change(f64, f64),
    End(f64, f64),
}

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

const SCROLL_AREA_THUMB_VAR: &str = "--lsx-scroll-area-thumb-color";

/// Resolves `Scroll` to the same behavior as `Hover` - no idle-timeout
/// primitive exists in this codebase yet to fade the scrollbar out after a
/// period of inactivity, see `ScrollbarVisibility` doc.
fn visibility_token(visibility: ScrollbarVisibility) -> &'static str {
    match visibility {
        ScrollbarVisibility::Always => "visible-always",
        ScrollbarVisibility::Hover | ScrollbarVisibility::Scroll => "visible-hover",
        ScrollbarVisibility::Hidden => "visible-hidden",
    }
}

fn size_token(size: ScrollbarSize) -> &'static str {
    match size {
        ScrollbarSize::Thin => "size-thin",
        ScrollbarSize::Auto => "size-auto",
    }
}

static SCROLL_AREA_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .display("block")
        .width("100%")
        .height("100%")
        .when("axis-vertical", sx().overflow_y("auto").overflow_x("hidden"))
        .when("axis-horizontal", sx().overflow_x("auto").overflow_y("hidden"))
        .when("axis-both", sx().overflow_x("auto").overflow_y("auto"))
        .when("axis-none", sx().overflow_x("hidden").overflow_y("hidden"))
        .scrollbar_color(format!(
            "var({SCROLL_AREA_THUMB_VAR}, var(--lsx-grey-4)) transparent"
        ))
        .when("visible-hidden", sx().scrollbar_width("none"));

    [ScrollbarSize::Thin, ScrollbarSize::Auto]
        .into_iter()
        .fold(base, |acc, size| {
            let token = size_token(size);
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

fn scroll_area_variables(color: Option<&ThemeAwareValue>) -> Variables {
    variables().with(SCROLL_AREA_THUMB_VAR, color.and_then(ThemeAwareValue::resolved))
}

fn axis_token(scrollbars: ScrollAxis) -> &'static str {
    match scrollbars {
        ScrollAxis::Vertical => "axis-vertical",
        ScrollAxis::Horizontal => "axis-horizontal",
        ScrollAxis::Both => "axis-both",
        ScrollAxis::None => "axis-none",
    }
}

/// Which edges (per axis) the last-seen scroll position was resting against
/// - only used to fire `on_*_reached` on the rising edge, not on every
/// scroll event while still resting there.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct EdgeState {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
}

base_props! {
    pub struct ScrollAreaProps {
        /// Which axes scroll - `"vertical"` (default), `"horizontal"`,
        /// `"both"`, or `"none"`.
        #[props(default, into)]
        scrollbars: Input<ScrollAxis>,
        /// When the scrollbar itself is visible - `"always"` (default),
        /// `"hover"`, `"hidden"`, or `"scroll"` (currently identical to
        /// `"hover"`).
        #[props(default, into)]
        scrollbar_visibility: Input<ScrollbarVisibility>,
        /// Maps to the CSS `scrollbar-width` keyword - `"thin"` (default)
        /// or `"auto"`.
        #[props(default, into)]
        scrollbar_size: Input<ScrollbarSize>,
        /// Scrollbar thumb color - track stays transparent.
        #[props(default, into)]
        scrollbar_color: Input<ThemeAwareValue>,
        /// Percent (0-100) along the horizontal axis to scroll to.
        /// `None` (default) leaves the scroll position at 0. A value bound
        /// to the caller's own state re-applies every time it changes;
        /// a literal only applies once, at mount.
        scroll_position_x: Option<f64>,
        /// Percent (0-100) along the vertical axis - see `scroll_position_x`.
        scroll_position_y: Option<f64>,
        #[props(default)]
        on_scroll: EventHandler<ScrollPositionEvent>,
        #[props(default)]
        on_top_reached: EventHandler<()>,
        #[props(default)]
        on_bottom_reached: EventHandler<()>,
        #[props(default)]
        on_left_reached: EventHandler<()>,
        #[props(default)]
        on_right_reached: EventHandler<()>,
        children: Element,
    }
}

/// Scrolls its content, filling the parent by default. Read the scroll
/// position via `on_scroll`/`on_*_reached`; set it imperatively via
/// `scroll_position_x`/`scroll_position_y` (reactive if bound to a signal,
/// initial-only if a literal).
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    let theme = use_theme();
    let root_id = use_hook(|| format!("lsx-scroll-area-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed)));

    let scrollbars = props
        .scrollbars
        .as_ref()
        .copied()
        .unwrap_or(theme.scroll_area.scrollbars);
    let visibility = props
        .scrollbar_visibility
        .as_ref()
        .copied()
        .unwrap_or(theme.scroll_area.visibility);
    let size = props
        .scrollbar_size
        .as_ref()
        .copied()
        .unwrap_or(theme.scroll_area.size);

    let mut is_scrolling = use_signal(|| false);
    let mut edges = use_signal(EdgeState::default);

    let on_scroll = props.on_scroll;
    let on_top_reached = props.on_top_reached;
    let on_bottom_reached = props.on_bottom_reached;
    let on_left_reached = props.on_left_reached;
    let on_right_reached = props.on_right_reached;

    let onscroll = move |event: Event<ScrollData>| {
        let data = event.data();
        let max_x = (data.scroll_width() - data.client_width()).max(0) as f64;
        let max_y = (data.scroll_height() - data.client_height()).max(0) as f64;
        let x_pct = if max_x > 0.0 { data.scroll_left() / max_x * 100.0 } else { 0.0 };
        let y_pct = if max_y > 0.0 { data.scroll_top() / max_y * 100.0 } else { 0.0 };

        if is_scrolling() {
            on_scroll.call(ScrollPositionEvent::Change(x_pct, y_pct));
        } else {
            is_scrolling.set(true);
            on_scroll.call(ScrollPositionEvent::Start(x_pct, y_pct));
        }

        let new_edges = EdgeState {
            top: data.scroll_top() <= 0.0,
            bottom: max_y <= 0.0 || data.scroll_top() >= max_y - 1.0,
            left: data.scroll_left() <= 0.0,
            right: max_x <= 0.0 || data.scroll_left() >= max_x - 1.0,
        };
        let previous = edges();
        if new_edges.top && !previous.top {
            on_top_reached.call(());
        }
        if new_edges.bottom && !previous.bottom {
            on_bottom_reached.call(());
        }
        if new_edges.left && !previous.left {
            on_left_reached.call(());
        }
        if new_edges.right && !previous.right {
            on_right_reached.call(());
        }
        edges.set(new_edges);
    };

    let onscrollend = move |_| {
        if is_scrolling() {
            is_scrolling.set(false);
        }
    };

    let root_id_for_position = root_id.clone();
    let scroll_position_x = props.scroll_position_x;
    let scroll_position_y = props.scroll_position_y;
    use_effect(use_reactive!(|scroll_position_x, scroll_position_y| {
        if scroll_position_x.is_none() && scroll_position_y.is_none() {
            return;
        }
        let Ok(container) = dom_api().query_selector(&format!("#{root_id_for_position}")) else {
            return;
        };
        let Ok(scroll_size) = container.scroll_size() else {
            return;
        };
        let Ok(viewport) = container.dimensions() else {
            return;
        };
        let Ok((current_x, current_y)) = container.scroll_offset() else {
            return;
        };
        let max_x = (scroll_size.width - viewport.width).max(0.0);
        let max_y = (scroll_size.height - viewport.height).max(0.0);

        let x = scroll_position_x.map_or(current_x, |pct| max_x * pct.clamp(0.0, 100.0) / 100.0);
        let y = scroll_position_y.map_or(current_y, |pct| max_y * pct.clamp(0.0, 100.0) / 100.0);
        let _ = container.scroll_to(x, y);
    }));

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(axis_token(scrollbars), true)
        .with(visibility_token(visibility), true)
        .with(size_token(size), true);

    let variables = scroll_area_variables(props.scrollbar_color.as_ref());

    rsx! {
        Box {
            id: "{root_id}",
            class: props.class,
            sx: props.sx,
            states,
            variables,
            framework_sx: &SCROLL_AREA_BASE_SX,
            // Chromium makes a scrollable `overflow: auto` region with actual
            // overflowing content an implicit tab stop of its own (arrow-key/
            // Page-Down scrolling) unless opted out - always true here since
            // scrolling is this component's entire purpose, and its content
            // is expected to carry its own focusable elements already.
            tabindex: "-1",
            attributes: props.attributes,
            onscroll,
            onscrollend,
            {props.children}
        }
    }
}
