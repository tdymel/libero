//! The interactive layer over a drawn chart: pointer, keys, tooltip, crosshair, marker.
//! It reads only the harvested marks, so every chart kind shares it.

use std::cell::Cell;

use dioxus::{
    html::{
        geometry::{ElementPoint, WheelDelta},
        input_data::MouseButton,
    },
    prelude::*,
};

use super::{
    ChartPart, ChartPoint, ChartRange, ValueFormat,
    engine::{Drawn, Geometry, Mark, MarkShape, Plot},
    hit::{self, Step},
    zoom::{Zoom, index_at, index_span, panned, zoomed},
};
use crate::{
    components::{
        accessibility::use_announcer,
        common::{Part, focus_ring_sx, has_shortcut_modifier},
    },
    hooks::{use_debounced_callback, use_element, use_escape_dismiss},
    localization::{ChartLabels, Formats, fill},
    platform::{ElementApi, wheel_travel_y},
    sx::{FORCED_COLORS, REDUCED_MOTION, Sx, sx},
    theme::{Size, SizeCss, TOOLTIP_BACKGROUND, TOOLTIP_COLOR},
    tokens::NamedColorCss,
};

/// Room between a point and its tooltip, in px.
const TIP_GAP: f64 = 12.0;
/// Half the marker's stroke, so all of it lies inside its bar.
const MARKER_INSET: f32 = 1.0;
/// How far a mouse moves, in px, before a press drags.
const MOUSE_SLOP: f64 = 4.0;
/// How far a touch moves along the x axis before it drags: Android's touch slop.
const TOUCH_SLOP: f64 = 8.0;
/// Wheel px that zoom by e: a trackpad pinch's spread or a few notches.
const WHEEL_ZOOM: f64 = 300.0;
/// A one-value brush's half width, in SVG units, so it shows on a line's point.
const BRUSH_MIN: f32 = 4.0;
/// Quiet time before the shown range is announced.
const SHOWING_DELAY: u64 = 400;

/// The overlay's rules, under the chart's plot.
pub(super) fn overlay_sx() -> Sx {
    let layer = sx().position("absolute").with("inset", "0");
    sx().selector(
        "& > [data-chart-marks]",
        layer.clone().with("pointer-events", "none"),
    )
    .selector(
        "& > [data-chart-layer]",
        layer
            // A vertical swipe still scrolls the page; a sideways one scrubs the points.
            .with("touch-action", "pan-y pinch-zoom")
            .border_radius("sm")
            .focus_visible(focus_ring_sx()),
    )
    // A sideways drag pans or brushes and two fingers zoom: only a vertical swipe stays the page's.
    .selector(
        "& > [data-chart-layer][data-gestures]",
        sx().with("touch-action", "pan-y"),
    )
    .selector(
        "& [data-chart-brush]",
        sx().with("fill", NamedColorCss::INK.value())
            .with("fill-opacity", "0.12")
            .with("stroke", NamedColorCss::INK.value())
            .with("stroke-width", "1")
            .media(
                FORCED_COLORS,
                sx().with("fill", "Highlight").with("stroke", "Highlight"),
            ),
    )
    .selector(
        "& > [data-slot='tooltip']",
        sx().position("absolute")
            .with("pointer-events", "none")
            .with("z-index", "1")
            .with("max-width", "100%")
            .padding("4px 8px")
            .border_radius("sm")
            .background_color(TOOLTIP_BACKGROUND.value())
            .color(TOOLTIP_COLOR.value())
            .box_shadow(SizeCss::SHADOW.value(Size::Sm))
            // Forced colours drop the shadow but paint a transparent outline.
            .outline("1px solid transparent")
            .font_size("sm")
            // As wide as its rows up to the plot's width, where a long name wraps.
            .width("max-content")
            .with("overflow-wrap", "anywhere")
            .with(
                "transition",
                "left 120ms ease-out, top 120ms ease-out, transform 120ms ease-out",
            )
            .media(REDUCED_MOTION, sx().with("transition", "none")),
    )
    .selector(
        "& > [data-slot='tooltip'] [data-chart-row]",
        sx().display("flex").align_items("center").gap("xs"),
    )
    .selector(
        "& > [data-slot='tooltip'] [data-chart-row] > :last-child",
        sx().with("margin-inline-start", "auto")
            .with("padding-inline-start", "12px")
            .with("flex", "none")
            .white_space("nowrap"),
    )
    .selector(
        "& > [data-slot='tooltip'] [data-active]",
        sx().font_weight("600"),
    )
    .selector(
        "& [data-chart-crosshair]",
        sx().with("stroke", NamedColorCss::TEXT_DIMMED.value())
            .with("stroke-dasharray", "4 4"),
    )
    .selector(
        "& [data-chart-marker]",
        sx().with("stroke", NamedColorCss::INK.value())
            .with("stroke-width", "2"),
    )
    .selector(
        "& [data-chart-dot]",
        sx().with("stroke", NamedColorCss::SURFACE.value()),
    )
}

