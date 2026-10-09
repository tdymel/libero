use dioxus::{html::input_data::MouseButton, prelude::*};

use super::{
    handle::{inline_x, physical_x},
    scroll_area::{SCROLLBARS_SLOT, ScrollAreaPart, thumb_color},
};
use crate::{
    components::{
        common::{HtmlTag, Part},
        layout::use_box,
    },
    hooks::{DragMove, DragOptions, ElementHandle, use_drag, use_element},
    platform::{ElementApi, scroll_timelines, when_laid_out},
    sx::{FORCED_COLORS, StaticSx, sx},
    theme::{
        SCROLL_AREA_RANGE_X, SCROLL_AREA_RANGE_Y, SCROLL_AREA_THUMB_TRAVEL, ScrollAxis,
        ScrollbarSize,
    },
};

/// Two layers over the area's padding box, out of its flow, each shifted by
/// the scroll offset on one axis so the tracks stay put while the content
/// moves. `SCROLL_AREA_KEYFRAMES` shifts them on the compositor where scroll
/// timelines exist; elsewhere the inline `translate` does, a frame late.
static DRAWN_BARS_SX: StaticSx = StaticSx::new(|| {
    let layer = || {
        sx().position("absolute")
            .top("0")
            .with("inset-inline-start", "0")
            .width("100%")
            .height("100%")
            .pointer_events("none")
            // Only the tracks show: a visible layer over the rows hides their contrast from axe.
            .with("visibility", "hidden")
    };
    // Above any row, inside the area's own stacking context.
    layer()
        .z_index("2147483647")
        // Never the scroll anchor: Firefox scrolled on after it as it followed
        // the offset, a runaway to the bottom (todo 1455).
        .with("overflow-anchor", "none")
        .selector("& > [data-scrollbars-x]", layer())
        .selector(
            "& [data-slot='scrollbar']",
            sx().position("absolute")
                .pointer_events("auto")
                .with("visibility", "visible"),
        )
        .selector(
            "& [data-slot='scrollbar'][data-orientation='vertical']",
            sx().top("0").with("inset-inline-end", "0"),
        )
        .selector(
            "& [data-slot='scrollbar'][data-orientation='horizontal']",
            sx().bottom("0").with("inset-inline-start", "0"),
        )
        .selector(
            "& [data-slot='thumb']",
            sx().position("absolute")
                .box_sizing("border-box")
                .padding("2px")
                // The whole width grabs; the inset part shows.
                .with("background-clip", "content-box")
                .background_color(thumb_color())
                .border_radius("9999px")
                .touch_action("none")
                .media(
                    FORCED_COLORS,
                    sx().with("forced-color-adjust", "none")
                        .background_color("CanvasText"),
                ),
        )
        .selector(
            "& [data-orientation='vertical'] > [data-slot='thumb']",
            sx().with("inset-inline", "0"),
        )
        .selector(
            "& [data-orientation='horizontal'] > [data-slot='thumb']",
            sx().with("inset-block", "0"),
        )
});

/// A thumb stays grabbable however long the content.
const MIN_THUMB: f64 = 20.0;

/// How big a drawn scrollbar is across, in px.
pub(super) fn thickness(size: ScrollbarSize) -> f64 {
    match size {
        ScrollbarSize::Thin => 8.0,
        ScrollbarSize::Auto => 12.0,
    }
}

/// What the drawn bars need of their area, in px.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ScrollMetrics {
    /// The padding box: `clientWidth`/`clientHeight`.
    pub view_width: f64,
    pub view_height: f64,
    pub content_width: f64,
    pub content_height: f64,
    /// From the inline start, so RTL shares the origin.
    pub x: f64,
    pub y: f64,
    /// `scrollLeft` as the platform has it, negative under RTL.
    pub left: f64,
}

impl ScrollMetrics {
    /// Whether `next` changes what the bars draw. Under scroll timelines the offsets
    /// move them on the compositor, so only a size does.
    fn redraws(previous: Option<Self>, next: Self, timelines: bool) -> bool {
        let Some(previous) = previous else {
            return true;
        };
        match timelines {
            true => {
                let size = |m: Self| {
                    (
                        m.view_width,
                        m.view_height,
                        m.content_width,
                        m.content_height,
                    )
                };
                size(previous) != size(next)
            }
            false => previous != next,
        }
    }
}

/// One drawn bar, along its axis: track length, thumb length and thumb start.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Bar {
    pub track: f64,
    pub thumb: f64,
    pub at: f64,
    range: f64,
}

