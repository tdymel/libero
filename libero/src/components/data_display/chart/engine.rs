//! Props to `charts-rs` and back: it draws the static SVG, we harvest where each mark
//! landed and strip its own tooltips, so the overlay and keyboard layers can be ours.

use charts_rs::{AxisType, Color as InkColor, LineStep, Position, Symbol};

use super::data::{ValueFormat, ungrouped};
use crate::{
    localization::Formats,
    theme::{BarLayout, ChartInk, Curve, HexColor, PieLabels},
};

/// Ahead of each y-axis label, so [`sanitize`] can write it again in the locale.
const AXIS_MARK: char = '\u{2060}';
/// The same for the labels of a numeric x axis.
const X_MARK: char = '\u{2061}';
/// Ahead of each percentage the renderer writes, to write it again in the locale.
const PERCENT_MARK: char = '\u{2062}';
/// Ahead of each label on a bar: written as the y labels, but not pinned to the axis.
const LABEL_MARK: char = '\u{2063}';
const DASH: &str = "6,4";
/// An axis' tick is 5 long; an axis or grid line is longer.
const PLOT_TICK: f32 = 10.0;
/// The pie's radius over half the plot's shorter side, as the renderer caps it.
const PIE_FILL: f32 = 0.8;
/// A label on a bar is drawn in this, then given the ink that reads on its bar.
const ON_BAR: InkColor = InkColor {
    r: 1,
    g: 2,
    b: 3,
    a: 255,
};
const ON_BAR_FILL: &str = "fill=\"#010203\"";

/// Where every mark of a drawn chart sits, in the SVG's own units.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Geometry {
    pub(crate) width: f32,
    pub(crate) height: f32,
    /// The area inside the axes; a pie's is the whole SVG.
    pub(crate) plot: Plot,
    /// Series by series, in data order; a gap has no mark.
    pub(crate) marks: Vec<Mark>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plot {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
}

impl Plot {
    pub(crate) fn whole(width: f32, height: f32) -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: width,
            bottom: height,
        }
    }

    /// The box the axes and grid lines span; ticks are too short to count.
    fn of(svg: &str, width: f32, height: f32) -> Self {
        let mut found: Option<Self> = None;
        let mut rest = svg;
        while let Some(open) = rest.find("<line ") {
            let close = rest[open..]
                .find('>')
                .map_or(rest.len(), |close| open + close + 1);
            let attributes = attributes(&rest[open..close]);
            rest = &rest[close..];
            let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
            let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                (number("x1"), number("y1"), number("x2"), number("y2"))
            else {
                continue;
            };
            if (x2 - x1).hypot(y2 - y1) <= PLOT_TICK {
                continue;
            }
            let line = Self {
                left: x1.min(x2),
                top: y1.min(y2),
                right: x1.max(x2),
                bottom: y1.max(y2),
            };
            found = Some(match found {
                Some(plot) => Self {
                    left: plot.left.min(line.left),
                    top: plot.top.min(line.top),
                    right: plot.right.max(line.right),
                    bottom: plot.bottom.max(line.bottom),
                },
                None => line,
            });
        }
        found
            .filter(|plot| plot.right > plot.left && plot.bottom > plot.top)
            .unwrap_or(Self::whole(width, height))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Mark {
    pub(crate) series: usize,
    pub(crate) index: usize,
    pub(crate) value: f64,
    pub(crate) shape: MarkShape,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MarkShape {
    /// A line's point.
    Point { x: f32, y: f32 },
    /// A bar.
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// A pie slice, as an SVG path.
    Sector { path: String },
}

/// A drawn chart: the SVG to show and where its marks are.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Drawn {
    pub(crate) svg: String,
    pub(crate) geometry: Geometry,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SeriesSpec {
    pub(crate) name: String,
    pub(crate) values: Vec<Option<f64>>,
    pub(crate) color: HexColor,
    pub(crate) dashed: bool,
}

/// Everything a line chart's SVG depends on, so a memo redraws only on a change.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LineSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: Vec<String>,
    /// Empty: `x` are categories, evenly spaced. Else each point sits at its number.
    pub(crate) x_numbers: Vec<f64>,
    pub(crate) series: Vec<SeriesSpec>,
    pub(crate) curve: Curve,
    pub(crate) grid: bool,
    pub(crate) points: bool,
    pub(crate) stroke_width: f32,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

fn ink(hex: HexColor) -> InkColor {
    InkColor::from((hex.r(), hex.g(), hex.b()))
}

pub(crate) fn draw_line(spec: &LineSpec) -> Option<Drawn> {
    let series = spec
        .series
        .iter()
        .map(|series| {
            let values = series.values.iter().map(|v| v.map(|v| v as f32)).collect();
            let mut drawn = charts_rs::Series::new_nullable(series.name.clone(), values);
            drawn.colors = Some(vec![Some(ink(series.color))]);
            drawn.stroke_dash_array = series.dashed.then(|| DASH.to_string());
            drawn.symbol = Some(match spec.points {
                true => Symbol::Circle(spec.stroke_width + 1.0, Some(ink(series.color))),
                false => Symbol::None,
            });
            match spec.curve {
                Curve::Linear => {}
                Curve::Smooth => drawn.smooth = Some(true),
                Curve::Step => drawn.step = Some(LineStep::End),
            }
            drawn
        })
        .collect();
    let mut chart = charts_rs::LineChart::new(series, spec.x.clone());
    chart.width = spec.width;
    chart.height = spec.height;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    // Its triggers are the marks we harvest; `sanitize` drops its tips.
    chart.tooltip.show = true;
    chart.animation = None;
    chart.series.stroke_width = spec.stroke_width;
    chart.series.colors = spec.series.iter().map(|s| ink(s.color)).collect();
    chart.grid.stroke_color = match spec.grid {
        true => ink(spec.ink.grid),
        false => InkColor::transparent(),
    };
    chart.x_axis.stroke_color = ink(spec.ink.axis);
    chart.x_axis.font.color = ink(spec.ink.label);
    if !spec.x_numbers.is_empty() {
        chart.x_axis.kind = AxisType::Value;
        chart.x_axis.values = spec.x_numbers.clone();
        chart.x_axis.formatter = Some(format!("{X_MARK}{{c}}"));
    }
    let unit = spec
        .unit
        .as_deref()
        .map(|unit| format!(" {unit}"))
        .unwrap_or_default();
    if let Some(y) = chart.y_axis_configs.first_mut() {
        y.stroke_color = ink(spec.ink.axis);
        y.font.color = ink(spec.ink.label);
        y.formatter = Some(format!("{AXIS_MARK}{{c}}{unit}"));
    }
    let write = |label| {
        spec.format
            .format(label, spec.unit.as_deref(), spec.formats)
    };
    let mut svg = chart.svg().ok()?;
    if let Some(width) = y_width(&svg, write) {
        if let Some(y) = chart.y_axis_configs.first_mut() {
            y.width = Some(width);
        }
        svg = chart.svg().ok()?;
    }

    let expected: Vec<_> = expected_marks(&spec.series);
    let (svg, triggers) = sanitize(&pin_to_axis(&svg), write);
    let x_formats = ungrouped(spec.formats);
    let svg = relabel(&svg, X_MARK, |label| {
        spec.format.format(label, None, &x_formats)
    });
    let marks = match_marks(&expected, &triggers);
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot::of(&svg, spec.width, spec.height),
            marks,
        },
        svg: spec.ink.paint(svg),
    })
}

