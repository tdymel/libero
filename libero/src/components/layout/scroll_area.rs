use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States, Variables,
        common::{base_props, dom_api, input_from_str, variables},
    },
    hooks::{use_root_id, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
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

/// `Scroll` behaves as `Hover`: nothing here can yet fade the scrollbar out
/// after an idle timeout. See `ScrollbarVisibility`.
fn visibility_token(visibility: ScrollbarVisibility) -> &'static str {
    match visibility {
        ScrollbarVisibility::Scroll => ScrollbarVisibility::Hover.state_name(),
        visibility => visibility.state_name(),
    }
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

fn scroll_area_variables(color: Option<&ThemeAwareValue>) -> Variables {
    variables().with(SCROLL_AREA_THUMB_VAR, color.and_then(|v| v.resolve(None)))
}

/// Which edges the last scroll position rested against, so `on_*_reached`
/// fires on the rising edge rather than every event.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct EdgeState {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
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
/// position via `on_scroll`/`on_*_reached`; set it imperatively via
/// `scroll_position_x`/`scroll_position_y` (reactive if bound to a signal,
/// initial-only if a literal).
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    let theme = use_theme();
    let root_id = use_root_id(&props.attributes);

    let scrollbars = props.scrollbars.copied_or(theme.scroll_area.scrollbars);
    let visibility = props
        .scrollbar_visibility
        .copied_or(theme.scroll_area.visibility);
    let size = props.scrollbar_size.copied_or(theme.scroll_area.size);

    let mut is_scrolling = use_signal(|| false);
    let mut edges = use_signal(EdgeState::default);

    let on_scroll = props.on_scroll;
    let on_top_reached = props.on_top_reached;
    let on_bottom_reached = props.on_bottom_reached;
    let on_left_reached = props.on_left_reached;
    let on_right_reached = props.on_right_reached;

    let onscroll = move |event: Event<ScrollData>| {
        let data = event.data();
        let (x_pct, y_pct, max_x, max_y) = scroll_metrics(&data);

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

    let onscrollend = move |event: Event<ScrollData>| {
        if is_scrolling() {
            is_scrolling.set(false);
            let (x_pct, y_pct, ..) = scroll_metrics(&event.data());
            on_scroll.call(ScrollPositionEvent::End(x_pct, y_pct));
        }
    };

    let scroll_position_x = props.scroll_position_x;
    let scroll_position_y = props.scroll_position_y;
    use_effect(use_reactive!(|scroll_position_x, scroll_position_y| {
        if scroll_position_x.is_none() && scroll_position_y.is_none() {
            return;
        }
        let Ok(container) = dom_api().query_selector(&format!("#{}", root_id())) else {
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
        .with(scrollbars.state_name(), true)
        .with(visibility_token(visibility), true)
        .with(size.state_name(), true);

    let variables = scroll_area_variables(props.scrollbar_color.as_ref());

    rsx! {
        Box {
            id: "{root_id}",
            class: props.class,
            sx: props.sx,
            states,
            variables,
            framework_sx: &SCROLL_AREA_BASE_SX,
            // Chromium makes an overflowing `overflow: auto` region an
            // implicit tab stop unless opted out. Always the case here, and
            // the content carries its own focusable elements.
            tabindex: "-1",
            attributes: props.attributes,
            onscroll,
            onscrollend,
            {props.children}
        }
    }
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
