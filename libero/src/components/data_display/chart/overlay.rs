//! The interactive layer over a drawn chart: pointer, keys, tooltip, crosshair, marker.
//! It reads only the harvested marks, so every chart kind shares it.

use dioxus::{html::geometry::ElementPoint, prelude::*};

use super::{
    ChartPart, ChartPoint, ValueFormat,
    engine::{Drawn, Geometry, Mark, MarkShape},
    hit::{self, Step},
};
use crate::{
    components::{
        accessibility::use_announcer,
        common::{Part, focus_ring_sx},
    },
    localization::{ChartLabels, Formats, fill},
    sx::{REDUCED_MOTION, Sx, sx},
    theme::{Size, SizeCss, TOOLTIP_BACKGROUND, TOOLTIP_COLOR},
    tokens::NamedColorCss,
};

/// Room between a point and its tooltip, in px.
const TIP_GAP: f64 = 12.0;

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
                .font_size("sm")
                .white_space("nowrap")
                .with(
                    "transition",
                    "left 120ms ease-out, top 120ms ease-out, bottom 120ms ease-out, transform 120ms ease-out",
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
                .with("padding-inline-start", "12px"),
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
    /// The series' own unit; unset, the chart's.
    pub(super) unit: Option<String>,
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
}

/// The series a mark belongs to; a pie's slices are its entries, one per index.
fn entry<'a>(series: &'a [OverlaySeries], mark: &Mark) -> Option<&'a OverlaySeries> {
    match mark.shape {
        MarkShape::Sector { .. } => series.get(mark.index),
        _ => series.get(mark.series),
    }
}