/// Everything a bar chart's SVG depends on.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BarSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: Vec<String>,
    pub(crate) series: Vec<SeriesSpec>,
    pub(crate) layout: BarLayout,
    pub(crate) horizontal: bool,
    /// Corner radius of every bar, in pixels.
    pub(crate) radius: f32,
    pub(crate) value_labels: bool,
    pub(crate) grid: bool,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

/// The settings a vertical and a horizontal bar chart share; both deref to the same base.
macro_rules! style_bars {
    ($chart:ident, $spec:ident, $unit:expr) => {
        $chart.width = $spec.width;
        $chart.height = $spec.height;
        $chart.background_color = InkColor::transparent();
        $chart.title.text = String::new();
        $chart.legend.show = Some(false);
        // Its triggers are the marks we harvest; `sanitize` drops its tips.
        $chart.tooltip.show = true;
        $chart.animation = None;
        $chart.series.colors = $spec.series.iter().map(|s| ink(s.color)).collect();
        $chart.series.label.formatter = format!("{LABEL_MARK}{{c}}{}", $unit);
        $chart.grid.stroke_color = match $spec.grid {
            true => ink($spec.ink.grid),
            false => InkColor::transparent(),
        };
        $chart.x_axis.stroke_color = ink($spec.ink.axis);
        $chart.x_axis.font.color = ink($spec.ink.label);
        if let Some(y) = $chart.y_axis_configs.first_mut() {
            y.stroke_color = ink($spec.ink.axis);
            y.font.color = ink($spec.ink.label);
            y.formatter = Some(format!("{AXIS_MARK}{{c}}{}", $unit));
        }
        // In a stack a label sits on its bar, not on the next one's; past a lying bar's end
        // the longest one's label would leave the plot.
        let inside = $spec.layout != BarLayout::Grouped || $spec.horizontal;
        $chart.series_label_position = inside.then_some(Position::Inside);
        $chart.series.label.font.color = match inside {
            true => ON_BAR,
            false => ink($spec.ink.label),
        };
    };
}

pub(crate) fn draw_bar(spec: &BarSpec) -> Option<Drawn> {
    // The renderer lists a horizontal chart's categories bottom up; ours read top down.
    let order: Vec<usize> = match spec.horizontal {
        true => (0..spec.x.len()).rev().collect(),
        false => (0..spec.x.len()).collect(),
    };
    let ordered: Vec<SeriesSpec> = spec
        .series
        .iter()
        .map(|series| SeriesSpec {
            values: order
                .iter()
                .map(|&i| series.values.get(i).copied().flatten())
                .collect(),
            ..series.clone()
        })
        .collect();
    let series = ordered
        .iter()
        .map(|series| {
            let values = series.values.iter().map(|v| v.map(|v| v as f32)).collect();
            let mut drawn = charts_rs::Series::new_nullable(series.name.clone(), values);
            drawn.label_show = spec.value_labels;
            drawn.stack = (spec.layout != BarLayout::Grouped).then(|| "stack".to_string());
            drawn
        })
        .collect();
    let x = order.iter().map(|&i| spec.x[i].clone()).collect();
    let percent = spec.layout == BarLayout::Percent;
    // A percent chart's values are shares out of 100.
    let unit = match percent {
        true => String::new(),
        false => unit_suffix(spec.unit.as_deref()),
    };
    let write = |value: f64| match percent {
        true => ValueFormat::Percent.format(value / 100.0, None, spec.formats),
        false => spec
            .format
            .format(value, spec.unit.as_deref(), spec.formats),
    };
    let svg = match spec.horizontal {
        true => {
            let mut chart = charts_rs::HorizontalBarChart::new(series, x);
            style_bars!(chart, spec, unit);
            chart.stack_percent = percent;
            pin_to_ticks(&chart.svg().ok()?, spec.width, write)
        }
        false => {
            let mut chart = charts_rs::BarChart::new(series, x);
            style_bars!(chart, spec, unit);
            chart.stack_percent = percent;
            let mut svg = chart.svg().ok()?;
            if let Some(width) = y_width(&svg, write) {
                if let Some(y) = chart.y_axis_configs.first_mut() {
                    y.width = Some(width);
                }
                svg = chart.svg().ok()?;
            }
            pin_to_axis(&svg)
        }
    };

    let rounded = format!(" rx=\"{0}\" ry=\"{0}\"", spec.radius);
    let (svg, triggers) = inert(&svg, write, &rounded);
    let svg = relabel(&svg, LABEL_MARK, write);
    let last = spec.x.len().saturating_sub(1);
    let marks = match_marks(&expected_marks(&ordered), &triggers)
        .into_iter()
        .map(|mark| Mark {
            index: if spec.horizontal {
                last - mark.index
            } else {
                mark.index
            },
            // The drawn bars of a percent chart carry shares; the mark keeps the value.
            value: spec.series[mark.series].values[order[mark.index]].unwrap_or(mark.value),
            ..mark
        })
        .collect::<Vec<_>>();
    let svg = ink_bar_labels(&svg, &marks, |series| {
        spec.ink.on(spec.series[series].color)
    });
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot::of(&svg, spec.width, spec.height),
            marks,
        },
        svg: spec.ink.paint(svg),
    })
}