/// A series as the overlay names and paints it.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct OverlaySeries {
    pub(super) name: String,
    pub(super) color: String,
    /// Drawn dashed: its legend key is a dash.
    pub(super) dashed: bool,
    /// The shape its points are drawn in, for its legend key; `None`, a plain swatch.
    pub(super) symbol: Option<crate::theme::PointSymbol>,
    /// A line among bars: its legend key is a stroke.
    pub(super) line: bool,
    /// The series' own unit; unset, the chart's.
    pub(super) unit: Option<String>,
    /// Each value's error range, written after it.
    pub(super) errors: Vec<Option<(f64, f64)>>,
    /// A scatter's x values as written, by index; empty elsewhere.
    pub(super) xs: Vec<String>,
    /// A point as the tooltip and the keys say it, by index: a scatter's x and y, a candle's
    /// prices. Empty, its value.
    pub(super) points: Vec<String>,
}

/// How a chart's points lie, for the pointer, the keys, the crosshair and the tooltip.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum Layout {
    /// In columns over the x values: lines, bars, slices.
    #[default]
    Columns,
    /// Anywhere, each with its own x: a scatter's.
    Free,
    /// On a radar's spokes, one column per spoke.
    Spokes,
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct ChartOverlayProps {
    pub(super) drawn: Memo<Option<Drawn>>,
    /// The plot's box in CSS px, once measured.
    pub(super) box_size: Signal<Option<(f64, f64)>>,
    /// Every series, hidden ones too, at its place in the chart's `series`.
    pub(super) series: Vec<OverlaySeries>,
    pub(super) x: Vec<String>,
    pub(super) format: ValueFormat,
    pub(super) unit: Option<String>,
    pub(super) formats: &'static Formats,
    pub(super) labels: ChartLabels,
    /// The chart's title, the plot's name.
    pub(super) title_id: Option<String>,
    pub(super) keys_id: String,
    pub(super) tooltip: bool,
    pub(super) tooltip_content: Option<Callback<ChartPoint, Element>>,
    pub(super) onpointclick: Option<EventHandler<ChartPoint>>,
    pub(super) onactivechange: Option<EventHandler<Option<ChartPoint>>>,
    /// Up and down walk the x values, left and right the series: a horizontal bar chart.
    pub(super) swap_axes: bool,
    pub(super) layout: Layout,
    pub(super) zoom: Option<Zoom>,
    pub(super) onrange: Option<EventHandler<Option<ChartRange>>>,
}

/// What a press on the plot is doing, by the pointer's id; positions in the layer's px.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Gesture {
    /// Not yet far enough to drag: still a tap or a click.
    Press {
        id: i32,
        x: f64,
        y: f64,
        touch: bool,
        pan: bool,
    },
    Pan {
        id: i32,
        from: f64,
        window: (usize, usize),
    },
    Brush {
        id: i32,
        anchor: usize,
    },
    /// Two fingers: their spread and middle at the start, along the x axis.
    Pinch {
        spread: f64,
        at: f64,
        window: (usize, usize),
    },
}

/// The series a mark belongs to; a pie's slices are its entries, one per index.
fn entry<'a>(series: &'a [OverlaySeries], mark: &Mark) -> Option<&'a OverlaySeries> {
    match mark.shape {
        MarkShape::Sector { .. } => series.get(mark.index),
        _ => series.get(mark.series),
    }
}

/// A slice's part of the drawn ones, a stage's of the funnel's first, as the labels write it.
fn share(marks: &[Mark], share_of: Option<f64>, mark: &Mark, formats: &Formats) -> String {
    let total = share_of.unwrap_or_else(|| marks.iter().map(|mark| mark.value).sum());
    let part = if total > 0.0 { mark.value / total } else { 0.0 };
    ValueFormat::Percent.format(part, None, formats)
}

/// SVG units to the box's px: the plot scales its SVG to fit and centres it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Fit {
    scale: f64,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl Fit {
    fn of(svg: (f32, f32), size: Option<(f64, f64)>) -> Self {
        let (w, h) = (f64::from(svg.0), f64::from(svg.1));
        let (width, height) = size.unwrap_or((w, h));
        let scale = (width / w).min(height / h);
        Self {
            scale,
            left: (width - w * scale) / 2.0,
            top: (height - h * scale) / 2.0,
            width,
            height,
        }
    }

    fn to_svg(self, x: f64, y: f64) -> (f32, f32) {
        (
            ((x - self.left) / self.scale) as f32,
            ((y - self.top) / self.scale) as f32,
        )
    }

    /// Where the tooltip goes: its own share across at the point's share across, so it
    /// never leaves the plot's sides; below a point in the top half, above one lower.
    /// Always by `top`: a `bottom` mid-transition with it would squeeze the box (3462).
    fn tip_style(self, (x, y): (f32, f32)) -> String {
        let left = (self.left + f64::from(x) * self.scale) / self.width * 100.0;
        let top = self.top + f64::from(y) * self.scale;
        let left = left.clamp(0.0, 100.0);
        let (top, lift) = match top <= self.height / 2.0 {
            true => (top + TIP_GAP, 0),
            false => (top - TIP_GAP, 100),
        };
        format!("left:{left:.2}%;top:{top:.1}px;transform:translate(-{left:.2}%,-{lift}%)")
    }
}