impl Bar {
    fn new(view: f64, content: f64, offset: f64, track: f64) -> Option<Self> {
        let range = content - view;
        if range <= 0.0 || track <= 0.0 {
            return None;
        }
        let thumb = (track * view / content).max(MIN_THUMB).min(track);
        let at = (track - thumb) * (offset / range).clamp(0.0, 1.0);
        Some(Self {
            track,
            thumb,
            at,
            range,
        })
    }

    /// The thumb's inline `translate`, which a scroll timeline's animation overrides:
    /// a WebView runs those while we write offsets too (todo 2019). None then, so a scroll writes nothing.
    fn shift(&self, timelines: bool, leftwards: bool) -> f64 {
        match (timelines, leftwards) {
            (true, _) => 0.0,
            (false, true) => -self.at,
            (false, false) => self.at,
        }
    }

    /// How far a scroll timeline moves the thumb, in px: leftwards for an RTL x bar.
    fn travel(&self, leftwards: bool) -> f64 {
        let travel = self.track - self.thumb;
        if leftwards { -travel } else { travel }
    }

    /// The scroll offset that puts the thumb's start `at` px along the track.
    pub fn offset_at(&self, at: f64) -> f64 {
        let free = self.track - self.thumb;
        if free <= 0.0 {
            return 0.0;
        }
        at.clamp(0.0, free) / free * self.range
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Bars {
    pub x: Option<Bar>,
    pub y: Option<Bar>,
}

/// The bars to draw: one per scrolled axis that overflows. The vertical track
/// starts `top` px down and ends `bottom` px up, clear of a sticky header and footer.
pub(super) fn bars(
    axis: ScrollAxis,
    metrics: ScrollMetrics,
    thickness: f64,
    (top, bottom): (f64, f64),
) -> Bars {
    // A pixel of slack, as for the tab stop: a rounded box is not overflow.
    let over = |content: f64, view: f64| content > view + 1.0;
    let y = matches!(axis, ScrollAxis::Vertical | ScrollAxis::Both)
        && over(metrics.content_height, metrics.view_height);
    let x = matches!(axis, ScrollAxis::Horizontal | ScrollAxis::Both)
        && over(metrics.content_width, metrics.view_width);
    // Both drawn: each stops short of the corner the other ends in.
    let corner = |other: bool| if other { thickness } else { 0.0 };
    Bars {
        y: y.then(|| {
            Bar::new(
                metrics.view_height,
                metrics.content_height,
                metrics.y,
                metrics.view_height - corner(x) - top.max(0.0) - bottom.max(0.0),
            )
        })
        .flatten(),
        x: x.then(|| {
            Bar::new(
                metrics.view_width,
                metrics.content_width,
                metrics.x,
                metrics.view_width - corner(y),
            )
        })
        .flatten(),
    }
}

/// What a `ScrollArea` shares with its [`ScrollAreaBars`].
#[derive(Clone, Copy, PartialEq)]
pub(super) struct DrawnBars {
    pub root: ElementHandle,
    /// The drawn layer, as big as the area's padding box.
    pub layer: ElementHandle,
    /// What the bars drew from.
    pub metrics: Signal<Option<ScrollMetrics>>,
    /// The latest read, offsets included: under scroll timelines a scroll alone
    /// leaves `metrics` behind.
    pub latest: CopyValue<Option<ScrollMetrics>>,
}

impl DrawnBars {
    fn update(mut self, next: ScrollMetrics) -> bool {
        self.latest.set(Some(next));
        let redraws = ScrollMetrics::redraws(*self.metrics.peek(), next, scroll_timelines());
        if redraws {
            self.metrics.set(Some(next));
        }
        redraws
    }

    /// From a scroll event, which carries every size and offset.
    pub fn scrolled(self, data: &ScrollData) {
        self.update(ScrollMetrics {
            view_width: data.client_width() as f64,
            view_height: data.client_height() as f64,
            content_width: data.scroll_width() as f64,
            content_height: data.scroll_height() as f64,
            x: inline_x(data.scroll_left()),
            y: data.scroll_top(),
            left: data.scroll_left(),
        });
    }

    /// Re-reads the geometry once laid out. A changed one is read once more:
    /// the tracks it replaced may have stretched the overflow it measured.
    pub fn measure(self, tries: u8) {
        let Self { root, layer, .. } = self;
        // Mounting the layer measures again.
        if !layer.is_mounted() {
            return;
        }
        when_laid_out(move || {
            // The layer, not the border box less `computed_px` borders: a WebView has none.
            let (size, content, offset) =
                (layer.dimensions(), root.scroll_size(), root.scroll_offset());
            spawn(async move {
                let (Ok(size), Ok(content), Ok((left, y))) =
                    (size.await, content.await, offset.await)
                else {
                    return;
                };
                let measured = ScrollMetrics {
                    view_width: size.width,
                    view_height: size.height,
                    content_width: content.width,
                    content_height: content.height,
                    x: inline_x(left),
                    y,
                    left,
                };
                if self.update(measured) && tries > 0 {
                    self.measure(tries - 1);
                }
            });
        });
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Axis {
    X,
    Y,
}

/// The track and thumb `ScrollArea` draws for `Always`. Hidden from assistive
/// technology and never focused: the area itself scrolls and holds the tab stop.
#[component]
pub(super) fn ScrollAreaBars(
    state: DrawnBars,
    scrollbars: ScrollAxis,
    size: ScrollbarSize,
    inset_top: f64,
    inset_bottom: f64,
) -> Element {
    let DrawnBars {
        root,
        metrics,
        latest,
        ..
    } = state;
    let thick = thickness(size);
    let current = move |axis: Axis| {
        let drawn = bars(scrollbars, latest()?, thick, (inset_top, inset_bottom));
        match axis {
            Axis::X => drawn.x,
            Axis::Y => drawn.y,
        }
    };
    // Scrolls so the thumb starts `at` px along its track; the other axis stays.
    let scroll_along = move |axis: Axis, at: f64, rtl: bool| {
        let (Some(metrics), Some(bar)) = (latest(), current(axis)) else {
            return;
        };
        let (x, y) = match axis {
            Axis::X => (bar.offset_at(at), metrics.y),
            Axis::Y => (metrics.x, bar.offset_at(at)),
        };
        let _ = root.scroll_to(physical_x(x, rtl), y);
    };

    // Where the thumb started and whether the area was RTL, for the drag in hand.
    let mut grab = use_hook(|| CopyValue::new((0.0, false)));
    let thumb_drag = |axis: Axis, thumb: ElementHandle| {
        use_drag(DragOptions {
            capture: thumb,
            onstart: Callback::new(move |_| {
                if let Some(bar) = current(axis) {
                    grab.set((bar.at, root.is_rtl()));
                }
            }),
            onmove: Callback::new(move |step: DragMove| {
                let (from, rtl) = grab();
                let delta = match axis {
                    Axis::Y => step.delta().y,
                    // The inline start is on the right under RTL.
                    Axis::X if rtl => -step.delta().x,
                    Axis::X => step.delta().x,
                };
                scroll_along(axis, from + delta, rtl);
            }),
            onend: Callback::new(|()| {}),
        })
    };
    let (thumb_x, thumb_y) = (use_element(), use_element());
    let drag_x = thumb_drag(Axis::X, thumb_x);
    let drag_y = thumb_drag(Axis::Y, thumb_y);

    // A press on the track centres the thumb on the pointer.
    let press = move |axis: Axis| {
        move |event: Event<PointerData>| {
            if matches!(event.trigger_button(), Some(button) if button != MouseButton::Primary) {
                return;
            }
            event.prevent_default();
            let Some(bar) = current(axis) else {
                return;
            };
            let rtl = root.is_rtl();
            let point = event.element_coordinates();
            let along = match axis {
                Axis::Y => point.y,
                Axis::X if rtl => bar.track - point.x,
                Axis::X => point.x,
            };
            scroll_along(axis, along - bar.thumb / 2.0, rtl);
        }
    };

    let measured = metrics();
    let layer_style = measured.map(|measured| {
        let range_y = (measured.content_height - measured.view_height).max(0.0);
        format!(
            "translate: 0 {}px; {}: {range_y}px",
            measured.y,
            SCROLL_AREA_RANGE_Y.name()
        )
    });
    let layer = use_box()
        .framework_sx(&DRAWN_BARS_SX)
        .focus_ring(false)
        .style(layer_style)
        .prepare()
        .element(&state.layer);
    // Empty until a bar shows: its size is what the area measures. Empty, it
    // runs no scroll animation, which would never settle.
    let render = |tracks: Option<Element>| {
        layer
            .attr("data-slot", SCROLLBARS_SLOT)
            .attr("data-empty", tracks.is_none())
            .attr("aria-hidden", "true")
            .render(HtmlTag::Div, Vec::new(), tracks.unwrap_or_else(|| rsx! {}))
    };

    let Some(measured) = measured else {
        return render(None);
    };
    let drawn = bars(scrollbars, measured, thick, (inset_top, inset_bottom));
    if drawn.x.is_none() && drawn.y.is_none() {
        return render(None);
    }
    // Under RTL the content moves right as it scrolls on: the layer follows left.
    let range_x = (measured.content_width - measured.view_width).max(0.0);
    let rtl = needs_direction(&drawn, range_x) && root.is_rtl();
    let range_x = if rtl { -range_x } else { range_x };
    let corner = |other: Option<Bar>| if other.is_some() { thick } else { 0.0 };
    let timelines = scroll_timelines();

    let tracks = rsx! {
        div {
            "data-scrollbars-x": true,
            style: "translate: {measured.left}px 0; {SCROLL_AREA_RANGE_X.name()}: {range_x}px",
            if let Some(bar) = drawn.y {
                div {
                    "data-slot": ScrollAreaPart::Scrollbar.slot(),
                    "data-orientation": "vertical",
                    style: "top: {inset_top.max(0.0)}px; bottom: {corner(drawn.x) + inset_bottom.max(0.0)}px; width: {thick}px",
                    onpointerdown: press(Axis::Y),
                    div {
                        "data-slot": ScrollAreaPart::Thumb.slot(),
                        style: "top: 0; translate: 0 {bar.shift(timelines, false)}px; height: {bar.thumb}px; {SCROLL_AREA_THUMB_TRAVEL.name()}: {bar.travel(false)}px",
                        onmounted: thumb_y.mount(),
                        onpointerdown: move |event: Event<PointerData>| {
                            event.stop_propagation();
                            drag_y.onpointerdown.call(event);
                        },
                        onpointermove: drag_y.onpointermove,
                        onpointerup: drag_y.onpointerup,
                        onpointercancel: drag_y.onpointercancel,
                    }
                }
            }
            if let Some(bar) = drawn.x {
                div {
                    "data-slot": ScrollAreaPart::Scrollbar.slot(),
                    "data-orientation": "horizontal",
                    style: "inset-inline-end: {corner(drawn.y)}px; height: {thick}px",
                    onpointerdown: press(Axis::X),
                    div {
                        "data-slot": ScrollAreaPart::Thumb.slot(),
                        style: "inset-inline-start: 0; translate: {bar.shift(timelines, rtl)}px 0; width: {bar.thumb}px; {SCROLL_AREA_THUMB_TRAVEL.name()}: {bar.travel(rtl)}px",
                        onmounted: thumb_x.mount(),
                        onpointerdown: move |event: Event<PointerData>| {
                            event.stop_propagation();
                            drag_x.onpointerdown.call(event);
                        },
                        onpointermove: drag_x.onpointermove,
                        onpointerup: drag_x.onpointerup,
                        onpointercancel: drag_x.onpointercancel,
                    }
                }
            }
        }
    };
    render(Some(tracks))
}

/// The direction is a computed-style read: only a horizontal bar or range needs it (todo 2060).
fn needs_direction(drawn: &Bars, range_x: f64) -> bool {
    drawn.x.is_some() || range_x > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 100px view over 400px of content.
    const TALL: ScrollMetrics = ScrollMetrics {
        view_width: 200.0,
        view_height: 100.0,
        content_width: 200.0,
        content_height: 400.0,
        x: 0.0,
        y: 0.0,
        left: 0.0,
    };

    #[test]
    fn the_thumb_is_the_visible_share_of_the_track() {
        let bar = bars(ScrollAxis::Vertical, TALL, 8.0, (0.0, 0.0)).y.unwrap();

        assert_eq!((bar.track, bar.thumb, bar.at), (100.0, 25.0, 0.0));
    }

    #[test]
    fn the_thumb_travels_with_the_offset() {
        let half = ScrollMetrics { y: 150.0, ..TALL };
        let end = ScrollMetrics { y: 300.0, ..TALL };

        assert_eq!(
            bars(ScrollAxis::Vertical, half, 8.0, (0.0, 0.0))
                .y
                .unwrap()
                .at,
            37.5
        );
        assert_eq!(
            bars(ScrollAxis::Vertical, end, 8.0, (0.0, 0.0))
                .y
                .unwrap()
                .at,
            75.0
        );
    }

    #[test]
    fn a_long_list_keeps_a_grabbable_thumb() {
        let long = ScrollMetrics {
            content_height: 100_000.0,
            ..TALL
        };

        assert_eq!(
            bars(ScrollAxis::Vertical, long, 8.0, (0.0, 0.0))
                .y
                .unwrap()
                .thumb,
            MIN_THUMB
        );
    }

    #[test]
    fn content_that_fits_or_an_unscrolled_axis_draws_nothing() {
        let fits = ScrollMetrics {
            content_height: 100.5,
            ..TALL
        };

        assert_eq!(bars(ScrollAxis::Vertical, fits, 8.0, (0.0, 0.0)).y, None);
        assert_eq!(
            bars(ScrollAxis::Horizontal, TALL, 8.0, (0.0, 0.0)),
            Bars { x: None, y: None }
        );
        assert_eq!(
            bars(ScrollAxis::None, TALL, 8.0, (0.0, 0.0)),
            Bars { x: None, y: None }
        );
    }

    #[test]
    fn two_bars_leave_the_corner_free() {
        let both = ScrollMetrics {
            content_width: 800.0,
            ..TALL
        };
        let drawn = bars(ScrollAxis::Both, both, 8.0, (0.0, 0.0));

        assert_eq!(drawn.y.unwrap().track, 92.0);
        assert_eq!(drawn.x.unwrap().track, 192.0);
    }

    /// Todo 1454: below a 30px sticky header, the track and its thumb shrink by it.
    #[test]
    fn a_top_inset_shortens_the_vertical_track_only() {
        let both = ScrollMetrics {
            content_width: 800.0,
            content_height: 200.0,
            ..TALL
        };
        let drawn = bars(ScrollAxis::Both, both, 8.0, (30.0, 0.0));

        assert_eq!(drawn.y.unwrap().track, 62.0);
        assert_eq!(drawn.y.unwrap().thumb, 31.0);
        assert_eq!(drawn.x.unwrap().track, 192.0);
    }

    /// Todo 2770: above a 30px sticky footer, the same.
    #[test]
    fn a_bottom_inset_shortens_the_vertical_track_only() {
        let both = ScrollMetrics {
            content_width: 800.0,
            content_height: 200.0,
            ..TALL
        };
        let drawn = bars(ScrollAxis::Both, both, 8.0, (0.0, 30.0));

        assert_eq!(drawn.y.unwrap().track, 62.0);
        assert_eq!(drawn.x.unwrap().track, 192.0);
    }

    #[test]
    fn a_thumb_position_maps_back_to_its_offset() {
        let bar = bars(ScrollAxis::Vertical, TALL, 8.0, (0.0, 0.0)).y.unwrap();

        assert_eq!(bar.offset_at(37.5), 150.0);
        assert_eq!(bar.offset_at(-10.0), 0.0);
        assert_eq!(bar.offset_at(500.0), 300.0);
    }

    /// Todo 1954: a scroll timeline moves the thumb, so a scroll alone redraws nothing.
    #[test]
    fn under_scroll_timelines_only_a_size_redraws() {
        let scrolled = ScrollMetrics { y: 150.0, ..TALL };
        let grown = ScrollMetrics {
            content_height: 800.0,
            ..TALL
        };

        assert!(ScrollMetrics::redraws(None, TALL, true));
        assert!(!ScrollMetrics::redraws(Some(TALL), scrolled, true));
        assert!(ScrollMetrics::redraws(Some(scrolled), grown, true));
        let bar = bars(ScrollAxis::Vertical, scrolled, 8.0, (0.0, 0.0))
            .y
            .unwrap();
        assert_eq!((bar.shift(true, false), bar.travel(false)), (0.0, 75.0));
        assert_eq!(bar.shift(true, true), 0.0);
        assert_eq!(bar.travel(true), -75.0);
    }

    #[test]
    fn only_a_horizontal_bar_or_range_reads_the_direction() {
        let wide = ScrollMetrics {
            content_width: 800.0,
            ..TALL
        };

        assert!(!needs_direction(
            &bars(ScrollAxis::Vertical, TALL, 8.0, (0.0, 0.0)),
            0.0
        ));
        assert!(needs_direction(
            &bars(ScrollAxis::Both, wide, 8.0, (0.0, 0.0)),
            600.0
        ));
        assert!(needs_direction(
            &bars(ScrollAxis::Vertical, wide, 8.0, (0.0, 0.0)),
            600.0
        ));
    }

    /// Without scroll timelines every offset redraws, the inline style placing the thumb.
    #[test]
    fn without_scroll_timelines_a_scroll_redraws() {
        let scrolled = ScrollMetrics { y: 150.0, ..TALL };

        assert!(ScrollMetrics::redraws(Some(TALL), scrolled, false));
        assert!(!ScrollMetrics::redraws(Some(scrolled), scrolled, false));
        let bar = bars(ScrollAxis::Vertical, scrolled, 8.0, (0.0, 0.0))
            .y
            .unwrap();
        assert_eq!(bar.shift(false, false), 37.5);
        // An RTL x thumb shifts left from its inline start, the right edge.
        assert_eq!(bar.shift(false, true), -37.5);
    }
}