/// A slice's part of the drawn ones, as the pie's labels write it.
fn share(marks: &[Mark], mark: &Mark, formats: &Formats) -> String {
    let total: f64 = marks.iter().map(|mark| mark.value).sum();
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
    fn tip_style(self, (x, y): (f32, f32)) -> String {
        let left = (self.left + f64::from(x) * self.scale) / self.width * 100.0;
        let top = self.top + f64::from(y) * self.scale;
        let left = left.clamp(0.0, 100.0);
        let vertical = match top <= self.height / 2.0 {
            true => format!("top:{:.1}px", top + TIP_GAP),
            false => format!("bottom:{:.1}px", self.height - top + TIP_GAP),
        };
        format!("left:{left:.2}%;transform:translateX(-{left:.2}%);{vertical}")
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
    } = props;
    let announcer = use_announcer();
    // By series and x index: a redraw keeps the point while it is drawn.
    let mut active = use_signal(|| None::<(usize, usize)>);
    // A tap pinned the point; a second tap on it is the click.
    let mut pinned = use_signal(|| false);
    let mut tap_on_active = use_signal(|| false);

    let write = {
        let (series, unit) = (series.clone(), unit.clone());
        move |mark: &Mark| {
            let own = entry(&series, mark).and_then(|series| series.unit.as_deref());
            format.format(mark.value, own.or(unit.as_deref()), formats)
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
            x: x.get(mark.index).cloned().unwrap_or_default(),
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
            .map(|drawn| share(&drawn.geometry.marks, mark, formats))
            .unwrap_or_default()
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
    let at_pointer = move |point: ElementPoint| {
        with_marks(&|geometry| {
            let fit = Fit::of((geometry.width, geometry.height), *box_size.peek());
            let (x, y) = fit.to_svg(point.x, point.y);
            hit::hit(&geometry.marks, x, y).map(|i| geometry.marks[i].clone())
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
            if event.pointer_type() == "touch" && !*pinned.peek() {
                return;
            }
            set_active(at_pointer(event.element_coordinates()).as_ref());
        }
    };
    let onpointerdown = {
        let mut set_active = set_active.clone();
        move |event: Event<PointerData>| {
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
    let onpointerleave = {
        let mut set_active = set_active.clone();
        move |event: Event<PointerData>| {
            // A touch leaves at every lift: its point stays pinned.
            if event.pointer_type() == "mouse" {
                set_active(None);
            }
        }
    };
    let onclick = {
        let click = click.clone();
        let mut set_active = set_active.clone();
        // The point under the click: a press without a move first sets none active.
        move |event: Event<MouseData>| {
            if *tap_on_active.peek() || !*pinned.peek() {
                let mark = at_pointer(event.element_coordinates());
                set_active(mark.as_ref());
                click(mark);
            }
        }
    };
    let onkeydown = {
        let mut set_active = set_active.clone();
        let point_of = point_of.clone();
        let write = write.clone();
        move |event: Event<KeyboardData>| {
            let step = match event.key() {
                Key::ArrowRight => Step::Next,
                Key::ArrowLeft => Step::Previous,
                Key::ArrowDown => Step::Down,
                Key::ArrowUp => Step::Up,
                Key::Home => Step::First,
                Key::End => Step::Last,
                // Unprevented, Android's WebView follows Enter with a click at the layer's centre.
                Key::Enter => {
                    event.prevent_default();
                    click(active_mark());
                    return;
                }
                Key::Character(ref c) if c == " " => {
                    event.prevent_default();
                    click(active_mark());
                    return;
                }
                Key::Escape if active.peek().is_some() => {
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
            let Some(mark) = with_marks(&|geometry| {
                let current = geometry
                    .marks
                    .iter()
                    .position(|mark| Some((mark.series, mark.index)) == key);
                hit::walk(&geometry.marks, current, step).map(|next| geometry.marks[next].clone())
            }) else {
                return;
            };
            set_active(Some(&mark));
            let point = point_of(&mark);
            let value = write(&mark);
            announcer.say(match mark.shape {
                MarkShape::Sector { .. } => fill(
                    labels.slice,
                    &[
                        ("name", &point.series_name),
                        ("value", &value),
                        ("share", &slice_share(&mark)),
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
    let marker = current.map(|mark| match &mark.shape {
        MarkShape::Point { x, y } => {
            let bottom = geometry.map(|g| g.height).unwrap_or_default();
            rsx! {
                if tooltip {
                    line { "data-chart-crosshair": true, x1: "{x}", x2: "{x}", y1: "0", y2: "{bottom}" }
                }
                circle { "data-chart-dot": true, cx: "{x}", cy: "{y}", r: "5", fill: color(mark), "stroke-width": "2" }
            }
        }
        MarkShape::Rect { x, y, width, height } => rsx! {
            rect { "data-chart-marker": true, x: "{x}", y: "{y}", width: "{width}", height: "{height}", fill: "none" }
        },
        MarkShape::Sector { path } => rsx! {
            path { "data-chart-marker": true, d: "{path}", fill: "none" }
        },
    });
    let tip = match (tooltip, current, fit) {
        (true, Some(mark), Some(fit)) => {
            let style = fit.tip_style(hit::anchor(&mark.shape));
            let body = match tooltip_content {
                Some(content) => content.call(point_of(mark)),
                None => tip_body(
                    mark,
                    geometry.map_or(&[][..], |g| &g.marks[..]),
                    &series,
                    &x,
                    &write,
                    formats,
                ),
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
            "aria-labelledby": title_id,
            "aria-describedby": keys_id.clone(),
            tabindex: "0",
            onpointermove,
            onpointerdown,
            onpointerleave,
            onclick,
            onkeydown,
            onfocusout,
        }
        div { id: keys_id, hidden: true, "{labels.keys}" }
        {announcer.render()}
    }
}

/// The x value, then each drawn series' value there; a pie's slice alone, with its share.
fn tip_body(
    active: &Mark,
    marks: &[Mark],
    series: &[OverlaySeries],
    x: &[String],
    write: &impl Fn(&Mark) -> String,
    formats: &Formats,
) -> Element {
    let name = |mark: &Mark| {
        entry(series, mark)
            .map(|series| (series.name.clone(), series.color.clone()))
            .unwrap_or_default()
    };
    let rows: Vec<(bool, String, String, String)> = match active.shape {
        MarkShape::Sector { .. } => {
            let (name, color) = name(active);
            let value = format!("{} ({})", write(active), share(marks, active, formats));
            vec![(true, name, color, value)]
        }
        _ => marks
            .iter()
            .filter(|mark| mark.index == active.index)
            .map(|mark| {
                let (name, color) = name(mark);
                (mark == active, name, color, write(mark))
            })
            .collect(),
    };
    let heading = match active.shape {
        MarkShape::Sector { .. } => None,
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
            "left:0.00%;transform:translateX(-0.00%);top:22.0px"
        );
        assert_eq!(
            fit.tip_style((150.0, 80.0)),
            "left:75.00%;transform:translateX(-75.00%);bottom:32.0px"
        );
    }
}