/// Gives every label drawn on a bar the ink that reads on the bar nearest to it.
fn ink_bar_labels(svg: &str, marks: &[Mark], ink: impl Fn(usize) -> HexColor) -> String {
    let distance = |mark: &Mark, (x, y): (f32, f32)| match mark.shape {
        MarkShape::Rect {
            x: left,
            y: top,
            width,
            height,
        } => {
            let dx = (left - x).max(x - left - width).max(0.0);
            let dy = (top - y).max(y - top - height).max(0.0);
            dx.hypot(dy)
        }
        _ => f32::INFINITY,
    };
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let nearest = number("x")
            .zip(number("y"))
            .filter(|_| tag.contains(ON_BAR_FILL))
            .and_then(|point| {
                marks
                    .iter()
                    .min_by(|a, b| distance(a, point).total_cmp(&distance(b, point)))
            });
        match nearest {
            Some(mark) => {
                out.push_str(&tag.replace(ON_BAR_FILL, &format!("fill=\"{}\"", ink(mark.series))))
            }
            None => out.push_str(tag),
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Everything a pie chart's SVG depends on; `slices` hold one value each.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PieSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) slices: Vec<SeriesSpec>,
    /// The ring's width over the radius; `None`, a full pie.
    pub(crate) donut: Option<f32>,
    pub(crate) labels: PieLabels,
    /// Degrees clockwise from 12 o'clock.
    pub(crate) start_angle: f32,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

pub(crate) fn draw_pie(spec: &PieSpec) -> Option<Drawn> {
    let series = spec
        .slices
        .iter()
        .map(|slice| {
            let value = slice.values.first().copied().flatten().unwrap_or(0.0);
            charts_rs::Series::new(slice.name.clone(), vec![value.max(0.0) as f32])
        })
        .collect();
    let mut chart = charts_rs::PieChart::new(series);
    chart.width = spec.width;
    chart.height = spec.height;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.rose_type = Some(false);
    chart.border_radius = Some(0.0);
    chart.start_angle = spec.start_angle;
    chart.radius = spec.width.min(spec.height) * PIE_FILL / 2.0;
    chart.inner_radius = spec
        .donut
        .map_or(0.0, |ring| chart.radius * (1.0 - ring.clamp(0.05, 1.0)));
    chart.series.colors = spec.slices.iter().map(|s| ink(s.color)).collect();
    chart.series.label.font.color = ink(spec.ink.label);
    chart.series.label.formatter = match spec.labels {
        PieLabels::Value => format!("{AXIS_MARK}{{c}}{}", unit_suffix(spec.unit.as_deref())),
        PieLabels::Percent => format!("{PERCENT_MARK}{{d}}"),
        PieLabels::Name => "{a}".to_string(),
        PieLabels::None => String::new(),
    };
    if spec.labels == PieLabels::None {
        // Past a full turn: no slice is wide enough for a label.
        chart.min_show_label_angle = 361.0;
    }
    let svg = chart.svg().ok()?;

    // A line of the page between two slices of a close colour.
    let edge = format!(" stroke=\"{}\" stroke-width=\"1\"", spec.ink.surface);
    let (svg, triggers) = inert(
        &svg,
        |label| {
            spec.format
                .format(label, spec.unit.as_deref(), spec.formats)
        },
        &edge,
    );
    let svg = relabel(&svg, PERCENT_MARK, |share| {
        ValueFormat::Percent.format(share / 100.0, None, spec.formats)
    });
    // Every slice is drawn, an empty one too.
    let expected: Vec<_> = spec
        .slices
        .iter()
        .enumerate()
        .map(|(i, slice)| {
            let value = slice.values.first().copied().flatten().unwrap_or(0.0);
            (0, i, value, slice.name.as_str())
        })
        .collect();
    Some(Drawn {
        svg: spec.ink.paint(svg),
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot::whole(spec.width, spec.height),
            marks: match_marks(&expected, &triggers),
        },
    })
}

fn unit_suffix(unit: Option<&str>) -> String {
    unit.map(|unit| format!(" {unit}")).unwrap_or_default()
}

/// `(series, index, value, name)` of every mark the renderer draws, in its order.
fn expected_marks(series: &[SeriesSpec]) -> Vec<(usize, usize, f64, &str)> {
    series
        .iter()
        .enumerate()
        .flat_map(|(s, series)| {
            series
                .values
                .iter()
                .enumerate()
                .filter_map(move |(i, value)| {
                    value.map(|value| (s, i, value, series.name.as_str()))
                })
        })
        .collect()
}

/// Pairs the renderer's triggers with the marks we expect; on any mismatch, no marks
/// rather than wrong ones.
fn match_marks(expected: &[(usize, usize, f64, &str)], triggers: &[Trigger]) -> Vec<Mark> {
    let matches = expected.len() == triggers.len()
        && expected
            .iter()
            .zip(triggers)
            .all(|((.., name), trigger)| trigger.series == *name);
    if !matches {
        return Vec::new();
    }
    expected
        .iter()
        .zip(triggers)
        .map(|(&(series, index, value, _), trigger)| Mark {
            series,
            index,
            value,
            shape: trigger.shape.clone(),
        })
        .collect()
}

/// One of the renderer's hover targets: a mark's shape and its series' name.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Trigger {
    pub(crate) series: String,
    pub(crate) shape: MarkShape,
}