/// The pointer layer, one tab stop whose arrows walk the points, the tooltip, the
/// crosshair and the active marker. The chart re-renders for none of them.
#[component]
pub(super) fn ChartOverlay(props: ChartOverlayProps) -> Element {
    let ChartOverlayProps {
        drawn,
        box_size,
        series,
        x,
        format,
        unit,
        formats,
        labels,
        title_id,
        keys_id,
        tooltip,
        tooltip_content,
        onpointclick,
        onactivechange,
        swap_axes,
        layout,
        zoom,
        onrange,
    } = props;
    let announcer = use_announcer();
    let layer = use_element();
    let mut gesture = use_signal(|| None::<Gesture>);
    // Touches down on the layer, by id, at their place along the x axis.
    let mut touches = use_signal(Vec::<(i32, f64)>::new);
    // A drag ended in this press: its click is no click.
    let mut dragged = use_signal(|| false);
    // The brush's band by x index, its anchor first.
    let mut selection = use_signal(|| None::<(usize, usize)>);
    // The keys moved the window under the active point: announce it once redrawn.
    let mut follow = use_signal(|| false);
    // The keys' pan says the point, not the range.
    let mut quiet = use_signal(|| false);
    let mut wheel_rest = use_signal(|| 0.0_f64);
    // Counts the point and selection announcements, so a zoom's late range skips a newer one.
    let spoke = use_hook(|| CopyValue::new(Cell::new(0_u32)));
    let showing = {
        let x = x.clone();
        use_debounced_callback(
            move |(start, end, spoke_then): (usize, usize, u32)| {
                let (Some(first), Some(last)) = (x.get(start), x.get(end.saturating_sub(1))) else {
                    return;
                };
                if spoke.read().get() != spoke_then {
                    return;
                }
                announcer.say(fill(labels.showing, &[("first", first), ("last", last)]));
            },
            SHOWING_DELAY,
        )
    };
    // A zoom or a pan reads out what the plot shows, once it settles; not the first draw.
    let mut seen = use_signal(|| None::<(usize, usize)>);
    use_effect(move || {
        let Some(zoom) = zoom else { return };
        let range = zoom.range();
        let before = *seen.peek();
        seen.set(Some(range));
        let hush = *quiet.peek();
        if hush {
            quiet.set(false);
        }
        if before.is_some_and(|before| before != range) && !hush {
            showing.call((range.0, range.1, spoke.read().get()));
        }
    });
    // By series and x index: a redraw keeps the point while it is drawn.
    let mut active = use_signal(|| None::<(usize, usize)>);
    // A tap pinned the point; a second tap on it is the click.
    let mut pinned = use_signal(|| false);
    let mut tap_on_active = use_signal(|| false);
    // Escape dismissed the tooltip: a hovering pointer shows none until it leaves.
    let mut muted = use_signal(|| false);

    let write = {
        let (series, unit) = (series.clone(), unit.clone());
        move |mark: &Mark| {
            let entry = entry(&series, mark);
            if let Some(own) = entry.and_then(|series| series.points.get(mark.index)) {
                return own.clone();
            }
            let unit = entry
                .and_then(|series| series.unit.as_deref())
                .or(unit.as_deref());
            let value = format.format(mark.value, unit, formats);
            let range = entry.and_then(|series| series.errors.get(mark.index).copied().flatten());
            match range {
                Some((low, high)) => format!(
                    "{value} ({}–{})",
                    format.format(low, None, formats),
                    format.format(high, unit, formats)
                ),
                None => value,
            }
        }
    };
    let point_of = {
        let (series, x) = (series.clone(), x.clone());
        move |mark: &Mark| ChartPoint {
            series: mark.series,
            series_name: entry(&series, mark)
                .map(|series| series.name.clone())
                .unwrap_or_default(),
            index: mark.index,
            x: match layout {
                Layout::Free => entry(&series, mark).and_then(|series| series.xs.get(mark.index)),
                _ => x.get(mark.index),
            }
            .cloned()
            .unwrap_or_default(),
            value: mark.value,
        }
    };
    let with_marks = move |read: &dyn Fn(&Geometry) -> Option<Mark>| {
        drawn
            .peek()
            .as_ref()
            .and_then(|drawn| read(&drawn.geometry))
    };
    let slice_share = move |mark: &Mark| {
        drawn
            .peek()
            .as_ref()
            .map(|drawn| {
                share(
                    &drawn.geometry.marks,
                    drawn.geometry.share_of,
                    mark,
                    formats,
                )
            })
            .unwrap_or_default()
    };
    let of_first = move || {
        drawn
            .peek()
            .as_ref()
            .and_then(|drawn| drawn.geometry.share_of)
    };
    let mut set_active = {
        let point_of = point_of.clone();
        move |next: Option<&Mark>| {
            let key = next.map(|mark| (mark.series, mark.index));
            if *active.peek() == key {
                return;
            }
            active.set(key);
            if let Some(onactivechange) = onactivechange {
                onactivechange.call(next.map(&point_of));
            }
        }
    };
    // Escape from anywhere on the page, as a hover tooltip takes no focus (WCAG 1.4.13).
    let dismiss = use_callback({
        let mut set_active = set_active.clone();
        move |()| {
            muted.set(true);
            pinned.set(false);
            set_active(None);
            announcer.clear();
        }
    });
    let _ = use_escape_dismiss(tooltip && active.read().is_some(), true, dismiss);
    let at_pointer = move |point: ElementPoint| {
        with_marks(&|geometry| {
            let fit = Fit::of((geometry.width, geometry.height), *box_size.peek());
            let (x, y) = fit.to_svg(point.x, point.y);
            let found = match layout {
                Layout::Columns => hit::hit(&geometry.marks, x, y),
                _ => hit::nearest(&geometry.marks, x, y),
            };
            found.map(|i| geometry.marks[i].clone())
        })
    };
    let click = {
        let point_of = point_of.clone();
        move |mark: Option<Mark>| {
            if let (Some(onpointclick), Some(mark)) = (onpointclick, mark) {
                onpointclick.call(point_of(&mark));
            }
        }
    };
    let say_point = {
        let point_of = point_of.clone();
        let write = write.clone();
        let series = series.clone();
        move |mark: &Mark| {
            let point = point_of(mark);
            let value = write(mark);
            let own = entry(&series, mark).and_then(|series| series.points.get(mark.index));
            spoke.read().set(spoke.read().get() + 1);
            announcer.say(match (layout, own, &mark.shape) {
                (Layout::Free, Some(own), _) => fill(
                    labels.point_free,
                    &[("name", &point.series_name), ("value", own)],
                ),
                (.., MarkShape::Sector { .. }) if of_first().is_some_and(|first| first <= 0.0) => {
                    fill(
                        labels.stage_value,
                        &[("name", &point.series_name), ("value", &value)],
                    )
                }
                (.., MarkShape::Sector { .. }) if of_first().is_some() => fill(
                    labels.stage,
                    &[
                        ("name", &point.series_name),
                        ("value", &value),
                        ("share", &slice_share(mark)),
                    ],
                ),
                (.., MarkShape::Sector { .. }) => fill(
                    labels.slice,
                    &[
                        ("name", &point.series_name),
                        ("value", &value),
                        ("share", &slice_share(mark)),
                    ],
                ),
                _ => fill(
                    labels.point,
                    &[
                        ("name", &point.series_name),
                        ("x", &point.x),
                        ("value", &value),
                    ],
                ),
            });
        }
    };
    // Once the keys' pan is drawn: the active point, or its series' nearest one past a gap.
    {
        let mut set_active = set_active.clone();
        let say_point = say_point.clone();
        use_effect(move || {
            let drawn = drawn.read();
            if !*follow.peek() {
                return;
            }
            follow.set(false);
            let Some((series, index)) = *active.peek() else {
                return;
            };
            let Some(mark) = drawn.as_ref().and_then(|drawn| {
                drawn
                    .geometry
                    .marks
                    .iter()
                    .filter(|mark| mark.series == series)
                    .min_by_key(|mark| mark.index.abs_diff(index))
                    .cloned()
            }) else {
                return;
            };
            // Keyed already, so `set_active` would skip the change.
            active.set(None);
            set_active(Some(&mark));
            say_point(&mark);
        });
    }
    let along = move |point: ElementPoint| if swap_axes { point.y } else { point.x };
    // The plot's x axis in the layer's px: where it starts, and its length.
    let axis_px = move || -> Option<(f64, f64)> {
        let drawn = drawn.peek();
        let geometry = &drawn.as_ref()?.geometry;
        let fit = Fit::of((geometry.width, geometry.height), *box_size.peek());
        let plot = geometry.plot;
        let (offset, from, to) = match swap_axes {
            true => (fit.top, plot.top, plot.bottom),
            false => (fit.left, plot.left, plot.right),
        };
        Some((
            offset + f64::from(from) * fit.scale,
            f64::from(to - from) * fit.scale,
        ))
    };
    let share_at = move |px: f64| {
        axis_px().map_or(0.5, |(start, length)| {
            ((px - start) / length.max(1.0)).clamp(0.0, 1.0)
        })
    };
    let index_under = move |px: f64| -> Option<usize> {
        let drawn = drawn.peek();
        let geometry = &drawn.as_ref()?.geometry;
        let fit = Fit::of((geometry.width, geometry.height), *box_size.peek());
        let offset = if swap_axes { fit.top } else { fit.left };
        index_at(
            &geometry.marks,
            ((px - offset) / fit.scale) as f32,
            swap_axes,
        )
    };
    // Px per x value along the axis, in `window`.
    let step_px = move |window: (usize, usize)| {
        axis_px().map(|(_, length)| length / (window.1 - window.0).max(1) as f64)
    };
    let zoomed_now = move || zoom.is_some_and(|zoom| zoom.peek() != (0, zoom.len));
    let gestures = zoom.is_some() || onrange.is_some();
    let range_of = {
        let x = x.clone();
        move |(a, b): (usize, usize)| ChartRange::of(a, b, &x)
    };
    let select = {
        let range_of = range_of.clone();
        move |band: (usize, usize)| {
            let range = range_of(band);
            spoke.read().set(spoke.read().get() + 1);
            announcer.say(fill(
                labels.selected,
                &[("first", &range.from), ("last", &range.to)],
            ));
            if let Some(onrange) = onrange {
                onrange.call(Some(range));
            }
        }
    };
    let active_mark = move || {
        let key = (*active.peek())?;
        with_marks(&|geometry| {
            geometry
                .marks
                .iter()
                .find(|mark| (mark.series, mark.index) == key)
                .cloned()
        })
    };

    let onpointermove = {
        let mut set_active = set_active.clone();
        move |event: Event<PointerData>| {
            if gestures {
                let point = event.element_coordinates();
                let id = event.pointer_id();
                if let Some(touch) = touches.write().iter_mut().find(|touch| touch.0 == id) {
                    touch.1 = along(point);
                }
                let current = *gesture.peek();
                match current {
                    Some(Gesture::Press {
                        id: pressed,
                        x,
                        y,
                        touch,
                        pan,
                    }) if pressed == id => {
                        let (dx, dy) = (point.x - x, point.y - y);
                        let (sideways, across) = if swap_axes { (dy, dx) } else { (dx, dy) };
                        let drags = match touch {
                            true => sideways.abs() >= TOUCH_SLOP && sideways.abs() > across.abs(),
                            false => dx.hypot(dy) >= MOUSE_SLOP,
                        };
                        if !drags {
                            // A swipe across the axis is the page's scroll.
                            if touch && across.abs() >= TOUCH_SLOP {
                                gesture.set(None);
                            }
                            return;
                        }
                        let from = if swap_axes { y } else { x };
                        let next = match (pan, zoom) {
                            (true, Some(zoom)) => Some(Gesture::Pan {
                                id,
                                from,
                                window: zoom.peek(),
                            }),
                            _ => index_under(from).map(|anchor| {
                                selection.set(Some((anchor, anchor)));
                                Gesture::Brush { id, anchor }
                            }),
                        };
                        gesture.set(next);
                        if next.is_some() {
                            dragged.set(true);
                            pinned.set(false);
                            set_active(None);
                            let _ = layer.set_pointer_capture(id);
                        }
                        return;
                    }
                    Some(Gesture::Pan {
                        id: panning,
                        from,
                        window,
                    }) if panning == id => {
                        if let (Some(zoom), Some(step)) = (zoom, step_px(window)) {
                            let by = (-(along(point) - from) / step).round() as isize;
                            zoom.set(panned(window, zoom.len, by));
                        }
                        return;
                    }
                    Some(Gesture::Brush {
                        id: brushing,
                        anchor,
                    }) if brushing == id => {
                        if let Some(head) = index_under(along(point))
                            && *selection.peek() != Some((anchor, head))
                        {
                            selection.set(Some((anchor, head)));
                        }
                        return;
                    }
                    Some(Gesture::Pinch { spread, at, window }) => {
                        if let (Some(zoom), [a, b]) = (zoom, touches.peek().as_slice()) {
                            let now = (a.1 - b.1).abs().max(1.0);
                            zoom.set(zoomed(window, zoom.len, spread / now, at));
                        }
                        return;
                    }
                    _ => {}
                }
            }
            if *muted.peek() || event.pointer_type() == "touch" && !*pinned.peek() {
                return;
            }
            set_active(at_pointer(event.element_coordinates()).as_ref());
        }
    };
    let onpointerdown = {
        let mut set_active = set_active.clone();
        move |event: Event<PointerData>| {
            dragged.set(false);
            if gestures {
                let point = event.element_coordinates();
                let touch = event.pointer_type() == "touch";
                if touch {
                    touches.write().push((event.pointer_id(), along(point)));
                }
                if let (Some(zoom), [a, b]) = (zoom, touches.peek().as_slice()) {
                    gesture.set(Some(Gesture::Pinch {
                        spread: (a.1 - b.1).abs().max(1.0),
                        at: share_at((a.1 + b.1) / 2.0),
                        window: zoom.peek(),
                    }));
                    dragged.set(true);
                    pinned.set(false);
                    set_active(None);
                    return;
                }
                let primary = !matches!(event.trigger_button(), Some(button) if button != MouseButton::Primary);
                let modifiers = event.modifiers();
                // With a brush, Shift or Alt pans instead.
                let pan =
                    zoomed_now() && (onrange.is_none() || modifiers.shift() || modifiers.alt());
                if primary && (pan || onrange.is_some()) {
                    gesture.set(Some(Gesture::Press {
                        id: event.pointer_id(),
                        x: point.x,
                        y: point.y,
                        touch,
                        pan,
                    }));
                }
            }
            if event.pointer_type() == "mouse" {
                pinned.set(false);
                return;
            }
            let mark = at_pointer(event.element_coordinates());
            let key = mark.as_ref().map(|mark| (mark.series, mark.index));
            tap_on_active.set(*pinned.peek() && key.is_some() && key == *active.peek());
            pinned.set(mark.is_some());
            set_active(mark.as_ref());
        }
    };
    let mut release = {
        let select = select.clone();
        move |event: Event<PointerData>| {
            let id = event.pointer_id();
            touches.write().retain(|touch| touch.0 != id);
            let current = *gesture.peek();
            match current {
                Some(Gesture::Brush { id: brushing, .. }) if brushing == id => {
                    gesture.set(None);
                    if let Some(band) = *selection.peek() {
                        select(band);
                    }
                }
                Some(Gesture::Press { id: pressed, .. } | Gesture::Pan { id: pressed, .. })
                    if pressed == id =>
                {
                    gesture.set(None)
                }
                Some(Gesture::Pinch { .. }) => gesture.set(None),
                _ => {}
            }
        }
    };
    let onpointerup = {
        let mut release = release.clone();
        move |event: Event<PointerData>| release(event)
    };
    let onpointercancel = move |event: Event<PointerData>| release(event);
    // Ctrl or Cmd with the wheel, and a trackpad's pinch, zoom; a sideways or Shift wheel pans.
    let onwheel = move |event: Event<WheelData>| {
        let Some(zoom) = zoom else {
            return;
        };
        let data = event.data();
        let modifiers = event.modifiers();
        if modifiers.ctrl() || modifiers.meta() {
            event.prevent_default();
            let travel = wheel_travel_y(&data, 40.0, 10.0);
            let at = share_at(along(event.element_coordinates()));
            zoom.zoom((travel / WHEEL_ZOOM).exp(), at);
            return;
        }
        let (dx, dy) = match data.delta() {
            WheelDelta::Pixels(delta) => (delta.x, delta.y),
            WheelDelta::Lines(delta) => (delta.x * 40.0, delta.y * 40.0),
            WheelDelta::Pages(delta) => (delta.x * 400.0, delta.y * 400.0),
        };
        let travel = match modifiers.shift() {
            true if dx != 0.0 => dx,
            true => dy,
            false if dx.abs() > dy.abs() => dx,
            false => return,
        };
        if !zoomed_now() {
            return;
        }
        event.prevent_default();
        let Some(step) = step_px(zoom.peek()) else {
            return;
        };
        let rest = *wheel_rest.peek() + travel / step;
        let by = rest.trunc();
        wheel_rest.set(rest - by);
        if by != 0.0 {
            zoom.pan(by as isize);
        }
    };
    let onpointerleave = {
        let mut set_active = set_active.clone();
        move |event: Event<PointerData>| {
            // A touch leaves at every lift: its point stays pinned. A pen hovers, as a mouse.
            if event.pointer_type() != "touch" {
                muted.set(false);
                pinned.set(false);
                set_active(None);
            }
        }
    };
    let onclick = {
        let click = click.clone();
        let mut set_active = set_active.clone();
        // The point under the click: a press without a move first sets none active.
        move |event: Event<MouseData>| {
            if *dragged.peek() {
                dragged.set(false);
                return;
            }
            if *tap_on_active.peek() || !*pinned.peek() {
                let mark = at_pointer(event.element_coordinates());
                set_active(mark.as_ref());
                click(mark);
            }
        }
    };
    let onkeydown = {
        let mut set_active = set_active.clone();
        let say_point = say_point.clone();
        let select = select.clone();
        move |event: Event<KeyboardData>| {
            if has_shortcut_modifier(&event) {
                return;
            }
            if let Some(zoom) = zoom {
                let (start, end) = zoom.peek();
                // Around the keys' point, else the middle.
                let at = active
                    .peek()
                    .filter(|(_, index)| (start..end).contains(index))
                    .map_or(0.5, |(_, index)| {
                        (index - start) as f64 / (end - start).saturating_sub(1).max(1) as f64
                    });
                let factor = match event.key() {
                    Key::Character(ref c) if c == "+" || c == "=" => Some(0.5),
                    Key::Character(ref c) if c == "-" || c == "_" => Some(2.0),
                    Key::Character(ref c) if c == "0" => Some(f64::INFINITY),
                    _ => None,
                };
                if let Some(factor) = factor {
                    event.prevent_default();
                    match factor.is_finite() {
                        true => zoom.zoom(factor, at),
                        false => zoom.reset(),
                    }
                    return;
                }
            }
            if event.key() == Key::Named(NamedKey::Escape)
                && onrange.is_some()
                && selection.peek().is_some()
            {
                event.stop_propagation();
                selection.set(None);
                announcer.say(labels.selection_cleared.to_string());
                if let Some(onrange) = onrange {
                    onrange.call(None);
                }
                return;
            }
            let step = match event.key() {
                Key::Named(NamedKey::ArrowRight) => Step::Next,
                Key::Named(NamedKey::ArrowLeft) => Step::Previous,
                Key::Named(NamedKey::ArrowDown) => Step::Down,
                Key::Named(NamedKey::ArrowUp) => Step::Up,
                Key::Named(NamedKey::Home) => Step::First,
                Key::Named(NamedKey::End) => Step::Last,
                // Unprevented, Android's WebView follows Enter with a click at the layer's centre.
                Key::Named(NamedKey::Enter) => {
                    event.prevent_default();
                    click(active_mark());
                    return;
                }
                Key::Character(ref c) if c == " " => {
                    event.prevent_default();
                    click(active_mark());
                    return;
                }
                Key::Named(NamedKey::Escape) if active.peek().is_some() => {
                    event.stop_propagation();
                    pinned.set(false);
                    set_active(None);
                    announcer.clear();
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            let step = if swap_axes { step.swapped() } else { step };
            let key = *active.peek();
            let from = active_mark();
            let Some(mark) = with_marks(&|geometry| {
                let current = geometry
                    .marks
                    .iter()
                    .position(|mark| Some((mark.series, mark.index)) == key);
                let next = match layout {
                    Layout::Free => hit::walk_free(&geometry.marks, current, step),
                    _ => hit::walk(&geometry.marks, current, step),
                };
                next.map(|next| geometry.marks[next].clone())
            }) else {
                return;
            };
            let along = matches!(step, Step::Next | Step::Previous);
            // Shift with an arrow drags the brush's head from where the keys stood.
            let mut brush = |head: usize| {
                let anchor = (*selection.peek())
                    .map(|(anchor, _)| anchor)
                    .or(from.as_ref().map(|from| from.index))
                    .unwrap_or(head);
                selection.set(Some((anchor, head)));
                (anchor, head)
            };
            let extend = onrange.is_some() && along && event.modifiers().shift();
            // At the window's edge the window moves on with the keys.
            if let (Some(zoom), Some(from)) = (zoom, from.as_ref())
                && along
                && mark == *from
            {
                let target = match step {
                    Step::Next => from.index + 1,
                    _ => from.index.wrapping_sub(1),
                };
                if target < zoom.len && !(zoom.peek().0..zoom.peek().1).contains(&target) {
                    quiet.set(true);
                    follow.set(true);
                    active.set(Some((from.series, target)));
                    zoom.reveal(target);
                    if extend {
                        select(brush(target));
                    }
                    return;
                }
            }
            set_active(Some(&mark));
            if extend {
                select(brush(mark.index));
            } else {
                say_point(&mark);
            }
        }
    };
    let onfocusout = move |_: Event<FocusData>| {
        pinned.set(false);
        set_active(None);
    };

    let drawn = drawn.read();
    let geometry = drawn.as_ref().map(|drawn| &drawn.geometry);
    let fit = geometry.map(|geometry| Fit::of((geometry.width, geometry.height), *box_size.read()));
    let key = *active.read();
    let current = geometry.and_then(|geometry| {
        geometry
            .marks
            .iter()
            .find(|mark| Some((mark.series, mark.index)) == key)
    });
    let color = |mark: &Mark| {
        entry(&series, mark)
            .map(|series| series.color.clone())
            .unwrap_or_default()
    };
    let view_box = geometry
        .map(|geometry| format!("0 0 {} {}", geometry.width, geometry.height))
        .unwrap_or_default();
    let plot = geometry.map(|g| g.plot).unwrap_or(Plot::whole(0.0, 0.0));
    let marker = current.map(|mark| match &mark.shape {
        MarkShape::Point { x, y } => {
            let Plot { left, top, right, bottom } = plot;
            // A scatter's point is read on both axes; a radar's spoke is drawn already.
            rsx! {
                if tooltip && layout != Layout::Spokes {
                    line { "data-chart-crosshair": true, x1: "{x}", x2: "{x}", y1: "{top}", y2: "{bottom}" }
                }
                if tooltip && layout == Layout::Free {
                    line { "data-chart-crosshair": true, x1: "{left}", x2: "{right}", y1: "{y}", y2: "{y}" }
                }
                circle { "data-chart-dot": true, cx: "{x}", cy: "{y}", r: "5", fill: color(mark), "stroke-width": "2" }
            }
        }
        MarkShape::Rect { x, y, width, height } => {
            // Within the bar and the plot, its stroke too: it never covers an axis.
            let left = x.max(plot.left) + MARKER_INSET;
            let top = y.max(plot.top) + MARKER_INSET;
            let right = (x + width).min(plot.right) - MARKER_INSET;
            let bottom = (y + height).min(plot.bottom) - MARKER_INSET;
            let (width, height) = ((right - left).max(0.0), (bottom - top).max(0.0));
            rsx! {
                rect { "data-chart-marker": true, x: "{left}", y: "{top}", width: "{width}", height: "{height}", fill: "none" }
            }
        }
        MarkShape::Sector { path } => rsx! {
            path { "data-chart-marker": true, d: "{path}", fill: "none" }
        },
    });
    let band = (*selection.read()).and_then(|(a, b)| {
        let geometry = geometry?;
        let (low, high) = (a.min(b), a.max(b));
        let (start, end) = zoom.map_or((0, usize::MAX), |zoom| zoom.range());
        if high < start || low >= end {
            return None;
        }
        let (edge_from, edge_to) = match swap_axes {
            true => (plot.top, plot.bottom),
            false => (plot.left, plot.right),
        };
        // An end out of the window runs to the plot's edge.
        let from = match low < start {
            true => edge_from,
            false => index_span(&geometry.marks, low, swap_axes)?.0,
        };
        let to = match high >= end {
            true => edge_to,
            false => index_span(&geometry.marks, high, swap_axes)?.1,
        };
        let middle = (from + to) / 2.0;
        let (from, to) = match to - from < 2.0 * BRUSH_MIN {
            true => (middle - BRUSH_MIN, middle + BRUSH_MIN),
            false => (from, to),
        };
        let (from, to) = (from.max(edge_from), to.min(edge_to));
        Some(match swap_axes {
            true => (plot.left, from, plot.right - plot.left, to - from),
            false => (from, plot.top, to - from, plot.bottom - plot.top),
        })
    });
    let tip = match (tooltip, current, fit) {
        (true, Some(mark), Some(fit)) => {
            let style = fit.tip_style(hit::anchor(&mark.shape));
            let body = match tooltip_content {
                Some(content) => content.call(point_of(mark)),
                None => tip_body(mark, geometry, &series, &x, &write, formats, layout),
            };
            Some((style, body))
        }
        _ => None,
    };

    rsx! {
        svg {
            "data-chart-marks": true,
            "aria-hidden": "true",
            "focusable": "false",
            width: "100%",
            height: "100%",
            view_box,
            if let Some((x, y, width, height)) = band {
                rect { "data-chart-brush": true, x: "{x}", y: "{y}", width: "{width}", height: "{height}" }
            }
            {marker}
        }
        if let Some((style, body)) = tip {
            // The keys announce the point: the box is for the eye.
            div { "data-slot": ChartPart::Tooltip.slot(), "aria-hidden": "true", style, {body} }
        }
        div {
            "data-chart-layer": true,
            // The plot owns its arrows: screen readers pass them on only to an application.
            role: "application",
            "aria-roledescription": labels.chart,
            // An application needs a name: without a title, the chart label.
            "aria-label": title_id.is_none().then_some(labels.chart),
            "aria-labelledby": title_id,
            "aria-describedby": keys_id.clone(),
            tabindex: "0",
            "data-gestures": gestures.then_some("true"),
            onmounted: layer.mount(),
            onpointermove,
            onpointerdown,
            onpointerup,
            onpointercancel,
            onwheel,
            onpointerleave,
            onclick,
            onkeydown,
            onfocusout,
        }
        div { id: keys_id, hidden: true,
            "{labels.keys}"
            if zoom.is_some() {
                " {labels.zoom_keys}"
            }
            if onrange.is_some() {
                " {labels.brush_keys}"
            }
        }
        {announcer.render()}
    }
}

/// The x value, then each drawn series' value there (a heatmap's cell alone); a pie's
/// slice alone, with its share.
fn tip_body(
    active: &Mark,
    geometry: Option<&Geometry>,
    series: &[OverlaySeries],
    x: &[String],
    write: &impl Fn(&Mark) -> String,
    formats: &Formats,
    layout: Layout,
) -> Element {
    let marks = geometry.map_or(&[][..], |g| &g.marks[..]);
    let (share_of, cells) = (
        geometry.and_then(|g| g.share_of),
        geometry.is_some_and(|g| g.cells),
    );
    let name = |mark: &Mark| {
        entry(series, mark)
            .map(|series| (series.name.clone(), series.color.clone()))
            .unwrap_or_default()
    };
    let own = entry(series, active).and_then(|series| series.points.get(active.index));
    let rows: Vec<(bool, String, String, String)> = match (layout, own, &active.shape) {
        // A scatter's point alone: no other point shares its x.
        (Layout::Free, Some(own), _) => {
            let (name, color) = name(active);
            vec![(true, name, color, own.clone())]
        }
        (.., MarkShape::Sector { .. }) => {
            let (name, color) = name(active);
            let value = match share_of {
                Some(first) if first <= 0.0 => write(active),
                _ => format!(
                    "{} ({})",
                    write(active),
                    share(marks, share_of, active, formats)
                ),
            };
            vec![(true, name, color, value)]
        }
        _ => marks
            .iter()
            .filter(|mark| match cells {
                true => *mark == active,
                false => mark.index == active.index,
            })
            .map(|mark| {
                let (name, color) = name(mark);
                (mark == active, name, color, write(mark))
            })
            .collect(),
    };
    let heading = match (layout, &active.shape) {
        (Layout::Free, _) | (_, MarkShape::Sector { .. }) => None,
        _ => x.get(active.index).cloned(),
    };
    rsx! {
        if let Some(heading) = heading {
            div { "{heading}" }
        }
        for (i, (is_active, name, color, value)) in rows.into_iter().enumerate() {
            div { key: "{i}", "data-chart-row": true, "data-active": is_active.then_some("true"),
                span { "data-chart-swatch": true, style: "--lsx-chart-swatch:{color}" }
                span { "{name}" }
                span { "{value}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_wider_than_the_svg_centres_it() {
        let fit = Fit::of((100.0, 50.0), Some((300.0, 100.0)));
        assert_eq!(fit.scale, 2.0);
        assert_eq!(fit.to_svg(150.0, 50.0), (50.0, 25.0));
    }

    #[test]
    fn a_tooltip_hugs_its_side_and_flips_below_the_middle() {
        let fit = Fit::of((200.0, 100.0), None);
        assert_eq!(
            fit.tip_style((0.0, 10.0)),
            "left:0.00%;top:22.0px;transform:translate(-0.00%,-0%)"
        );
        assert_eq!(
            fit.tip_style((150.0, 80.0)),
            "left:75.00%;top:68.0px;transform:translate(-75.00%,-100%)"
        );
    }
}