/// The renderer's SVG made inert: no own tooltips, no `<title>`s, the page's font, its
/// y labels written by `axis`, sized by its container. Returns the triggers it dropped.
pub(crate) fn sanitize(svg: &str, axis: impl Fn(f64) -> String) -> (String, Vec<Trigger>) {
    inert(svg, axis, "")
}

/// [`sanitize`], with `extra` attributes on every kept mark.
fn inert(svg: &str, axis: impl Fn(f64) -> String, extra: &str) -> (String, Vec<Trigger>) {
    let mut out = String::with_capacity(svg.len());
    let mut triggers = Vec::new();
    let mut rest = svg;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('>').map(|close| open + close) else {
            rest = &rest[open..];
            break;
        };
        let tag = &rest[open..=close];
        let name = tag_name(tag);
        let attributes = attributes(tag);
        let class = attribute(&attributes, "class");
        let end_of = |rest: &str| {
            let closing = format!("</{name}>");
            rest[close..]
                .find(&closing)
                .map(|end| close + end + closing.len())
                .unwrap_or(close + 1)
        };
        if name == "style" || class == Some("ct-tip") {
            rest = &rest[end_of(rest)..];
            continue;
        }
        if class == Some("ct-trigger") {
            if let Some(trigger) = trigger(name, &attributes) {
                triggers.push(trigger);
            }
            let end = end_of(rest);
            // A line's trigger is an invisible circle; a bar or a slice is the mark itself.
            if name != "circle" {
                out.push_str(&self_closing(name, &attributes, extra));
            }
            rest = &rest[end..];
            continue;
        }
        if name == "svg" {
            out.push_str(&root(&attributes));
        } else {
            out.push_str(tag);
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    let out = out.replace("font-family=\"Roboto\"", "font-family=\"inherit\"");
    (relabel(&out, AXIS_MARK, axis), triggers)
}

fn tag_name(tag: &str) -> &str {
    let inner = tag.trim_start_matches('<');
    let end = inner
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(inner.len());
    &inner[..end]
}

/// A tag's attributes, values still escaped.
fn attributes(tag: &str) -> Vec<(&str, &str)> {
    let mut found = Vec::new();
    let mut rest = &tag[1 + tag_name(tag).len()..];
    while let Some(eq) = rest.find("=\"") {
        let name = rest[..eq].trim();
        let after = &rest[eq + 2..];
        let Some(end) = after.find('"') else { break };
        found.push((name, &after[..end]));
        rest = &after[end + 1..];
    }
    found
}

fn attribute<'a>(attributes: &[(&str, &'a str)], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(attribute, _)| *attribute == name)
        .map(|(_, value)| *value)
}

fn trigger(name: &str, attributes: &[(&str, &str)]) -> Option<Trigger> {
    let number = |name| attribute(attributes, name)?.parse::<f32>().ok();
    let shape = match name {
        "circle" => MarkShape::Point {
            x: number("cx")?,
            y: number("cy")?,
        },
        "rect" => MarkShape::Rect {
            x: number("x")?,
            y: number("y")?,
            width: number("width")?,
            height: number("height")?,
        },
        "path" => MarkShape::Sector {
            path: attribute(attributes, "d")?.to_string(),
        },
        _ => return None,
    };
    Some(Trigger {
        series: unescape(attribute(attributes, "data-series")?),
        shape,
    })
}

/// A mark without the renderer's hover hooks.
fn self_closing(name: &str, attributes: &[(&str, &str)], extra: &str) -> String {
    let kept: String = attributes
        .iter()
        .filter(|(attribute, _)| *attribute != "class" && !attribute.starts_with("data-"))
        .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
        .collect();
    format!("<{name}{kept}{extra}/>")
}

/// The root at its container's size, out of the accessibility tree and the tab order.
fn root(attributes: &[(&str, &str)]) -> String {
    let kept: String = attributes
        .iter()
        .filter(|(attribute, _)| !matches!(*attribute, "width" | "height"))
        .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
        .collect();
    format!("<svg width=\"100%\" height=\"100%\" aria-hidden=\"true\" focusable=\"false\"{kept}>")
}

/// Writes every label behind `mark` again with `axis`.
fn relabel(svg: &str, mark: char, axis: impl Fn(f64) -> String) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(at) = rest.find(mark) {
        out.push_str(&rest[..at]);
        let label = &rest[at + mark.len_utf8()..];
        let end = label.find(['\n', '<']).unwrap_or(label.len());
        let number = label[..end].split(' ').next().unwrap_or_default();
        match scaled(number.trim_end_matches('%')) {
            Some(value) => out.push_str(&escape(&axis(value))),
            None => out.push_str(&label[..end]),
        }
        rest = &label[end..];
    }
    out.push_str(rest);
    out
}

/// The renderer places a y label by its own text's width; written again in the locale it
/// would leave the axis. Pins each one by the right end the renderer gave it.
fn pin_to_axis(svg: &str) -> String {
    pin_labels(svg, Pin::<fn(f64) -> String>::End)
}

/// A lying bar chart's value labels sit under their ticks: pinned by their centre, as
/// `axis` writes them, and kept inside the `width`.
fn pin_to_ticks(svg: &str, width: f32, axis: impl Fn(f64) -> String) -> String {
    pin_labels(svg, Pin::Middle { width, axis })
}

enum Pin<F> {
    End,
    Middle { width: f32, axis: F },
}

fn pin_labels<F: Fn(f64) -> String>(svg: &str, pin: Pin<F>) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let Some(close) = rest[open..].find('>').map(|close| open + close) else {
            break;
        };
        out.push_str(&rest[..open]);
        let tag = &rest[open..=close];
        let label = &rest[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        let attributes = attributes(tag);
        let x = attribute(&attributes, "x").and_then(|x| x.parse::<f32>().ok());
        match x {
            Some(x) if label.starts_with(AXIS_MARK) => {
                let drawn = text_width(&attributes, label);
                let (x, anchor) = match &pin {
                    Pin::End => (x + drawn, "end"),
                    Pin::Middle { width, axis } => {
                        let number = label[AXIS_MARK.len_utf8()..].split(' ').next();
                        let half = number
                            .and_then(scaled)
                            .map_or(drawn, |value| text_width(&attributes, &axis(value)))
                            / 2.0;
                        let centre = x + drawn / 2.0;
                        (centre.min(width - half).max(half), "middle")
                    }
                };
                let kept: String = attributes
                    .iter()
                    .filter(|(attribute, _)| *attribute != "x")
                    .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
                    .collect();
                out.push_str(&format!("<text x=\"{x}\" text-anchor=\"{anchor}\"{kept}>"));
            }
            _ => out.push_str(tag),
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// The y axis' width once `axis` writes its labels, when that is more than the renderer
/// kept for them.
fn y_width(svg: &str, axis: impl Fn(f64) -> String) -> Option<f32> {
    let mut kept = 0.0f32;
    let mut needed = 0.0f32;
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let close = open + rest[open..].find('>')?;
        let attributes = attributes(&rest[open..=close]);
        let label = &rest[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        if let Some(number) = label.strip_prefix(AXIS_MARK) {
            let number = number.split(' ').next().unwrap_or_default();
            kept = kept.max(text_width(&attributes, label));
            if let Some(value) = scaled(number) {
                needed = needed.max(text_width(&attributes, &axis(value)));
            }
        }
        rest = &rest[close + 1..];
    }
    // The renderer's own room: the label's width and 5.
    (needed > kept).then_some(needed + 5.0)
}

/// A label's width in the renderer's own measure, in the tag's font.
fn text_width(attributes: &[(&str, &str)], text: &str) -> f32 {
    let family = attribute(attributes, "font-family").unwrap_or(charts_rs::DEFAULT_FONT_FAMILY);
    let size = attribute(attributes, "font-size")
        .and_then(|size| size.parse().ok())
        .unwrap_or(14.0);
    charts_rs::measure_text_width_family(family, size, text).map_or(0.0, |text| text.width())
}

/// A number the renderer may have written with a `k`, `M`, `G` or `T` suffix: `1.5k`.
fn scaled(text: &str) -> Option<f64> {
    let factor = match text.chars().last()? {
        'k' => 1e3,
        'M' => 1e6,
        'G' => 1e9,
        'T' => 1e12,
        _ => return text.parse().ok(),
    };
    let number: f64 = text[..text.len() - 1].parse().ok()?;
    Some(number * factor)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Color, ColorShade, Theme, chart_hex};

    fn line(series: Vec<SeriesSpec>, x: &[&str]) -> LineSpec {
        LineSpec {
            width: 320.0,
            height: 180.0,
            x: x.iter().map(|x| x.to_string()).collect(),
            x_numbers: Vec::new(),
            series,
            curve: Curve::Linear,
            grid: true,
            points: true,
            stroke_width: 2.0,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: Some("kg".to_string()),
            formats: &Formats::GERMAN,
        }
    }

    fn spec(name: &str, values: &[Option<f64>]) -> SeriesSpec {
        SeriesSpec {
            name: name.to_string(),
            values: values.to_vec(),
            color: chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6),
            dashed: false,
        }
    }

    /// Contract with the pinned `charts-rs`: under forced colours the ink it draws is ours to
    /// trade, so no stand-in hex is left and the series keep theirs.
    #[test]
    fn forced_ink_leaves_no_stand_in_in_the_svg() {
        let mut spec = line(vec![spec("A", &[Some(1.0), Some(3.0)])], &["Mon", "Tue"]);
        spec.ink = ChartInk::of(&Theme::DEFAULT, true);
        let svg = draw_line(&spec).expect("drawn").svg;
        assert!(!svg.contains("#020"), "{svg}");
        assert!(svg.contains("CanvasText"));
        assert!(svg.contains(&spec.series[0].color.to_string()));
    }

    /// The y labels, written again grouped and with their unit, end on one line left of
    /// the plot: the axis grows to hold them.
    #[test]
    fn grouped_y_labels_end_left_of_the_plot() {
        let drawn = draw_line(&line(
            vec![spec("A", &[Some(12_400.0), Some(21_300.0)])],
            &["Q1", "Q2"],
        ))
        .expect("drawn");
        let ends: Vec<f32> = drawn
            .svg
            .split("<text x=\"")
            .skip(1)
            .filter(|tag| tag.contains("text-anchor=\"end\""))
            .filter_map(|tag| tag.split('"').next()?.parse().ok())
            .collect();
        assert!(ends.len() > 2, "{}", drawn.svg);
        assert!(ends.iter().all(|end| *end == ends[0]), "{ends:?}");
        assert!(drawn.svg.contains("18.000 kg"), "{}", drawn.svg);
        let first = xs(&drawn)[0];
        assert!(
            ends[0] < first,
            "labels end at {} past the plot at {first}",
            ends[0]
        );
    }

    /// Contract with the pinned `charts-rs`: a line's points come back in data order,
    /// gaps skipped, and the SVG keeps none of its own tooltips.
    #[test]
    fn a_line_s_points_are_harvested_and_its_tooltips_dropped() {
        let drawn = draw_line(&line(
            vec![
                spec("A & B", &[Some(1.0), None, Some(3.0)]),
                spec("C", &[Some(2.0), Some(2.5), Some(1.0)]),
            ],
            &["Mon", "Tue", "Wed"],
        ))
        .expect("drawn");
        let marks = &drawn.geometry.marks;

        let places: Vec<_> = marks.iter().map(|m| (m.series, m.index)).collect();
        assert_eq!(places, [(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)]);
        let ys: Vec<_> = marks
            .iter()
            .map(|m| match m.shape {
                MarkShape::Point { y, .. } => y,
                _ => panic!("{m:?}"),
            })
            .collect();
        // Higher values sit higher up.
        assert!(ys[1] < ys[0] && ys[3] < ys[2], "{ys:?}");
        for gone in [
            "ct-trigger",
            "ct-tip",
            "<title",
            "<style",
            "Roboto",
            "\u{2060}",
            "\u{2061}",
        ] {
            assert!(!drawn.svg.contains(gone), "{gone}: {}", drawn.svg);
        }
        assert!(
            drawn
                .svg
                .starts_with("<svg width=\"100%\" height=\"100%\" aria-hidden=\"true\"")
        );
        assert!(drawn.svg.contains("2,5 kg"), "{}", drawn.svg);
    }

    fn xs(drawn: &Drawn) -> Vec<f32> {
        drawn
            .geometry
            .marks
            .iter()
            .map(|m| match m.shape {
                MarkShape::Point { x, .. } => x,
                _ => panic!("{m:?}"),
            })
            .collect()
    }

    /// Numbers are placed by value, categories evenly, and both are harvested in data order.
    #[test]
    fn numeric_x_values_are_spaced_by_value() {
        let values = [Some(1.0), Some(2.0), Some(3.0)];
        let mut numeric = line(vec![spec("A", &values)], &["1", "2", "10"]);
        let even = xs(&draw_line(&numeric).expect("drawn"));
        assert!(
            ((even[1] - even[0]) - (even[2] - even[1])).abs() < 0.1,
            "{even:?}"
        );

        numeric.x_numbers = vec![1.0, 2.0, 10.0];
        let drawn = draw_line(&numeric).expect("drawn");
        let at = xs(&drawn);
        assert_eq!(at.len(), 3, "{drawn:?}");
        let ratio = (at[2] - at[1]) / (at[1] - at[0]);
        assert!((ratio - 8.0).abs() < 0.05, "{at:?}");
    }

    /// The renderer's `2.02k` on the axis reads as the number in the chart's format.
    #[test]
    fn numeric_x_labels_are_written_in_the_locale() {
        let mut numeric = line(vec![spec("A", &[Some(1.0), Some(2.0)])], &["a", "b"]);
        numeric.x_numbers = vec![2020.0, 2030.0];
        let years = draw_line(&numeric).expect("drawn").svg;
        assert!(years.contains("\n2025\n"), "{years}");
        numeric.x_numbers = vec![0.0, 1.0];
        let svg = draw_line(&numeric).expect("drawn").svg;
        assert!(svg.contains("\n0,5\n") || svg.contains("\n0,2\n"), "{svg}");
    }

    #[test]
    fn a_non_finite_x_number_still_draws() {
        let mut numeric = line(
            vec![spec("A", &[Some(1.0), Some(2.0), Some(3.0)])],
            &["a", "b", "c"],
        );
        numeric.x_numbers = vec![1.0, f64::NAN, 3.0];
        let drawn = draw_line(&numeric).expect("drawn");
        let places: Vec<_> = drawn.geometry.marks.iter().map(|m| m.index).collect();
        assert!(places.is_empty() || places == [0, 2], "{places:?}");
        numeric.x_numbers = vec![f64::NAN; 3];
        assert!(draw_line(&numeric).is_some());
    }

    #[test]
    fn a_suffixed_number_is_read_back() {
        assert_eq!(scaled("1.5k"), Some(1500.0));
        assert_eq!(scaled("-2M"), Some(-2e6));
        assert_eq!(scaled("42"), Some(42.0));
        assert_eq!(scaled("Jan"), None);
        assert_eq!(scaled(""), None);
    }

    /// The series' hex is the one drawn: no `var()` reaches the renderer.
    #[test]
    fn a_line_is_drawn_in_its_hex() {
        let drawn =
            draw_line(&line(vec![spec("A", &[Some(1.0), Some(2.0)])], &["a", "b"])).expect("drawn");
        let hex = chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6).to_string();
        assert!(
            drawn.svg.to_uppercase().contains(&hex.to_uppercase()),
            "{hex}: {}",
            drawn.svg
        );
    }

    #[test]
    fn a_mismatch_harvests_no_marks() {
        let triggers = [Trigger {
            series: "B".to_string(),
            shape: MarkShape::Point { x: 0.0, y: 0.0 },
        }];
        assert!(match_marks(&[(0, 0, 1.0, "A")], &triggers).is_empty());
    }

    /// Contract: a bar is a trigger `rect`, kept as the mark, its hooks stripped.
    #[test]
    fn a_bar_is_harvested_as_its_rect() {
        let mut chart = charts_rs::BarChart::new(
            vec![charts_rs::Series::new("A".to_string(), vec![1.0, 2.0])],
            vec!["x".to_string(), "y".to_string()],
        );
        chart.tooltip.show = true;
        chart.background_color = InkColor::transparent();
        let (svg, triggers) = sanitize(&chart.svg().expect("svg"), |v| v.to_string());

        assert_eq!(triggers.len(), 2);
        assert!(
            triggers
                .iter()
                .all(|t| matches!(t.shape, MarkShape::Rect { height, .. } if height > 0.0))
        );
        assert_eq!(svg.matches("<rect").count(), 2, "{svg}");
        assert!(
            !svg.contains("ct-trigger") && !svg.contains("<title"),
            "{svg}"
        );
    }

    fn bars(layout: BarLayout, horizontal: bool) -> BarSpec {
        BarSpec {
            width: 320.0,
            height: 180.0,
            x: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            series: vec![
                spec("A", &[Some(1.0), Some(3.0), None]),
                spec("B", &[Some(3.0), Some(1.0), Some(2.0)]),
            ],
            layout,
            horizontal,
            radius: 4.0,
            value_labels: true,
            grid: true,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: None,
            formats: &Formats::GERMAN,
        }
    }

    fn rect(mark: &Mark) -> (f32, f32, f32, f32) {
        match mark.shape {
            MarkShape::Rect {
                x,
                y,
                width,
                height,
            } => (x, y, width, height),
            _ => panic!("{mark:?}"),
        }
    }

    /// Contract: grouped bars come back series by series, rounded, a gap without a bar.
    #[test]
    fn grouped_bars_are_harvested_in_data_order() {
        let drawn = draw_bar(&bars(BarLayout::Grouped, false)).expect("drawn");
        let marks = &drawn.geometry.marks;

        let places: Vec<_> = marks.iter().map(|m| (m.series, m.index)).collect();
        assert_eq!(places, [(0, 0), (0, 1), (1, 0), (1, 1), (1, 2)]);
        // A's 3 is taller than its 1, and A's bar sits left of B's in the same group.
        assert!(rect(&marks[1]).3 > rect(&marks[0]).3);
        assert!(rect(&marks[0]).0 < rect(&marks[2]).0);
        assert_eq!(drawn.svg.matches("rx=\"4\"").count(), 5, "{}", drawn.svg);
        assert!(!drawn.svg.contains("data-series"), "{}", drawn.svg);
    }

    /// Horizontal, the first category is the top row and the marks keep data indices.
    #[test]
    fn horizontal_bars_read_top_down() {
        let drawn = draw_bar(&bars(BarLayout::Grouped, true)).expect("drawn");
        let marks = &drawn.geometry.marks;

        let a = marks
            .iter()
            .find(|m| (m.series, m.index) == (1, 0))
            .expect("B at a");
        let c = marks
            .iter()
            .find(|m| (m.series, m.index) == (1, 2))
            .expect("B at c");
        assert!(rect(a).1 < rect(c).1, "{marks:?}");
        // Values grow to the right.
        assert!(rect(a).2 > rect(c).2, "{marks:?}");
        assert_eq!(a.value, 3.0);
    }

    /// A percent stack fills the plot's height, its labels are shares in the locale, and
    /// its marks keep the values.
    #[test]
    fn percent_bars_fill_the_stack_and_keep_their_values() {
        let drawn = draw_bar(&bars(BarLayout::Percent, false)).expect("drawn");
        let marks = &drawn.geometry.marks;

        let (a, b) = (rect(&marks[0]), rect(&marks[2]));
        assert!((a.3 * 3.0 - b.3).abs() < 1.0, "{a:?} {b:?}");
        // Stacked: same x, B on top of A.
        assert_eq!(a.0, b.0);
        assert!((b.1 + b.3 - a.1).abs() < 1.0, "{a:?} {b:?}");
        assert_eq!(marks[0].value, 1.0);
        assert!(
            drawn.svg.contains("25%") && drawn.svg.contains("100%"),
            "{}",
            drawn.svg
        );
        assert!(!drawn.svg.contains('\u{2061}'), "{}", drawn.svg);
        // Labels on the bars take the ink that reads on them.
        assert!(!drawn.svg.contains(ON_BAR_FILL), "{}", drawn.svg);
        let on = ChartInk::of(&Theme::DEFAULT, false).on(spec("A", &[]).color);
        assert!(
            drawn.svg.contains(&format!("fill=\"{on}\"")),
            "{on}: {}",
            drawn.svg
        );
    }

    fn pie(labels: PieLabels, donut: Option<f32>) -> PieSpec {
        PieSpec {
            width: 320.0,
            height: 320.0,
            slices: vec![spec("A", &[Some(1.0)]), spec("B", &[Some(3.0)])],
            donut,
            labels,
            start_angle: 0.0,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: Some("kg".to_string()),
            formats: &Formats::GERMAN,
        }
    }

    /// The first slice starts at 12 o'clock and runs clockwise; the labels are in the locale.
    #[test]
    fn a_pie_starts_at_the_top_and_writes_its_labels() {
        let drawn = draw_pie(&pie(PieLabels::Percent, None)).expect("drawn");
        let marks = &drawn.geometry.marks;

        let places: Vec<_> = marks.iter().map(|m| (m.series, m.index)).collect();
        assert_eq!(places, [(0, 0), (0, 1)]);
        let MarkShape::Sector { path } = &marks[0].shape else {
            panic!("{marks:?}")
        };
        // A quarter from the top, clockwise: from the centre straight up, then to the right.
        let point = |token: &str| -> (f32, f32) {
            let (x, y) = token.split_once(',').expect(path);
            (x.parse().expect(path), y.parse().expect(path))
        };
        let tokens: Vec<_> = path.split(' ').collect();
        let (top, end) = (point(&tokens[1][1..]), point(tokens[tokens.len() - 3]));
        assert!((top.0 - 160.0).abs() < 0.5 && top.1 < 160.0, "{path}");
        assert!(end.0 > 160.0 && (end.1 - 160.0).abs() < 0.5, "{path}");
        assert!(
            drawn.svg.contains("25%") && drawn.svg.contains("75%"),
            "{}",
            drawn.svg
        );

        let values = draw_pie(&pie(PieLabels::Value, None)).expect("drawn");
        assert!(values.svg.contains("3 kg"), "{}", values.svg);
        let none = draw_pie(&pie(PieLabels::None, Some(0.4))).expect("drawn");
        assert!(!none.svg.contains("<text"), "{}", none.svg);
    }

    /// The plot is the box inside the axes, grid or not: above the x labels, right of
    /// the y labels, and every mark in it.
    #[test]
    fn the_plot_is_the_box_inside_the_axes() {
        let mut lines = line(vec![spec("A", &[Some(1.0), Some(3.0)])], &["Mon", "Tue"]);
        lines.grid = false;
        let mut lying = bars(BarLayout::Grouped, true);
        lying.grid = false;
        for geometry in [
            draw_line(&lines).expect("drawn").geometry,
            draw_bar(&bars(BarLayout::Stacked, false))
                .expect("drawn")
                .geometry,
            draw_bar(&lying).expect("drawn").geometry,
        ] {
            let plot = geometry.plot;
            assert!(plot.left > 10.0 && plot.top > 0.0, "{plot:?}");
            assert!(plot.bottom < geometry.height - 10.0, "{plot:?}");
            assert!(plot.right <= geometry.width, "{plot:?}");
            for mark in &geometry.marks {
                let (x, y) = hit_anchor(&mark.shape);
                assert!(
                    (plot.left..=plot.right).contains(&x) && (plot.top..=plot.bottom).contains(&y),
                    "{mark:?} {plot:?}"
                );
            }
        }
        let pie = draw_pie(&pie(PieLabels::Percent, None))
            .expect("drawn")
            .geometry;
        assert_eq!(pie.plot, Plot::whole(pie.width, pie.height));
    }

    /// Lying, the value labels under the plot, written again grouped, stay centred on
    /// their ticks.
    #[test]
    fn lying_value_labels_sit_on_their_ticks() {
        let mut lying = bars(BarLayout::Grouped, true);
        lying.series = vec![spec("A", &[Some(12_400.0), Some(31_000.0), Some(8_000.0)])];
        let svg = draw_bar(&lying).expect("drawn").svg;
        let ticks: Vec<f32> = svg
            .split("<line ")
            .skip(1)
            .filter_map(|tag| {
                let number = |name: &str| -> Option<f32> {
                    tag.split(&format!("{name}=\""))
                        .nth(1)?
                        .split('"')
                        .next()?
                        .parse()
                        .ok()
                };
                let (x1, x2, y1, y2) = (number("x1")?, number("x2")?, number("y1")?, number("y2")?);
                (x1 == x2 && y2 - y1 == 5.0).then_some(x1)
            })
            .collect();
        let labels: Vec<f32> = svg
            .split("<text x=\"")
            .skip(1)
            .filter(|tag| tag.contains("text-anchor=\"middle\""))
            .filter_map(|tag| tag.split('"').next()?.parse().ok())
            .collect();
        assert!(labels.len() > 2 && svg.contains("10.400"), "{svg}");
        // The last one ends at the SVG's edge rather than past it.
        let (last, inner) = labels.split_last().expect("labels");
        for label in inner {
            assert!(
                ticks.iter().any(|tick| (tick - label).abs() < 1.0),
                "{label} off {ticks:?}"
            );
        }
        assert!(
            *last > inner[inner.len() - 1] && *last < 320.0 - 15.0,
            "{last}"
        );
    }

    fn hit_anchor(shape: &MarkShape) -> (f32, f32) {
        match *shape {
            MarkShape::Point { x, y } => (x, y),
            MarkShape::Rect {
                x,
                y,
                width,
                height,
            } => (x + width / 2.0, y + height / 2.0),
            MarkShape::Sector { .. } => panic!("{shape:?}"),
        }
    }

    /// Contract: a slice is a trigger `path`, one per series.
    #[test]
    fn a_pie_slice_is_harvested_as_its_path() {
        let mut chart = charts_rs::PieChart::new(vec![
            charts_rs::Series::new("A".to_string(), vec![1.0]),
            charts_rs::Series::new("B".to_string(), vec![3.0]),
        ]);
        chart.tooltip.show = true;
        let (_, triggers) = sanitize(&chart.svg().expect("svg"), |v| v.to_string());

        let names: Vec<_> = triggers.iter().map(|t| t.series.as_str()).collect();
        assert_eq!(names, ["A", "B"]);
        assert!(
            triggers
                .iter()
                .all(|t| matches!(&t.shape, MarkShape::Sector { path } if path.starts_with('M')))
        );
    }
}
