//! Props to `charts-rs` and back: it draws the static SVG, we harvest where each mark
//! landed and strip its own tooltips, so the overlay and keyboard layers can be ours.

use charts_rs::{
    AxisLabelOverflow, AxisType, Color as InkColor, DEFAULT_FONT_FAMILY, LineStep,
    MarkLineCategory, Position, SeriesCategory, Symbol,
};

use super::data::{Trend, ValueFormat, ungrouped};
use crate::{
    localization::Formats,
    theme::{BarLayout, ChartInk, Curve, HexColor, PieLabels, PointSymbol},
};

/// Ahead of each y-axis label, so [`sanitize`] can write it again in the locale.
const AXIS_MARK: char = '\u{2060}';
/// The same for the labels of a numeric x axis.
const X_MARK: char = '\u{2061}';
/// Ahead of each percentage the renderer writes, to write it again in the locale.
const PERCENT_MARK: char = '\u{2062}';
/// Ahead of each label on a bar: written as the y labels, but not pinned to the axis.
const LABEL_MARK: char = '\u{2063}';
/// Before every right y axis label, written again with that axis' unit.
const AXIS2_MARK: char = '\u{2064}';
const DASH: &str = "6,4";
/// An axis' tick is 5 long; an axis or grid line is longer.
const PLOT_TICK: f32 = 10.0;
/// The pie's radius over half the plot's shorter side, as the renderer caps it.
const PIE_FILL: f32 = 0.8;
/// The least of its full radius a pie keeps to make room for its labels.
const PIE_MIN: f32 = 0.4;
/// Room between a label and the SVG's edge.
const EDGE: f32 = 4.0;
/// A label on a bar is drawn in this, then given the ink that reads on its bar.
const ON_BAR: InkColor = InkColor {
    r: 1,
    g: 2,
    b: 3,
    a: 255,
};
const ON_BAR_FILL: &str = "fill=\"#010203\"";
/// The renderer's room at each side of a band and between two bars of a group, in px.
const RENDERER_MARGIN: f32 = 5.0;
const RENDERER_GAP: f32 = 3.0;

/// Where every mark of a drawn chart sits, in the SVG's own units.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Geometry {
    pub(crate) width: f32,
    pub(crate) height: f32,
    /// The area inside the axes; a pie's is the square its `center` fills.
    pub(crate) plot: Plot,
    /// Series by series, in data order; a gap has no mark.
    pub(crate) marks: Vec<Mark>,
    /// A donut's hole radius, round the SVG's middle; 0 without one.
    pub(crate) hole: f32,
    /// A funnel's stages are shares of its first, a pie's slices of their sum: `None`.
    pub(crate) share_of: Option<f64>,
    /// A heatmap's marks are cells: each tipped alone, not with its column.
    pub(crate) cells: bool,
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
    pub(crate) extras: SeriesExtras,
}

/// What a series asks beyond its values and look; a bar or a slice draws none of it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SeriesExtras {
    pub(crate) symbol: PointSymbol,
    /// Each mark line's value and the text written by it.
    pub(crate) mark_lines: Vec<(f64, String)>,
    /// Each marked point's x index and the text pinned by it.
    pub(crate) mark_points: Vec<(usize, String)>,
    /// Measured on the right y axis.
    pub(crate) secondary: bool,
    /// A line over a bar chart's bars.
    pub(crate) line: bool,
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
    /// Each series' error range per value, drawn as a band around its line.
    pub(crate) errors: Vec<Vec<Option<(f64, f64)>>>,
    pub(crate) curve: Curve,
    /// The areas' opacity; `None`, no area.
    pub(crate) area: Option<f32>,
    pub(crate) stacked: bool,
    pub(crate) axes: Axes,
    pub(crate) grid: bool,
    pub(crate) points: bool,
    pub(crate) stroke_width: f32,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    /// The right y axis' unit.
    pub(crate) y2_unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

/// What a line or bar chart's axes are given beyond its data.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Axes {
    /// The value axis' ends; unset, from the data and 0.
    pub(crate) min: Option<f64>,
    pub(crate) max: Option<f64>,
    /// The steps the value axis is cut into.
    pub(crate) ticks: usize,
    pub(crate) x_title: Option<String>,
    pub(crate) y_title: Option<String>,
    /// A line chart's right y axis, read downward.
    pub(crate) y2_title: Option<String>,
    /// The locale's million, billion and trillion suffixes for a long axis' ticks.
    pub(crate) compact: [&'static str; 3],
}

impl Axes {
    /// `lying`: the renderer's first y config is a horizontal bar chart's value axis, at the
    /// bottom, and its x title the categories' beside them.
    fn style(
        &self,
        chart: &mut charts_rs::ChartBase,
        y: &mut [charts_rs::YAxisConfig],
        lying: bool,
    ) {
        let (bottom, side) = match lying {
            true => (&self.y_title, &self.x_title),
            false => (&self.x_title, &self.y_title),
        };
        chart.x_axis.title = bottom.clone().unwrap_or_default();
        if let Some(first) = y.first_mut() {
            first.min = self.min.map(|min| min as f32);
            first.max = self.max.map(|max| max as f32);
            first.split_number = self.ticks.max(1);
            first.title = side.clone();
        }
        // Cut like the left one, so its labels sit on the grid's lines.
        if let Some(second) = y.get_mut(1) {
            second.split_number = self.ticks.max(1);
            second.title = self.y2_title.clone();
        }
    }
}

/// Every series' mark lines inside its value axis' ends, where unset the data's extent and
/// 0, which the axis always spans; in the order the renderer draws them.
fn kept_mark_lines<'a>(series: &'a [SeriesSpec], axes: &Axes) -> Vec<Vec<&'a (f64, String)>> {
    let ends = |secondary: bool| {
        let values = series
            .iter()
            .filter(|s| s.extras.secondary == secondary)
            .flat_map(|s| s.values.iter().flatten().copied());
        let fixed = |end: Option<f64>| end.filter(|_| !secondary);
        let low = fixed(axes.min).unwrap_or(values.clone().fold(0.0, f64::min));
        (low, fixed(axes.max).unwrap_or(values.fold(0.0, f64::max)))
    };
    let axis = [ends(false), ends(true)];
    series
        .iter()
        .map(|series| {
            let (low, high) = axis[series.extras.secondary as usize];
            let lines = series.extras.mark_lines.iter();
            lines
                .filter(|(value, _)| (low..=high).contains(value))
                .collect()
        })
        .collect()
}

fn renderer_mark_line(&(value, _): &(f64, String)) -> charts_rs::MarkLine {
    charts_rs::MarkLine {
        category: MarkLineCategory::Value(value as f32),
        ..Default::default()
    }
}

/// Writes each mark line's own text, in drawing order, over the bare number the renderer
/// puts by it: in the label ink, haloed in the surface's so the data under it cannot blur it.
/// One wider than the plot from its `left` is left out; the summary keeps it.
fn label_mark_lines(svg: &str, texts: &[&str], ink: &ChartInk, left: f32) -> String {
    let mut texts = texts.iter();
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find("</text>")
            .map_or(rest.len(), |end| start + end + "</text>".len());
        let element = &rest[start..end];
        rest = &rest[end..];
        let tag = &element[..element.find('>').map_or(element.len(), |i| i + 1)];
        let mark = tag.contains("dy=\"-6\"") && tag.contains("text-anchor=\"end\"");
        let Some(text) = mark.then(|| texts.next()).flatten() else {
            out.push_str(element);
            continue;
        };
        let attributes = attributes(tag);
        let end = attribute(&attributes, "x").and_then(|x| x.parse::<f32>().ok());
        if end.is_some_and(|end| end - text_width(&attributes, text) < left) {
            continue;
        }
        let kept: String = attributes
            .iter()
            .map(|&(name, value)| match name {
                "fill" => format!(" fill=\"{}\"", ink.label),
                _ => format!(" {name}=\"{value}\""),
            })
            .collect();
        let halo = format!(
            " stroke=\"{}\" stroke-width=\"3\" stroke-linejoin=\"round\" paint-order=\"stroke\"",
            ink.surface
        );
        out.push_str(&format!("<text{kept}{halo}>\n{}\n</text>", escape(text)));
    }
    out.push_str(rest);
    out
}

/// Each marked point's value in a pill of its series' colour, above the point or, with no
/// room there, below it; in black or white, whichever reads on the colour.
fn pin_mark_points(
    svg: &str,
    all: &[SeriesSpec],
    chart_width: f32,
    ink: &ChartInk,
    marks: &[Mark],
) -> String {
    const HEIGHT: f32 = 20.0;
    const GAP: f32 = 6.0;
    const SIZE: f32 = 12.0;
    let mut pins = String::new();
    for (s, series) in all.iter().enumerate() {
        for (index, text) in &series.extras.mark_points {
            let place = marks.iter().find(|m| m.series == s && m.index == *index);
            let Some(MarkShape::Point { x, y }) = place.map(|mark| &mark.shape) else {
                continue;
            };
            let width = charts_rs::measure_text_width_family(DEFAULT_FONT_FAMILY, SIZE, text)
                .map_or(0.0, |text| text.width())
                + HEIGHT * 0.6;
            let left = (x - width / 2.0).clamp(1.0, (chart_width - width - 1.0).max(1.0));
            let top = match y - GAP - HEIGHT >= 1.0 {
                true => y - GAP - HEIGHT,
                false => y + GAP,
            };
            pins.push_str(&format!(
                "<g class=\"ct-pin\"><rect x=\"{left}\" y=\"{top}\" width=\"{width}\" height=\"{HEIGHT}\" \
                 rx=\"{}\" fill=\"{}\"/><text x=\"{}\" y=\"{}\" font-size=\"{SIZE}\" \
                 font-family=\"inherit\" text-anchor=\"middle\" dominant-baseline=\"central\" \
                 fill=\"{}\">{}</text></g>",
                HEIGHT / 2.0,
                series.color,
                left + width / 2.0,
                top + HEIGHT / 2.0,
                ink.on(series.color),
                escape(text),
            ));
        }
    }
    match svg.rfind("</svg>") {
        Some(end) => format!("{}{pins}{}", &svg[..end], &svg[end..]),
        None => svg.to_string(),
    }
}

fn ink(hex: HexColor) -> InkColor {
    InkColor::from((hex.r(), hex.g(), hex.b()))
}

/// A point's marker, sized so every shape covers about a circle's area.
fn symbol(points: bool, stroke_width: f32, series: &SeriesSpec) -> Symbol {
    let (r, fill) = (stroke_width + 1.0, Some(ink(series.color)));
    match (points, series.extras.symbol) {
        (false, _) => Symbol::None,
        (true, PointSymbol::Circle) => Symbol::Circle(r, fill),
        (true, PointSymbol::Square) => Symbol::Rect(r * 0.9, fill),
        (true, PointSymbol::Triangle) => Symbol::Triangle(r * 1.35, fill),
        (true, PointSymbol::Diamond) => Symbol::Diamond(r * 1.25, fill),
    }
}

/// A line's error ranges as a band, only where the line has a point: elsewhere the renderer
/// would add a hover target the marks do not know. A stack draws none, its lines at sums.
fn band(spec: &LineSpec, series: usize) -> Option<charts_rs::SeriesBand> {
    let ranges = spec.errors.get(series).filter(|_| !spec.stacked)?;
    band_of(ranges, &spec.series.get(series)?.values)
}

fn band_of(ranges: &[Option<(f64, f64)>], values: &[Option<f64>]) -> Option<charts_rs::SeriesBand> {
    let at = |i: usize| {
        values
            .get(i)
            .copied()
            .flatten()
            .and(ranges.get(i).copied().flatten())
    };
    let end = |pick: fn((f64, f64)) -> f64| -> Vec<Option<f32>> {
        (0..values.len())
            .map(|i| at(i).map(|range| pick(range) as f32))
            .collect()
    };
    let lower = end(|range| range.0);
    lower
        .iter()
        .any(Option::is_some)
        .then(|| charts_rs::SeriesBand {
            lower,
            upper: end(|range| range.1),
            stroke_width: None,
        })
}

fn bend(drawn: &mut charts_rs::Series, curve: Curve) {
    match curve {
        Curve::Linear => {}
        Curve::Smooth => drawn.smooth = Some(true),
        Curve::Step => drawn.step = Some(LineStep::End),
    }
}

/// A right y axis like the left one, its labels behind [`AXIS2_MARK`].
fn second_axis(y: &mut Vec<charts_rs::YAxisConfig>, unit: &Option<String>) {
    if let Some(left) = y.first() {
        let second = charts_rs::YAxisConfig {
            formatter: Some(format!("{AXIS2_MARK}{{c}}{}", unit_suffix(unit.as_deref()))),
            // `y_min` and `y_max` are the left axis' only.
            min: None,
            max: None,
            ..left.clone()
        };
        y.truncate(1);
        y.push(second);
    }
}

pub(crate) fn draw_line(spec: &LineSpec) -> Option<Drawn> {
    // A stack draws its lines at their sums, not at their own values.
    let marked = match spec.stacked {
        true => Vec::new(),
        false => kept_mark_lines(&spec.series, &spec.axes),
    };
    let series = spec
        .series
        .iter()
        .enumerate()
        .map(|(i, series)| {
            let values = series.values.iter().map(|v| v.map(|v| v as f32)).collect();
            let mut drawn = charts_rs::Series::new_nullable(series.name.clone(), values);
            drawn.colors = Some(vec![Some(ink(series.color))]);
            if let Some(lines) = marked.get(i) {
                drawn.mark_lines = lines.iter().copied().map(renderer_mark_line).collect();
            }
            drawn.stroke_dash_array = series.dashed.then(|| DASH.to_string());
            drawn.y_axis_index = series.extras.secondary as usize;
            drawn.band = band(spec, i);
            drawn.symbol = Some(symbol(spec.points, spec.stroke_width, series));
            drawn.stack = (spec.stacked && !series.extras.secondary).then(|| "stack".to_string());
            bend(&mut drawn, spec.curve);
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
    chart.series.fill = spec.area.is_some();
    chart.series.fill_opacity = spec.area;
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
    } else {
        // Every label drawn, so `fit_x_labels` finds each.
        chart.x_axis.label_overflow = AxisLabelOverflow::Ellipsis;
    }
    let unit = |unit: &Option<String>| {
        unit.as_deref()
            .map(|unit| format!(" {unit}"))
            .unwrap_or_default()
    };
    if let Some(y) = chart.y_axis_configs.first_mut() {
        y.stroke_color = ink(spec.ink.axis);
        y.font.color = ink(spec.ink.label);
        y.formatter = Some(format!("{AXIS_MARK}{{c}}{}", unit(&spec.unit)));
    }
    if spec.series.iter().any(|series| series.extras.secondary) {
        second_axis(&mut chart.y_axis_configs, &spec.y2_unit);
    }
    spec.axes
        .style(&mut chart.base, &mut chart.y_axis_configs, false);
    round_axes(&chart.base, &mut chart.y_axis_configs);
    let auto = spec.format == ValueFormat::Auto;
    let x_formats = ungrouped(spec.formats);
    let write = |svg: &str, mark: char| {
        let (unit, formats) = match mark {
            X_MARK => (None, &x_formats),
            AXIS2_MARK => (spec.y2_unit.as_deref(), spec.formats),
            _ => (spec.unit.as_deref(), spec.formats),
        };
        let compact = &spec.axes.compact;
        let plain = move |label| spec.format.format(label, unit, formats);
        axis_write(svg, mark, auto, unit, formats, compact, plain)
    };
    let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
        let mut svg = chart.svg().ok()?;
        let exact = exact_axes(&svg, &chart.y_axis_configs, false);
        let widths = [
            y_width(&svg, &exact, AXIS_MARK, write(&exact, AXIS_MARK)),
            y_width(&svg, &exact, AXIS2_MARK, write(&exact, AXIS2_MARK)),
        ];
        if widths.iter().any(Option::is_some) {
            for (y, width) in chart.y_axis_configs.iter_mut().zip(widths) {
                y.width = width.or(y.width);
            }
            svg = chart.svg().ok()?;
        }
        Some(svg)
    })?;

    let expected: Vec<_> = expected_marks(&spec.series);
    // Pinned by the renderer's own labels, then written in full.
    let svg = exact_axes(&pin_to_axis(&svg), &chart.y_axis_configs, false);
    let svg = match spec.x_numbers.is_empty() {
        true => fit_x_labels(&svg, &spec.x, spec.width, spec.height),
        false => svg,
    };
    let (svg, triggers) = sanitize(&svg, write(&svg, AXIS_MARK));
    let svg = relabel(&svg, AXIS2_MARK, write(&svg, AXIS2_MARK));
    let write_x = write(&svg, X_MARK);
    let svg = pin_to_ticks(&svg, X_MARK, spec.width, &write_x);
    let svg = relabel(&svg, X_MARK, write_x);
    let marks = match_marks(&expected, &triggers);
    let texts: Vec<&str> = marked
        .iter()
        .flatten()
        .map(|(_, text)| text.as_str())
        .collect();
    let plot = Plot::of(&svg, spec.width, spec.height);
    let svg = label_mark_lines(&svg, &texts, &spec.ink, plot.left);
    let svg = pin_mark_points(&svg, &spec.series, spec.width, &spec.ink, &marks);
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot,
            marks,
            hole: 0.0,
            share_of: None,
            cells: false,
        },
        svg: spec.ink.paint(svg),
    })
}

/// A scatter series as drawn: its finite points, each with its place in the series.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScatterSpecSeries {
    pub(crate) name: String,
    pub(crate) points: Vec<(usize, f64, f64, Option<f64>)>,
    pub(crate) color: HexColor,
    pub(crate) symbol: PointSymbol,
}

/// Everything a scatter chart's SVG depends on.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScatterSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) series: Vec<ScatterSpecSeries>,
    /// Sized points: every series' points are bubbles, a point without a size the middle size.
    pub(crate) bubble: bool,
    pub(crate) trend: Option<Trend>,
    /// The y axis' ends and steps, both titles.
    pub(crate) axes: Axes,
    pub(crate) x_min: Option<f64>,
    pub(crate) x_max: Option<f64>,
    pub(crate) grid: bool,
    /// A plain point's radius.
    pub(crate) radius: f32,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

/// A scatter point's marker, sized so every shape covers about a circle's area.
fn scatter_symbol(symbol: PointSymbol, r: f32) -> Symbol {
    match symbol {
        PointSymbol::Circle => Symbol::Circle(r, None),
        PointSymbol::Square => Symbol::Rect(r * 0.9, None),
        PointSymbol::Triangle => Symbol::Triangle(r * 1.35, None),
        PointSymbol::Diamond => Symbol::Diamond(r * 1.25, None),
    }
}

/// The scatter's points in the renderer's order: bubbles largest first, as it draws them.
fn scatter_marks(spec: &ScatterSpec) -> Vec<(usize, usize, f64, &str)> {
    spec.series
        .iter()
        .enumerate()
        .flat_map(|(s, series)| {
            let mut points: Vec<_> = series.points.iter().collect();
            if spec.bubble {
                let size = |p: &(usize, f64, f64, Option<f64>)| p.3.map_or(0.0, |v| v as f32);
                points.sort_by(|a, b| size(b).total_cmp(&size(a)));
            }
            points
                .into_iter()
                .map(move |&(i, _, y, _)| (s, i, y, series.name.as_str()))
        })
        .collect()
}

pub(crate) fn draw_scatter(spec: &ScatterSpec) -> Option<Drawn> {
    let series = spec
        .series
        .iter()
        .map(|series| {
            let values = series
                .points
                .iter()
                .flat_map(|&(_, x, y, size)| {
                    let size = spec.bubble.then_some(size.map(|v| v as f32));
                    [Some(x as f32), Some(y as f32)].into_iter().chain(size)
                })
                .collect();
            charts_rs::Series::new_nullable(series.name.clone(), values)
        })
        .collect();
    let mut chart = charts_rs::ScatterChart::new(series);
    chart.width = spec.width;
    chart.height = spec.height;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.series.colors = spec.series.iter().map(|s| ink(s.color)).collect();
    chart.series.stroke_width = 2.0;
    chart.bubble = spec.bubble;
    // The largest bubble a tenth of the plot's short side, so a small chart is not buried.
    chart.bubble_min_size = spec.radius;
    chart.bubble_max_size = (spec.width.min(spec.height) / 10.0).clamp(spec.radius * 2.0, 30.0);
    chart.series_symbol_sizes = vec![spec.radius; spec.series.len()];
    chart.series_symbols = spec
        .series
        .iter()
        .map(|s| match spec.bubble {
            // Translucent, so overlapping bubbles and the trend lines show through.
            true => Symbol::Circle(spec.radius, Some(ink(s.color).with_alpha(150))),
            false => scatter_symbol(s.symbol, spec.radius),
        })
        .collect();
    chart.regression = spec.trend.map(|trend| match trend {
        Trend::Linear => charts_rs::Regression::Linear,
        Trend::Exponential => charts_rs::Regression::Exponential,
        Trend::Logarithmic => charts_rs::Regression::Logarithmic,
        Trend::Polynomial(_) => charts_rs::Regression::Polynomial,
    });
    if let Some(Trend::Polynomial(order)) = spec.trend {
        chart.regression_order = usize::from(order.clamp(2, 6));
    }
    chart.grid.stroke_color = match spec.grid {
        true => ink(spec.ink.grid),
        false => InkColor::transparent(),
    };
    chart.x_axis.stroke_color = ink(spec.ink.axis);
    chart.x_axis.font.color = ink(spec.ink.label);
    if let Some(y) = chart.y_axis_configs.first_mut() {
        y.stroke_color = ink(spec.ink.axis);
        y.font.color = ink(spec.ink.label);
        y.formatter = Some(format!(
            "{AXIS_MARK}{{c}}{}",
            unit_suffix(spec.unit.as_deref())
        ));
    }
    spec.axes
        .style(&mut chart.base, &mut chart.y_axis_configs, false);
    // Both axes fit the data: a scatter reads its spread, not its distance from 0.
    let column = |values: Vec<f64>| {
        [charts_rs::Series::new(
            String::new(),
            values.into_iter().map(|v| v as f32).collect(),
        )]
    };
    let points = || spec.series.iter().flat_map(|s| &s.points);
    if let Some(y) = chart.y_axis_configs.first_mut() {
        round_axis(&column(points().map(|p| p.2).collect()), 0, y, None, false);
    }
    let mut x = chart.y_axis_configs.first().cloned().unwrap_or_default();
    x.title = None;
    x.min = spec.x_min.map(|min| min as f32);
    x.max = spec.x_max.map(|max| max as f32);
    x.formatter = Some(format!("{X_MARK}{{c}}"));
    round_axis(
        &column(points().map(|p| p.1).collect()),
        0,
        &mut x,
        None,
        false,
    );
    chart.x_axis_config = x;

    let auto = spec.format == ValueFormat::Auto;
    // A scatter's x is a measure, grouped as its y; not a year.
    let write = |svg: &str, mark: char| {
        let unit = (mark != X_MARK).then_some(spec.unit.as_deref()).flatten();
        let (formats, compact) = (spec.formats, &spec.axes.compact);
        let plain = move |label| spec.format.format(label, unit, formats);
        axis_write(svg, mark, auto, unit, formats, compact, plain)
    };
    let exact = |svg: &str, chart: &charts_rs::ScatterChart| {
        let svg = exact_axes(svg, &chart.y_axis_configs, false);
        exact_ticks(&svg, X_MARK, &chart.x_axis_config, true)
    };
    let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
        let svg = chart.svg().ok()?;
        let full = exact(&svg, chart);
        match y_width(&svg, &full, AXIS_MARK, write(&full, AXIS_MARK)) {
            Some(width) => {
                if let Some(y) = chart.y_axis_configs.first_mut() {
                    y.width = Some(width);
                }
                chart.svg().ok()
            }
            None => Some(svg),
        }
    })?;

    let svg = trends_on_top(&exact(&pin_to_axis(&svg), &chart));
    let (svg, triggers) = inert(&svg, write(&svg, AXIS_MARK), "", true);
    let write_x = write(&svg, X_MARK);
    let svg = pin_to_ticks(&svg, X_MARK, spec.width, &write_x);
    let svg = relabel(&svg, X_MARK, write_x);
    // A shape's centre is its point.
    let triggers: Vec<Trigger> = triggers
        .into_iter()
        .map(|trigger| {
            let (x, y) = match trigger.shape {
                MarkShape::Rect {
                    x,
                    y,
                    width,
                    height,
                } => (x + width / 2.0, y + height / 2.0),
                MarkShape::Point { x, y } => (x, y),
                // A triangle or diamond: its corners' middle.
                MarkShape::Sector { ref path } => {
                    let corners: Vec<f32> = path
                        .split_whitespace()
                        .filter_map(|n| n.parse().ok())
                        .collect();
                    let count = (corners.len() / 2).max(1) as f32;
                    let sum = |odd: usize| corners.iter().skip(odd).step_by(2).sum::<f32>();
                    (sum(0) / count, sum(1) / count)
                }
            };
            Trigger {
                shape: MarkShape::Point { x, y },
                ..trigger
            }
        })
        .collect();
    let marks = match_marks(&scatter_marks(spec), &triggers);
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot::of(&svg, spec.width, spec.height),
            marks,
            hole: 0.0,
            share_of: None,
            cells: false,
        },
        svg: spec.ink.paint(svg),
    })
}

/// The trend lines moved after every point: the renderer draws each after its own series', so
/// a later series' points would hide it.
fn trends_on_top(svg: &str) -> String {
    let (mut rest, mut lines) = (svg.to_string(), String::new());
    while let Some(at) = rest.find("class=\"ct-regression\"") {
        let (Some(start), Some(len)) = (rest[..at].rfind('<'), rest[at..].find("/>")) else {
            break;
        };
        lines.extend(rest.drain(start..at + len + 2));
    }
    match rest.rfind("</svg>") {
        Some(end) => rest.insert_str(end, &lines),
        None => rest.push_str(&lines),
    }
    rest
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
    pub(crate) axes: Axes,
    /// The value labels in the middle of their bars rather than past their ends.
    pub(crate) labels_inside: bool,
    /// Per series, a bar's own colour per x value; `None`, the series' colour.
    pub(crate) bar_colors: Vec<Vec<Option<HexColor>>>,
    /// Per series, each value's error range; a percent stack draws none.
    pub(crate) errors: Vec<Vec<Option<(f64, f64)>>>,
    /// The share of a category's band its bars fill; `None`, the renderer's 5 px margins.
    pub(crate) bar_width: Option<f32>,
    /// Between two bars of a group, in pixels.
    pub(crate) bar_gap: f32,
    pub(crate) grid: bool,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    /// The right y axis' unit, of its lines.
    pub(crate) y2_unit: Option<String>,
    /// The lines' look, as a line chart's.
    pub(crate) curve: Curve,
    pub(crate) points: bool,
    pub(crate) stroke_width: f32,
    pub(crate) formats: &'static Formats,
}

/// The marks in the renderer's order: every bar series', then every line's.
fn bar_marks(series: &[SeriesSpec]) -> Vec<(usize, usize, f64, &str)> {
    let (lines, bars): (Vec<_>, Vec<_>) = expected_marks(series)
        .into_iter()
        .partition(|mark| series[mark.0].extras.line);
    bars.into_iter().chain(lines).collect()
}

/// The settings a vertical and a horizontal bar chart share; both deref to the same base.
macro_rules! style_bars {
    ($chart:ident, $spec:ident, $axes:ident, $unit:expr) => {
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
        $axes.style(
            &mut $chart.base,
            &mut $chart.y_axis_configs,
            $spec.horizontal,
        );
        let inside = $spec.labels_inside;
        $chart.series_label_position = inside.then_some(Position::Inside);
        $chart.series.label.font.color = match inside {
            true => ON_BAR,
            false => ink($spec.ink.label),
        };
    };
}

/// A percent stack with losses reads each share of the stack's absolute sum, up above 0 and
/// down below it: the axis runs to the furthest of each, in steps of 20 (else the renderer's).
fn percent_axes(axes: &Axes, series: &[SeriesSpec]) -> Axes {
    const STEP: f64 = 20.0;
    let bars: Vec<_> = series.iter().filter(|s| !s.extras.line).collect();
    let (mut up, mut down) = (0.0_f64, 0.0_f64);
    for index in 0..bars.iter().map(|s| s.values.len()).max().unwrap_or(0) {
        let values = bars
            .iter()
            .filter_map(|s| s.values.get(index).copied().flatten());
        let (gain, loss) = values.fold((0.0, 0.0), |(gain, loss), v: f64| {
            (gain + v.max(0.0), loss + (-v).max(0.0))
        });
        let total = gain + loss;
        if total > 0.0 {
            up = up.max(gain / total * 100.0);
            down = down.max(loss / total * 100.0);
        }
    }
    if down == 0.0 {
        return axes.clone();
    }
    let (min, max) = (-(down / STEP).ceil() * STEP, (up / STEP).ceil() * STEP);
    Axes {
        min: Some(min),
        max: Some(max),
        ticks: ((max - min) / STEP) as usize,
        ..axes.clone()
    }
}

/// Puts axis `index`'s unset ends on whole steps of 1, 2, 2.5 or 5 times a power of ten, the
/// step count nearest its `split_number`, or exactly `count` steps; set ends stay. With
/// `zero`, the axis spans 0 too.
fn round_axis(
    list: &[charts_rs::Series],
    index: usize,
    config: &mut charts_rs::YAxisConfig,
    count: Option<usize>,
    zero: bool,
) {
    if config.min.is_some() && config.max.is_some() {
        return;
    }
    // What the renderer spans: each value, each band's ends and each stack's sums.
    let mut values = Vec::new();
    let mut stacks = std::collections::BTreeMap::<(&str, usize), (f64, f64)>::new();
    for series in list.iter().filter(|s| s.y_axis_index == index) {
        for (i, value) in series.data.iter().enumerate() {
            let Some(value) = value.filter(|v| v.is_finite()).map(f64::from) else {
                continue;
            };
            match &series.stack {
                None => values.push(value),
                Some(key) => {
                    let sums = stacks.entry((key, i + series.start_index)).or_default();
                    match value >= 0.0 {
                        true => sums.0 += value,
                        false => sums.1 += value,
                    }
                }
            }
        }
        for bounds in [&series.band, &series.error_bar].into_iter().flatten() {
            let ends = bounds.lower.iter().chain(&bounds.upper).flatten();
            values.extend(ends.filter(|v| v.is_finite()).map(|v| f64::from(*v)));
        }
    }
    values.extend(stacks.into_values().flat_map(|(up, down)| [up, down]));
    if values.is_empty() {
        return;
    }
    let (min, max) = (config.min.map(f64::from), config.max.map(f64::from));
    let start = if zero { 0.0 } else { values[0] };
    let low = min.unwrap_or(values.iter().fold(start, |low, v| v.min(low)));
    let high = max.unwrap_or(values.iter().fold(start, |high, v| v.max(high)));
    // Only zeros: an axis up to 1.
    let high = match (high > low, max) {
        (false, None) => low + 1.0,
        _ => high,
    };
    if high.partial_cmp(&low) != Some(std::cmp::Ordering::Greater) {
        return;
    }
    let target = count.unwrap_or(config.split_number).max(1);
    let ends = |step: f64| {
        let steps = |span: f64| (span / step - 1e-9).ceil() * step;
        match (min, max) {
            (Some(min), _) => (min, min + steps(high - min)),
            (_, Some(max)) => (max - steps(max - low), max),
            _ => (
                (low / step + 1e-9).floor() * step,
                (high / step - 1e-9).ceil() * step,
            ),
        }
    };
    let steps = |step: f64| {
        let (low, high) = ends(step);
        ((high - low) / step).round() as usize
    };
    let power = 10_f64.powf(((high - low) / target as f64).log10().floor());
    let candidates = [0.1, 1.0, 10.0]
        .into_iter()
        .flat_map(|scale| [1.0, 2.0, 2.5, 5.0].map(|m| m * scale * power));
    // Exactly `count`: the finest step that fits. Else the nearest count, then the tightest ends.
    let step = match count {
        Some(count) => candidates
            .filter(|step| steps(*step) <= count)
            .reduce(f64::min),
        None => candidates.min_by(|a, b| {
            let off = |step: f64| steps(step).abs_diff(target);
            let span = |step: f64| ends(step).1 - ends(step).0;
            let tighter = span(*a).total_cmp(&span(*b));
            off(*a).cmp(&off(*b)).then(tighter).then(b.total_cmp(a))
        }),
    };
    let Some(step) = step else { return };
    let (low, _) = ends(step);
    let n = count.unwrap_or(steps(step)).max(1);
    let high = match (min, max) {
        (_, Some(max)) => max,
        _ => low + step * n as f64,
    };
    let low = match max {
        Some(max) => max - step * n as f64,
        None => low,
    };
    config.min = Some(low as f32);
    config.max = Some(high as f32);
    config.split_number = n;
}

/// [`round_axis`] on the left axis, and the right one cut into as many steps.
fn round_axes(base: &charts_rs::ChartBase, y: &mut [charts_rs::YAxisConfig]) {
    if let Some(left) = y.first_mut() {
        round_axis(&base.series_list, 0, left, None, true);
    }
    let count = y.first().map(|left| left.split_number);
    if let Some(right) = y.get_mut(1) {
        round_axis(&base.series_list, 1, right, count, true);
    }
}

/// [`exact_ticks`] on both value axes.
fn exact_axes(svg: &str, y: &[charts_rs::YAxisConfig], across: bool) -> String {
    let svg = match y.first() {
        Some(left) => exact_ticks(svg, AXIS_MARK, left, across),
        None => svg.to_string(),
    };
    match y.get(1) {
        Some(right) => exact_ticks(&svg, AXIS2_MARK, right, false),
        None => svg,
    }
}

/// Writes axis `mark`'s labels in full, as `config` cuts it: the renderer keeps one decimal.
/// `across`: the labels run along the bottom, else up the side.
fn exact_ticks(svg: &str, mark: char, config: &charts_rs::YAxisConfig, across: bool) -> String {
    let (Some(min), Some(max)) = (config.min, config.max) else {
        return svg.to_string();
    };
    let n = config.split_number.max(1);
    let step = (f64::from(max) - f64::from(min)) / n as f64;
    let mut at = Vec::new();
    let mut rest = 0;
    while let Some(open) = svg[rest..].find("<text").map(|open| rest + open) {
        let Some(close) = svg[open..].find('>').map(|close| open + close) else {
            break;
        };
        let attributes = attributes(&svg[open..=close]);
        let label = &svg[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim_start();
        if label.starts_with(mark) {
            let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
            let place = number(if across { "x" } else { "y" }).unwrap_or(0.0);
            let start = svg.len() - svg[close + 1..].trim_start().len() + mark.len_utf8();
            let length = svg[start..].find([' ', '\n', '<']).unwrap_or(0);
            at.push((place, start, length));
        }
        rest = close + 1;
    }
    if at.len() != n + 1 {
        return svg.to_string();
    }
    // Lowest first: leftmost along the bottom, lowest up the side.
    at.sort_by(|a, b| match across {
        true => a.0.total_cmp(&b.0),
        false => b.0.total_cmp(&a.0),
    });
    let mut values: Vec<_> = at
        .iter()
        .enumerate()
        .map(|(i, &(_, start, length))| (start, length, f64::from(min) + step * i as f64))
        .collect();
    values.sort_by_key(|value| value.0);
    let mut out = String::with_capacity(svg.len());
    let mut last = 0;
    for (start, length, value) in values {
        out.push_str(&svg[last..start]);
        out.push_str(&format!("{value}"));
        last = start + length;
    }
    out.push_str(&svg[last..]);
    out
}

/// The value axis of bars whose ranges the renderer does not know of, stacks and lying bars:
/// unset ends reach the furthest range end, on whole steps of a round size.
fn error_axes(axes: &Axes, spec: &BarSpec) -> Axes {
    let stacked = spec.layout != BarLayout::Grouped;
    let (mut low, mut high) = (0.0_f64, 0.0_f64);
    for index in 0..spec.x.len() {
        let (mut up, mut down) = (0.0_f64, 0.0_f64);
        for (s, series) in spec.series.iter().enumerate() {
            let value = series.values.get(index).copied().flatten();
            let Some(value) = value.filter(|_| !series.extras.line && !series.extras.secondary)
            else {
                continue;
            };
            let base = match (stacked, value >= 0.0) {
                (false, _) => 0.0,
                (true, true) => up,
                (true, false) => down,
            };
            let range = spec
                .errors
                .get(s)
                .and_then(|e| e.get(index).copied().flatten());
            for end in [value]
                .into_iter()
                .chain(range.into_iter().flat_map(|r| [r.0, r.1]))
            {
                low = low.min(base + end);
                high = high.max(base + end);
            }
            match value >= 0.0 {
                true => up += value,
                false => down += value,
            }
        }
    }
    let raw = (high - low) / axes.ticks.max(1) as f64;
    if axes.min.is_some() || axes.max.is_some() || raw <= 0.0 || !raw.is_finite() {
        return axes.clone();
    }
    let power = 10_f64.powf(raw.log10().floor());
    let step = power
        * [1.0, 2.0, 2.5, 5.0, 10.0]
            .into_iter()
            .find(|n| raw / power <= *n)
            .unwrap_or(10.0);
    let (min, max) = ((low / step).floor() * step, (high / step).ceil() * step);
    Axes {
        min: Some(min),
        max: Some(max),
        ticks: ((max - min) / step).round().max(1.0) as usize,
        ..axes.clone()
    }
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
    // The specs carry mark lines only where a mark stands at its own value.
    let marked = match !spec.horizontal {
        true => kept_mark_lines(&ordered, &spec.axes),
        false => Vec::new(),
    };
    let bar_color = |series: usize, index: usize| {
        spec.bar_colors
            .get(series)
            .and_then(|colors| colors.get(index).copied().flatten())
            .unwrap_or(spec.series[series].color)
    };
    // The renderer draws a range on upright grouped bars only; the rest are drawn after it.
    let errors = spec.layout == BarLayout::Grouped && !spec.horizontal;
    let series = ordered
        .iter()
        .enumerate()
        .map(|(s, series)| {
            let values = series.values.iter().map(|v| v.map(|v| v as f32)).collect();
            let mut drawn = charts_rs::Series::new_nullable(series.name.clone(), values);
            if let Some(lines) = marked.get(s) {
                drawn.mark_lines = lines.iter().copied().map(renderer_mark_line).collect();
            }
            // A line keeps out of the stacks and the bars' slots, its colour by its place.
            if series.extras.line {
                drawn.category = Some(SeriesCategory::Line);
                drawn.index = Some(s);
                drawn.stroke_dash_array = series.dashed.then(|| DASH.to_string());
                drawn.y_axis_index = series.extras.secondary as usize;
                drawn.symbol = Some(symbol(spec.points, spec.stroke_width, series));
                let ranges: Vec<_> = order
                    .iter()
                    .map(|&i| spec.errors.get(s).and_then(|e| e.get(i).copied().flatten()))
                    .collect();
                drawn.band = band_of(&ranges, &series.values);
                bend(&mut drawn, spec.curve);
                return drawn;
            }
            drawn.label_show = spec.value_labels;
            drawn.y_axis_index = series.extras.secondary as usize;
            drawn.stack = (spec.layout != BarLayout::Grouped).then(|| "stack".to_string());
            drawn.colors = Some(order.iter().map(|&i| Some(ink(bar_color(s, i)))).collect());
            let ranges: Vec<Option<(f64, f64)>> = order
                .iter()
                .map(|&i| spec.errors.get(s).and_then(|e| e.get(i).copied().flatten()))
                .collect();
            if errors && ranges.iter().any(Option::is_some) {
                let end = |pick: fn((f64, f64)) -> f64| {
                    ranges.iter().map(|r| r.map(|r| pick(r) as f32)).collect()
                };
                drawn.error_bar = Some(charts_rs::SeriesBand {
                    lower: end(|r| r.0),
                    upper: end(|r| r.1),
                    stroke_width: None,
                });
            }
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
    let write2 = |value: f64| {
        spec.format
            .format(value, spec.y2_unit.as_deref(), spec.formats)
    };
    let auto = spec.format == ValueFormat::Auto;
    let (unit1, compact) = (spec.unit.as_deref(), &spec.axes.compact);
    let axis = |svg: &str| {
        axis_write(
            svg,
            AXIS_MARK,
            auto && !percent,
            unit1,
            spec.formats,
            compact,
            write,
        )
    };
    let axis2 = |svg: &str| {
        let unit = spec.y2_unit.as_deref();
        axis_write(svg, AXIS2_MARK, auto, unit, spec.formats, compact, write2)
    };
    let ranged = (spec.series.iter().zip(&spec.errors))
        .any(|(series, errors)| !series.extras.line && errors.iter().any(Option::is_some));
    let axes = match (percent, ranged && !errors) {
        (true, _) => percent_axes(&spec.axes, &ordered),
        (false, true) => error_axes(&spec.axes, spec),
        (false, false) => spec.axes.clone(),
    };
    let svg = match spec.horizontal {
        true => {
            let mut chart = charts_rs::HorizontalBarChart::new(series, x);
            style_bars!(chart, spec, axes, unit);
            chart.stack_percent = percent;
            if !percent {
                round_axes(&chart.base, &mut chart.y_axis_configs);
            }
            let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
                let svg = chart.svg().ok()?;
                let exact = exact_axes(&svg, &chart.y_axis_configs, true);
                Some(pin_to_ticks(&svg, AXIS_MARK, spec.width, axis(&exact)))
            })?;
            exact_axes(&svg, &chart.y_axis_configs, true)
        }
        false => {
            let mut chart = charts_rs::BarChart::new(series, x.clone());
            style_bars!(chart, spec, axes, unit);
            chart.x_axis.label_overflow = AxisLabelOverflow::Ellipsis;
            chart.stack_percent = percent;
            chart.series.stroke_width = spec.stroke_width;
            if spec.series.iter().any(|s| s.extras.secondary) {
                second_axis(&mut chart.y_axis_configs, &spec.y2_unit);
                axes.style(&mut chart.base, &mut chart.y_axis_configs, false);
            }
            if !percent {
                round_axes(&chart.base, &mut chart.y_axis_configs);
            }
            let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
                let mut svg = chart.svg().ok()?;
                let exact = exact_axes(&svg, &chart.y_axis_configs, false);
                let widths = [
                    y_width(&svg, &exact, AXIS_MARK, axis(&exact)),
                    y_width(&svg, &exact, AXIS2_MARK, axis2(&exact)),
                ];
                if widths.iter().any(Option::is_some) {
                    for (y, width) in chart.y_axis_configs.iter_mut().zip(widths) {
                        y.width = width.or(y.width);
                    }
                    svg = chart.svg().ok()?;
                }
                Some(pin_to_axis(&svg))
            })?;
            let svg = exact_axes(&svg, &chart.y_axis_configs, false);
            fit_x_labels(&svg, &x, spec.width, spec.height)
        }
    };

    let (svg, triggers) = inert(&svg, axis(&svg), "", false);
    let svg = relabel(&svg, AXIS2_MARK, axis2(&svg));
    // The renderer's margins would leave a narrow band's bars a sliver.
    let narrow = {
        let plot = Plot::of(&svg, spec.width, spec.height);
        let span = match spec.horizontal {
            true => plot.bottom - plot.top,
            false => plot.right - plot.left,
        };
        span < 4.0 * RENDERER_MARGIN * spec.x.len() as f32
    };
    let (svg, triggers) = match narrow || spec.bar_width.is_some() || spec.bar_gap != RENDERER_GAP {
        true => relayout(&svg, triggers, &bar_marks(&ordered), spec),
        false => (svg, triggers),
    };
    let negative: Vec<bool> = bar_marks(&ordered)
        .iter()
        .map(|&(_, _, value, _)| value < 0.0)
        .collect();
    let svg = round_free_ends(&svg, &triggers, &negative, spec.horizontal, spec.radius);
    let svg = match spec.horizontal || spec.labels_inside {
        true => svg,
        false => place_outside_labels(
            &svg,
            Plot::of(&svg, spec.width, spec.height).bottom,
            spec.ink.surface,
        ),
    };
    // The renderer writes its f32s; a label writes the value its bar was drawn from.
    let values: Vec<f64> = (ordered.iter().filter(|s| !s.extras.line))
        .flat_map(|s| s.values.iter().flatten().copied())
        .collect();
    let exact = |label: f64| {
        let near = |value: &f64| (value - label).abs() <= value.abs() * 1e-6;
        values.iter().copied().find(near).unwrap_or(label)
    };
    let svg = relabel(&svg, LABEL_MARK, |label| match percent {
        true => write(label),
        false => write(exact(label)),
    });
    let last = spec.x.len().saturating_sub(1);
    let marks = match_marks(&bar_marks(&ordered), &triggers)
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
    let svg = ink_bar_labels(&svg, &marks, |mark| {
        spec.ink.on(bar_color(mark.series, mark.index))
    });
    let svg = draw_error_bars(svg, &marks, spec);
    // An error bar is drawn in the labels' ink; one on a bar is in the page's.
    let svg = svg.replace(
        "stroke=\"#010203\"",
        &format!("stroke=\"{}\"", spec.ink.label),
    );
    let texts: Vec<&str> = marked
        .iter()
        .flatten()
        .map(|(_, text)| text.as_str())
        .collect();
    let plot = Plot::of(&svg, spec.width, spec.height);
    let svg = label_mark_lines(&svg, &texts, &spec.ink, plot.left);
    let svg = pin_mark_points(&svg, &spec.series, spec.width, &spec.ink, &marks);
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot,
            marks,
            hole: 0.0,
            share_of: None,
            cells: false,
        },
        svg: spec.ink.paint(svg),
    })
}

/// The bars as paths rounded only at the free end, the one away from the axis they grow
/// from (the top of a rising bar, the bottom of a falling one, the right end of a lying
/// one); the end at the axis and an end against another bar stay square.
fn round_free_ends(
    svg: &str,
    triggers: &[Trigger],
    negative: &[bool],
    horizontal: bool,
    radius: f32,
) -> String {
    let bars: Vec<[f32; 4]> = triggers
        .iter()
        .filter_map(|trigger| match trigger.shape {
            MarkShape::Rect {
                x,
                y,
                width,
                height,
            } => Some([x, y, x + width, y + height]),
            _ => None,
        })
        .collect();
    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
    let overlap = |a0: f32, a1: f32, b0: f32, b1: f32| a0 < b1 - 0.5 && b0 < a1 - 0.5;
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find("<rect ") {
        out.push_str(&rest[..open]);
        let close = rest[open..]
            .find("/>")
            .map_or(rest.len(), |close| open + close + 2);
        let tag = &rest[open..close];
        rest = &rest[close..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let (Some(x), Some(y), Some(width), Some(height)) =
            (number("x"), number("y"), number("width"), number("height"))
        else {
            out.push_str(tag);
            continue;
        };
        let [left, top, right, bottom] = [x, y, x + width, y + height];
        let own = [left, top, right, bottom];
        let falls = bars
            .iter()
            .position(|bar| *bar == own)
            .and_then(|i| negative.get(i))
            .copied()
            .unwrap_or(false);
        let mut touched = false;
        for &[l, t, r, b] in bars.iter().filter(|other| **other != own) {
            touched |= match (horizontal, falls) {
                (false, false) => near(b, top) && overlap(l, r, left, right),
                (false, true) => near(t, bottom) && overlap(l, r, left, right),
                (true, false) => near(l, right) && overlap(t, b, top, bottom),
                (true, true) => near(r, left) && overlap(t, b, top, bottom),
            };
        }
        let r = match touched {
            true => 0.0,
            false => radius.min(width / 2.0).min(height / 2.0).max(0.0),
        };
        let [tl, tr, br, bl] = match (horizontal, falls) {
            (false, false) => [r, r, 0.0, 0.0],
            (false, true) => [0.0, 0.0, r, r],
            (true, false) => [0.0, r, r, 0.0],
            (true, true) => [r, 0.0, 0.0, r],
        };
        let d = format!(
            "M{},{top} H{} A{tr},{tr} 0 0 1 {right},{} V{} A{br},{br} 0 0 1 {},{bottom} H{} A{bl},{bl} 0 0 1 {left},{} V{} A{tl},{tl} 0 0 1 {},{top} Z",
            left + tl,
            right - tr,
            top + tr,
            bottom - br,
            right - br,
            left + bl,
            bottom - bl,
            top + tl,
            left + tl,
        );
        let kept: String = attributes
            .iter()
            .filter(|(name, _)| !matches!(*name, "x" | "y" | "width" | "height"))
            .map(|(name, value)| format!(" {name}=\"{value}\""))
            .collect();
        out.push_str(&format!("<path d=\"{d}\"{kept}/>"));
    }
    out.push_str(rest);
    out
}

/// The bars set again across their axis at the spec's width and gap, each centred in its
/// band as the renderer centres them; their labels and error bars move with their centres.
fn relayout(
    svg: &str,
    triggers: Vec<Trigger>,
    expected: &[(usize, usize, f64, &str)],
    spec: &BarSpec,
) -> (String, Vec<Trigger>) {
    let count = spec.x.len();
    if expected.len() != triggers.len() || count == 0 {
        return (svg.to_string(), triggers);
    }
    let plot = Plot::of(svg, spec.width, spec.height);
    let (start, span) = match spec.horizontal {
        true => (plot.top, plot.bottom - plot.top),
        false => (plot.left, plot.right - plot.left),
    };
    let grouped = spec.layout == BarLayout::Grouped;
    // Lines take no slot; a stack takes one per y axis, as the renderer keys them.
    let bars: Vec<usize> = (0..spec.series.len())
        .filter(|s| !spec.series[*s].extras.line)
        .collect();
    let mut keys: Vec<usize> = Vec::new();
    for &bar in &bars {
        let key = match grouped {
            true => bar,
            false => spec.series[bar].extras.secondary as usize,
        };
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    let slot_of = |series: usize| {
        let key = match grouped {
            true => series,
            false => spec.series[series].extras.secondary as usize,
        };
        keys.iter().position(|k| *k == key).unwrap_or(0)
    };
    let slots = keys.len().max(1);
    let unit = span / count as f32;
    let group = match spec.bar_width {
        Some(share) => unit * share.clamp(0.1, 1.0),
        // A band narrower than the margins keeps half of it.
        None => (unit - 2.0 * RENDERER_MARGIN).max(unit / 2.0),
    };
    let gap = spec.bar_gap.max(0.0).min(group / slots as f32 / 2.0);
    let width = ((group - gap * (slots - 1) as f32) / slots as f32).max(1.0);
    // Old `(from, to)` across the axis to the new ones, one per drawn bar.
    let mut moves: Vec<((f32, f32), (f32, f32))> = Vec::new();
    let triggers = triggers
        .into_iter()
        .zip(expected)
        .map(|(trigger, &(series, index, ..))| {
            let MarkShape::Rect {
                x,
                y,
                width: w,
                height: h,
            } = trigger.shape
            else {
                return trigger;
            };
            // A lying chart's first category is at the bottom.
            let band = if spec.horizontal {
                count - 1 - index
            } else {
                index
            };
            let slot = slot_of(series);
            let from = start + unit * band as f32 + (unit - group) / 2.0;
            // Round trips through the SVG's text unchanged.
            let new: f32 = format!("{}", from + (width + gap) * slot as f32)
                .parse()
                .unwrap_or_default();
            let old = if spec.horizontal { (y, h) } else { (x, w) };
            moves.push((old, (new, width)));
            let shape = match spec.horizontal {
                true => MarkShape::Rect {
                    x,
                    y: new,
                    width: w,
                    height: width,
                },
                false => MarkShape::Rect {
                    x: new,
                    y,
                    width,
                    height: h,
                },
            };
            Trigger { shape, ..trigger }
        })
        .collect();

    let near = |a: f32, b: f32| (a - b).abs() < 0.15;
    let centre = |(at, size): (f32, f32)| at + size / 2.0;
    let shift = |middle: f32| {
        moves
            .iter()
            .find(|(old, _)| near(centre(*old), middle))
            .map(|(old, new)| centre(*new) - centre(*old))
    };
    let (along, size) = if spec.horizontal {
        ("y", "height")
    } else {
        ("x", "width")
    };
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let close = rest[open..]
            .find('>')
            .map_or(rest.len(), |close| open + close + 1);
        let tag = &rest[open..close];
        rest = &rest[close..];
        let name = tag_name(tag);
        let attributes = attributes(tag);
        let number = |name: &str| attribute(&attributes, name)?.parse::<f32>().ok();
        let replaced: Vec<(&str, f32)> = match name {
            "rect" => number(along)
                .zip(number(size))
                .and_then(|old| moves.iter().find(|(from, _)| *from == old))
                .map(|(_, (at, width))| vec![(along, *at), (size, *width)])
                .unwrap_or_default(),
            "text" => number(along)
                .and_then(|at| Some(vec![(along, at + shift(at)?)]))
                .unwrap_or_default(),
            "line" => {
                let ends = if spec.horizontal {
                    ["y1", "y2"]
                } else {
                    ["x1", "x2"]
                };
                number(ends[0])
                    .zip(number(ends[1]))
                    .and_then(|(a, b)| {
                        let by = shift((a + b) / 2.0)?;
                        Some(vec![(ends[0], a + by), (ends[1], b + by)])
                    })
                    .unwrap_or_default()
            }
            _ => Vec::new(),
        };
        if replaced.is_empty() {
            out.push_str(tag);
            continue;
        }
        let kept: String = attributes
            .iter()
            .map(
                |(attribute, value)| match replaced.iter().find(|(name, _)| name == attribute) {
                    Some((_, number)) => format!(" {attribute}=\"{number}\""),
                    None => format!(" {attribute}=\"{value}\""),
                },
            )
            .collect();
        let end = if tag.ends_with("/>") { "/>" } else { ">" };
        out.push_str(&format!("<{name}{kept}{end}"));
    }
    out.push_str(rest);
    (out, triggers)
}

/// The error ranges of the bars the renderer draws none on, lying ones and stacks: each is
/// measured from the end of its own segment, at the scale of the segment's value.
fn draw_error_bars(svg: String, marks: &[Mark], spec: &BarSpec) -> String {
    let own = spec.layout == BarLayout::Grouped && !spec.horizontal;
    if own || spec.layout == BarLayout::Percent {
        return svg;
    }
    let mut lines = String::new();
    for mark in marks {
        let range = spec.errors.get(mark.series).and_then(|e| e.get(mark.index));
        let (
            Some(Some((low, high))),
            MarkShape::Rect {
                x,
                y,
                width,
                height,
            },
        ) = (range, &mark.shape)
        else {
            continue;
        };
        let (x, y, width, height) = (*x, *y, *width, *height);
        let value = mark.value;
        if spec.series[mark.series].extras.line || value == 0.0 {
            continue;
        }
        let line = |[x1, y1, x2, y2]: [f32; 4]| {
            format!(
                "<line stroke-width=\"1.5\" x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"{}\"/>",
                spec.ink.label
            )
        };
        // Lying, the value grows to the right; upright, up the page.
        let (across, thickness, length, reach, sign) = match spec.horizontal {
            true => (y + height / 2.0, height, width, [x, x + width], 1.0),
            false => (x + width / 2.0, width, height, [y + height, y], -1.0),
        };
        let reach = reach[(value > 0.0) as usize];
        let scale = length / value.abs() as f32;
        let [from, to] = [*low, *high].map(|end| reach + sign * (end - value) as f32 * scale);
        let cap = (thickness / 4.0).clamp(2.0, 6.0);
        let [stem, a, b] = match spec.horizontal {
            true => [
                [from, across, to, across],
                [from, across - cap, from, across + cap],
                [to, across - cap, to, across + cap],
            ],
            false => [
                [across, from, across, to],
                [across - cap, from, across + cap, from],
                [across - cap, to, across + cap, to],
            ],
        };
        lines.extend([line(stem), line(a), line(b)]);
    }
    match svg.rfind("</svg>") {
        Some(end) if !lines.is_empty() => format!("{}{lines}{}", &svg[..end], &svg[end..]),
        _ => svg,
    }
}

/// The renderer writes an upright bar's label past its top, inside a falling bar. Moves a
/// falling one's under its lower end, or just inside it in the bar's ink when the x axis is
/// too near; a label past its bar is haloed in `surface`, so a line under it cannot blur it.
fn place_outside_labels(svg: &str, bottom: f32, surface: HexColor) -> String {
    const GAP: f32 = 8.0;
    let halo = format!(
        " stroke=\"{surface}\" stroke-width=\"3\" stroke-linejoin=\"round\" paint-order=\"stroke\""
    );
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let Some(close) = rest[open..].find('>').map(|close| open + close) else {
            break;
        };
        out.push_str(&rest[..open]);
        let tag = &rest[open..=close];
        let label = rest[close + 1..].trim_start();
        rest = &rest[close + 1..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let (Some(end), Some(value)) = (number("y"), label.strip_prefix(LABEL_MARK)) else {
            out.push_str(tag);
            continue;
        };
        let size = number("font-size").unwrap_or(14.0);
        let below = GAP + size * 0.8;
        let (dy, inside) = match value.starts_with('-') {
            false => (number("dy").unwrap_or(0.0), false),
            true if end + below + size * 0.25 <= bottom => (below, false),
            true => (-(GAP / 2.0 + size * 0.25), true),
        };
        let kept: String = attributes
            .iter()
            .filter(|(name, _)| *name != "dy" && !(inside && *name == "fill"))
            .map(|(name, value)| format!(" {name}=\"{value}\""))
            .collect();
        let ink = if inside {
            format!(" {ON_BAR_FILL}")
        } else {
            halo.clone()
        };
        out.push_str(&format!("<text dy=\"{dy}\"{kept}{ink}>"));
    }
    out.push_str(rest);
    out
}

/// Gives every label drawn on a bar the ink that reads on the bar nearest to it.
fn ink_bar_labels(svg: &str, marks: &[Mark], ink: impl Fn(&Mark) -> HexColor) -> String {
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
                out.push_str(&tag.replace(ON_BAR_FILL, &format!("fill=\"{}\"", ink(mark))))
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
    /// Where the last slice ends, in the same degrees; `None`, a full turn on.
    pub(crate) end_angle: Option<f32>,
    /// Every slice the same angle, its radius by its value: a nightingale rose.
    pub(crate) rose: bool,
    /// Between two slices, in pixels.
    pub(crate) slice_gap: f32,
    /// Corner radius of a slice, in pixels; 0 keeps them square.
    pub(crate) slice_radius: f32,
    /// Labels on their slices rather than beside them.
    pub(crate) labels_inside: bool,
    /// A slice narrower than this many degrees gets no label.
    pub(crate) min_label_angle: f32,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

impl PieSpec {
    /// The degrees the slices share. An end at or before the start counts a turn on.
    fn span(&self) -> f32 {
        match self.end_angle {
            Some(end) if end.is_finite() && end > self.start_angle => {
                (end - self.start_angle).min(360.0)
            }
            Some(end) if end.is_finite() => {
                let span = (end - self.start_angle).rem_euclid(360.0);
                if span > 0.0 { span } else { 360.0 }
            }
            _ => 360.0,
        }
    }
}

/// A point `radius` from `(cx, cy)` at `angle` degrees clockwise from 12 o'clock.
fn on_circle((cx, cy): (f32, f32), radius: f32, angle: f32) -> (f32, f32) {
    let (sin, cos) = angle.to_radians().sin_cos();
    (cx + radius * sin, cy - radius * cos)
}

/// The box round the part of the unit circle from `start` over `span` degrees with a hole
/// of radius `hole`, `[left, top, right, bottom]`, as the renderer centres it.
fn arc_box(start: f32, span: f32, hole: f32) -> [f32; 4] {
    let mut angles = vec![start, start + span];
    let mut cardinal = (start / 90.0).ceil() * 90.0;
    while cardinal < start + span {
        angles.push(cardinal);
        cardinal += 90.0;
    }
    let mut found = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for angle in angles {
        for radius in [hole, 1.0] {
            let (x, y) = on_circle((0.0, 0.0), radius, angle);
            found = [
                found[0].min(x),
                found[1].min(y),
                found[2].max(x),
                found[3].max(y),
            ];
        }
    }
    found
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
    chart.rose_type = Some(spec.rose);
    chart.border_radius = Some(0.0);
    chart.start_angle = spec.start_angle;
    let span = spec.span();
    chart.end_angle = (span < 360.0).then_some(spec.start_angle + span);
    chart.series.colors = spec.slices.iter().map(|s| ink(s.color)).collect();
    chart.series_label_position = spec.labels_inside.then(|| "inside".to_string());
    chart.min_show_label_angle = spec.min_label_angle.max(0.0);
    chart.series.label.font.color = match spec.labels_inside {
        true => ON_BAR,
        false => ink(spec.ink.label),
    };
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
    let hole_share = spec.donut.map_or(0.0, |ring| 1.0 - ring.clamp(0.05, 1.0));
    let ring = |radius: f32| radius * hole_share;
    let partial = span < 360.0;
    // The renderer centres a part of the circle on what is drawn of it, in the whole SVG.
    let [left, top, right, bottom] = arc_box(spec.start_angle, span, hole_share);
    let middle = |radius: f32| match partial {
        true => (
            spec.width / 2.0 - (left + right) / 2.0 * radius,
            spec.height / 2.0 - (top + bottom) / 2.0 * radius,
        ),
        false => (spec.width / 2.0, spec.height / 2.0),
    };
    // A full pie on a square canvas around the middle, its radius the renderer's share of
    // it: the pie stays centred as it shrinks for labels that would leave the SVG.
    let mut draw = |radius: f32| {
        let side = radius * 2.0 / PIE_FILL;
        chart.radius = radius;
        chart.inner_radius = ring(radius);
        chart.margin.left = if partial {
            0.0
        } else {
            (spec.width - side) / 2.0
        };
        chart.margin.right = chart.margin.left;
        chart.margin.top = if partial {
            0.0
        } else {
            (spec.height - side) / 2.0
        };
        chart.margin.bottom = chart.margin.top;
        chart.svg().ok()
    };
    let full = match partial {
        true => (spec.width * PIE_FILL / (right - left).max(0.1))
            .min(spec.height * PIE_FILL / (bottom - top).max(0.1)),
        false => spec.width.min(spec.height) * PIE_FILL / 2.0,
    };
    let mut radius = full;
    let mut svg = draw(radius)?;
    // A label moves in by at most what the radius loses: a few rounds settle it.
    for _ in 0..4 {
        let Some(over) = overflow(&svg, spec.width, spec.height) else {
            break;
        };
        let next = (radius - over.into_iter().fold(0.0, f32::max)).max(full * PIE_MIN);
        if next >= radius {
            break;
        }
        radius = next;
        svg = draw(radius)?;
    }
    let hole = ring(radius);
    let centre = middle(radius);
    let svg = match spec.labels_inside {
        true => centre_inside_labels(&svg, centre, hole),
        false => svg,
    };
    // What `center` fills: the hole, or the half of it a ring of half a turn or less bends round.
    let (inner, inner_radius) = match span <= 180.0 {
        true => (
            on_circle(centre, hole / 2.0, spec.start_angle + span / 2.0),
            hole / 2.0,
        ),
        false => (centre, hole),
    };

    // A line of the page between two slices of a close colour; a lone slice has no seam.
    let shares = spec
        .slices
        .iter()
        .filter(|s| s.values.first().copied().flatten().is_some_and(|v| v > 0.0))
        .count();
    let gap = if shares > 1 {
        spec.slice_gap.max(0.0)
    } else {
        0.0
    };
    let edge = format!(" stroke=\"{}\" stroke-width=\"{gap}\"", spec.ink.surface);
    let (svg, triggers) = inert(
        &svg,
        |label| {
            spec.format
                .format(label, spec.unit.as_deref(), spec.formats)
        },
        &edge,
        false,
    );
    let (svg, triggers) = round_slices(svg, triggers, centre, spec.slice_radius);
    let svg = relabel(&svg, PERCENT_MARK, |share| {
        ValueFormat::Percent.format(share / 100.0, None, spec.formats)
    });
    let svg = match spec.labels_inside {
        true => svg,
        false => {
            let values: Vec<f64> = spec
                .slices
                .iter()
                .map(|s| s.values.first().copied().flatten().unwrap_or(0.0))
                .collect();
            drop_colliding_labels(&cut_labels(&svg, spec.width), &values)
        }
    };
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
    let marks = match_marks(&expected, &triggers);
    let svg = ink_slice_labels(
        &svg,
        &marks,
        |slice| spec.ink.on(spec.slices[slice].color),
        spec.ink.label,
    );
    Some(Drawn {
        svg: spec.ink.paint(svg),
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot {
                left: inner.0 - inner_radius,
                top: inner.1 - inner_radius,
                right: inner.0 + inner_radius,
                bottom: inner.1 + inner_radius,
            },
            marks,
            hole,
            share_of: None,
            cells: false,
        },
    })
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FunnelSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) stages: Vec<SeriesSpec>,
    pub(crate) labels: PieLabels,
    /// A percent label is a share of this, the first stage's value.
    pub(crate) first: f64,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

pub(crate) fn draw_funnel(spec: &FunnelSpec) -> Option<Drawn> {
    let value = |stage: &SeriesSpec| {
        stage
            .values
            .first()
            .copied()
            .flatten()
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .max(0.0)
    };
    let series = spec
        .stages
        .iter()
        .map(|stage| charts_rs::Series::new(stage.name.clone(), vec![value(stage) as f32]))
        .collect();
    let mut chart = charts_rs::FunnelChart::new(series);
    chart.width = spec.width;
    chart.height = spec.height;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.margin = Default::default();
    chart.series.colors = spec.stages.iter().map(|s| ink(s.color)).collect();
    chart.series_label_position = Some("inside".to_string());
    chart.series.label.font.color = ON_BAR;
    chart.series.label.formatter = match spec.labels {
        PieLabels::Value => format!("{AXIS_MARK}{{c}}{}", unit_suffix(spec.unit.as_deref())),
        PieLabels::Percent => format!("{PERCENT_MARK}{{c}}"),
        PieLabels::Name => "{a}".to_string(),
        // Blank, as an empty one is the renderer's "{a}: {c}"; `ink_slice_labels` drops it.
        PieLabels::None => " ".to_string(),
    };
    let svg = centre_stage_labels(&chart.svg().ok()?);
    let (svg, triggers) = inert(
        &svg,
        |label| {
            spec.format
                .format(label, spec.unit.as_deref(), spec.formats)
        },
        "",
        false,
    );
    let svg = relabel(&svg, PERCENT_MARK, |value| match spec.first > 0.0 {
        true => ValueFormat::Percent.format(value / spec.first, None, spec.formats),
        false => String::new(),
    });
    // The renderer draws the widest stage first; a sort keeps ties in data order.
    let mut order: Vec<usize> = (0..spec.stages.len()).collect();
    order.sort_by(|a, b| {
        let (a, b) = (
            value(&spec.stages[*a]) as f32,
            value(&spec.stages[*b]) as f32,
        );
        b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
    });
    let expected: Vec<_> = order
        .iter()
        .map(|&i| (0, i, value(&spec.stages[i]), spec.stages[i].name.as_str()))
        .collect();
    let mut marks = match_marks(&expected, &triggers);
    let svg = ink_slice_labels(
        &svg,
        &marks,
        |stage| spec.ink.on(spec.stages[stage].color),
        spec.ink.label,
    );
    // The keys walk the author's order, as the summary and the legend read.
    marks.sort_by_key(|mark| mark.index);
    Some(Drawn {
        svg: spec.ink.paint(svg),
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot::whole(spec.width, spec.height),
            marks,
            hole: 0.0,
            share_of: Some(spec.first),
            cells: false,
        },
    })
}

/// Anchors every label on a stage at its middle, on its baseline: the renderer writes it
/// from its left end round a central baseline, measured in its own font.
fn centre_stage_labels(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let label = rest[..rest.find('<').unwrap_or(rest.len())].trim();
        let (Some(x), Some(y), true) = (number("x"), number("y"), tag.contains(ON_BAR_FILL)) else {
            out.push_str(tag);
            continue;
        };
        let size = number("font-size").unwrap_or(14.0);
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| !matches!(*attribute, "x" | "y" | "dominant-baseline"))
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\"{kept}>",
            x + text_width(&attributes, label) / 2.0,
            y + size * 0.35
        ));
    }
    out.push_str(rest);
    out
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HeatmapSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: Vec<String>,
    /// The rows' names, top first.
    pub(crate) y: Vec<String>,
    /// A row per `y`, a value per `x`; `None` is an empty cell.
    pub(crate) values: Vec<Vec<Option<f64>>>,
    /// The scale, low to high: two or more.
    pub(crate) colors: Vec<HexColor>,
    pub(crate) range: (f64, f64),
    /// Classes of the same width instead of a gradient; below 2, none.
    pub(crate) steps: usize,
    pub(crate) labels: bool,
    /// The colour scale drawn under the plot, with its values.
    pub(crate) scale: bool,
    /// The scale's key for an empty cell, when there is one.
    pub(crate) no_value: Option<String>,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

impl HeatmapSpec {
    /// A row's value at a column, if it has a finite one.
    pub(crate) fn value(&self, row: usize, column: usize) -> Option<f64> {
        self.values
            .get(row)?
            .get(column)
            .copied()
            .flatten()
            .filter(|value| value.is_finite())
    }
}

pub(crate) fn draw_heatmap(spec: &HeatmapSpec) -> Option<Drawn> {
    let (rows, columns) = (spec.y.len(), spec.x.len());
    if rows == 0 || columns == 0 {
        return None;
    }
    // The renderer's row 0 is the bottom one; ours is the top.
    let below = |row: usize| rows - 1 - row;
    let data: Vec<(usize, f32)> = (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .filter_map(|(row, column)| {
            let value = spec.value(row, column)?;
            Some((below(row) * columns + column, value as f32))
        })
        .collect();
    let y_drawn: Vec<String> = spec.y.iter().rev().cloned().collect();
    let mut chart = charts_rs::HeatmapChart::new(data, spec.x.clone(), y_drawn);
    let room = match spec.scale {
        true => SCALE_ROOM.min(spec.height / 3.0),
        false => 0.0,
    };
    chart.width = spec.width;
    chart.height = spec.height - room;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.x_axis.stroke_color = ink(spec.ink.axis);
    chart.x_axis.font.color = ink(spec.ink.label);
    chart.x_axis.label_overflow = AxisLabelOverflow::Ellipsis;
    let (low, high) = spec.range;
    let high = if high > low { high } else { low + 1.0 };
    let scale = &mut chart.heatmap_series;
    scale.min = low as f32;
    scale.max = high as f32;
    scale.colors = spec.colors.iter().map(|color| ink(*color)).collect();
    scale.steps = spec.steps;
    scale.min_font_color = (0, 0, 0).into();
    scale.max_font_color = (255, 255, 255).into();
    chart.series.label.formatter = format!("{LABEL_MARK}{{c}}");
    let svg = chart.svg().ok()?;
    let svg = fit_x_labels(&svg, &spec.x, spec.width, spec.height);
    let (svg, triggers) = inert(&svg, |value| value.to_string(), "", false);
    let svg = restyle_cells(&svg, &spec.ink);
    let expected: Vec<_> = (0..rows)
        .rev()
        .flat_map(|row| {
            (0..columns).filter_map(move |column| {
                Some((row, column, spec.value(row, column)?, spec.y[row].as_str()))
            })
        })
        .collect();
    let mut marks = match_marks(&expected, &triggers);
    let write = |value: f64| {
        spec.format
            .format(value, spec.unit.as_deref(), spec.formats)
    };
    let svg = write_cell_labels(&svg, &marks, spec.labels, write);
    marks.sort_by_key(|mark| (mark.series, mark.index));
    let plot = Plot::of(&svg, spec.width, spec.height);
    let svg = match spec.scale {
        true => with_scale(&svg, spec, plot, spec.height - room, write),
        false => svg,
    };
    Some(Drawn {
        svg: spec.ink.paint(svg),
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot,
            marks,
            hole: 0.0,
            share_of: None,
            cells: true,
        },
    })
}

/// Room under a heatmap's plot for its colour scale, the bar's height, its pieces.
const SCALE_ROOM: f32 = 40.0;
const SCALE_BAR: f32 = 10.0;
const SCALE_PIECES: usize = 32;
/// An empty cell's outline, and its key's.
const EMPTY_DASH: &str = "4,3";

/// The colour at `position` (0 to 1) of a gradient through `colors`, as the renderer mixes it.
fn gradient(colors: &[HexColor], position: f64) -> HexColor {
    let (Some(first), Some(last)) = (colors.first(), colors.last()) else {
        return HexColor::new(0);
    };
    if !position.is_finite() || position <= 0.0 || colors.len() < 2 {
        return *first;
    }
    if position >= 1.0 {
        return *last;
    }
    let place = position * (colors.len() - 1) as f64;
    let index = (place as usize).min(colors.len() - 2);
    let share = place - index as f64;
    let (from, to) = (colors[index], colors[index + 1]);
    let mix = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * share).round() as u32;
    HexColor::new(
        (mix(from.r(), to.r()) << 16) | (mix(from.g(), to.g()) << 8) | mix(from.b(), to.b()),
    )
}

/// The heatmap's SVG grown by its scale: a bar under the plot from the lowest value's colour
/// to the highest's, in classes with `steps`, its ends' values (and the classes') below it.
fn with_scale(
    svg: &str,
    spec: &HeatmapSpec,
    plot: Plot,
    top: f32,
    write: impl Fn(f64) -> String,
) -> String {
    let (low, high) = spec.range;
    let bar_top = top + 6.0;
    let label = |x: f32, y: f32, anchor: &str, text: &str| {
        format!(
            "<text x=\"{x}\" y=\"{y}\" text-anchor=\"{anchor}\" font-size=\"12\" font-family=\"inherit\" fill=\"{}\">{}</text>",
            spec.ink.label,
            escape(text)
        )
    };
    let mut scale = String::from("<g>");
    // The empty cell's key at the end: an outline as the cells', its name after it.
    let key = spec.no_value.as_ref().map_or(0.0, |name| {
        let room = SCALE_BAR + 6.0 + text_width(&[("font-size", "12")], name);
        let x = plot.right - room;
        scale.push_str(&format!(
            "<rect x=\"{x}\" y=\"{bar_top}\" width=\"{SCALE_BAR}\" height=\"{SCALE_BAR}\" fill=\"none\" stroke=\"{}\" stroke-dasharray=\"{EMPTY_DASH}\"/>",
            spec.ink.axis
        ));
        scale.push_str(&label(x + SCALE_BAR + 6.0, bar_top + SCALE_BAR, "start", name));
        room + 16.0
    });
    let (left, width) = (plot.left, (plot.right - plot.left - key).max(0.0));
    let pieces = match spec.steps >= 2 {
        true => spec.steps,
        false => SCALE_PIECES,
    };
    let piece = width / pieces as f32;
    for i in 0..pieces {
        let position = match spec.steps >= 2 {
            true => i as f64 / (pieces - 1) as f64,
            false => (i as f64 + 0.5) / pieces as f64,
        };
        // A hair wider, so no seam shows between two pieces.
        scale.push_str(&format!(
            "<rect x=\"{}\" y=\"{bar_top}\" width=\"{}\" height=\"{SCALE_BAR}\" fill=\"{}\"/>",
            left + piece * i as f32,
            piece + 0.5,
            gradient(&spec.colors, position)
        ));
    }
    let text = |x: f32, anchor: &str, value: f64| {
        label(x, bar_top + SCALE_BAR + 14.0, anchor, &write(value))
    };
    scale.push_str(&text(left, "start", low));
    scale.push_str(&text(left + width, "end", high));
    if spec.steps >= 2 {
        let fits = (1..spec.steps).all(|i| {
            let value = low + (high - low) * i as f64 / spec.steps as f64;
            text_width(&[("font-size", "12")], &write(value)) < piece - 2.0 * EDGE
        });
        for i in (1..spec.steps).filter(|_| fits) {
            let value = low + (high - low) * i as f64 / spec.steps as f64;
            scale.push_str(&text(left + piece * i as f32, "middle", value));
        }
    }
    scale.push_str("</g>");
    appended(svg, spec.width, spec.height, &scale)
}

/// `extra` drawn last, the viewBox grown to the full `width` and `height`.
fn appended(svg: &str, width: f32, height: f32, extra: &str) -> String {
    let svg = svg.trim_end().trim_end_matches("</svg>");
    let svg = match svg.find("viewBox=\"") {
        Some(at) => {
            let start = at + "viewBox=\"".len();
            let end = svg[start..].find('"').map_or(start, |end| start + end);
            format!("{}0 0 {width} {height}{}", &svg[..start], &svg[end..])
        }
        None => svg.to_string(),
    };
    format!("{svg}{extra}</svg>")
}

/// The renderer's y axis in our ink without its ticks (they fall between no rows), cells
/// parted by the surface, an empty one outlined in the grid's ink instead of a stripe.
fn restyle_cells(svg: &str, ink: &ChartInk) -> String {
    const RENDERER_AXIS: &str = "#6E7079";
    let svg = svg
        .replace(
            &format!("stroke=\"{RENDERER_AXIS}\""),
            &format!("stroke=\"{}\"", ink.axis),
        )
        .replace(
            &format!("fill=\"{RENDERER_AXIS}\""),
            &format!("fill=\"{}\"", ink.label),
        );
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg.as_str();
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        match tag_name(tag) {
            "line" => {
                let tick = matches!(
                    (number("x1"), number("x2"), number("y1"), number("y2")),
                    (Some(x1), Some(x2), Some(y1), Some(y2)) if y1 == y2 && (x2 - x1).abs() <= 5.0
                );
                if !tick {
                    out.push_str(tag);
                }
            }
            "rect" if attribute(&attributes, "data-index").is_some() => {
                let kept: String = ["x", "y", "width", "height"]
                    .iter()
                    .filter_map(|name| {
                        Some(format!(" {name}=\"{}\"", attribute(&attributes, name)?))
                    })
                    .collect();
                out.push_str(&format!(
                    "<rect{kept} fill=\"none\" stroke=\"{}\" stroke-dasharray=\"{EMPTY_DASH}\"/>",
                    ink.axis
                ));
            }
            "rect" => {
                let kept: String = attributes
                    .iter()
                    .filter(|(attribute, _)| *attribute != "stroke")
                    .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
                    .collect();
                out.push_str(&format!("<rect{kept} stroke=\"{}\"/>", ink.surface));
            }
            _ => out.push_str(tag),
        }
    }
    out.push_str(rest);
    out
}

/// A cell's label written by `write` at its middle, in the renderer's order of `marks`;
/// left out unless `show`, or when wider or taller than its cell.
fn write_cell_labels(
    svg: &str,
    marks: &[Mark],
    show: bool,
    write: impl Fn(f64) -> String,
) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    let mut cells = marks.iter();
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        let content = &rest[end..rest[end..].find('<').map_or(rest.len(), |at| end + at)];
        if !content.trim_start().starts_with(LABEL_MARK) {
            out.push_str(tag);
            rest = &rest[end..];
            continue;
        }
        rest = rest[end..]
            .find("</text>")
            .map_or("", |close| &rest[end + close + 7..]);
        let Some(Mark {
            value,
            shape:
                MarkShape::Rect {
                    x,
                    y,
                    width,
                    height,
                },
            ..
        }) = cells.next()
        else {
            continue;
        };
        let attributes = attributes(tag);
        let size = attribute(&attributes, "font-size")
            .and_then(|size| size.parse::<f32>().ok())
            .unwrap_or(14.0);
        let label = write(*value);
        if !show || text_width(&attributes, &label) > width - 2.0 * EDGE || size > *height {
            continue;
        }
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| !matches!(*attribute, "x" | "y" | "dominant-baseline"))
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\"{kept}>{}</text>",
            x + width / 2.0,
            y + height / 2.0 + size * 0.35,
            escape(&label)
        ));
    }
    out.push_str(rest);
    out
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CandleSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) x: Vec<String>,
    pub(crate) name: String,
    /// Each period's open, high, low and close; `None`, a gap.
    pub(crate) candles: Vec<Option<[f64; 4]>>,
    pub(crate) up: HexColor,
    pub(crate) down: HexColor,
    /// Ticks for the open and the close on a line, instead of bodies.
    pub(crate) ohlc: bool,
    /// The names of a rising and a falling candle, keyed under the plot.
    pub(crate) key: Option<(String, String)>,
    pub(crate) axes: Axes,
    pub(crate) grid: bool,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

/// A rising candle is drawn in this, then hollowed out: the renderer's wick takes its fill.
const UP_FILL: InkColor = InkColor {
    r: 1,
    g: 2,
    b: 4,
    a: 255,
};
const UP_HEX: &str = "#010204";

pub(crate) fn draw_candles(spec: &CandleSpec) -> Option<Drawn> {
    // The renderer reads open, close, lowest, highest.
    let data = spec
        .candles
        .iter()
        .flat_map(|candle| match candle {
            Some([open, high, low, close]) => [*open, *close, *low, *high].map(|v| Some(v as f32)),
            None => [None; 4],
        })
        .collect();
    let series = charts_rs::Series::new_nullable(spec.name.clone(), data);
    let mut chart = charts_rs::CandlestickChart::new(vec![series], spec.x.clone());
    let room = match spec.key {
        Some(_) => KEY_ROOM.min(spec.height / 3.0),
        None => 0.0,
    };
    chart.width = spec.width;
    chart.height = spec.height - room;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.candlestick_style = match spec.ohlc {
        true => charts_rs::CandlestickStyle::Ohlc,
        false => charts_rs::CandlestickStyle::Candle,
    };
    chart.candlestick_up_color = UP_FILL;
    chart.candlestick_up_border_color = ink(spec.up);
    chart.candlestick_down_color = ink(spec.down);
    chart.candlestick_down_border_color = ink(spec.down);
    chart.grid.stroke_color = match spec.grid {
        true => ink(spec.ink.grid),
        false => InkColor::transparent(),
    };
    chart.x_axis.stroke_color = ink(spec.ink.axis);
    chart.x_axis.font.color = ink(spec.ink.label);
    chart.x_axis.label_overflow = AxisLabelOverflow::Ellipsis;
    let unit = unit_suffix(spec.unit.as_deref());
    if let Some(y) = chart.y_axis_configs.first_mut() {
        y.stroke_color = ink(spec.ink.axis);
        y.font.color = ink(spec.ink.label);
        y.formatter = Some(format!("{AXIS_MARK}{{c}}{unit}"));
    }
    spec.axes
        .style(&mut chart.base, &mut chart.y_axis_configs, false);
    // Prices rarely start near 0: the axis spans the candles only.
    if let Some(left) = chart.y_axis_configs.first_mut() {
        round_axis(&chart.base.series_list, 0, left, None, false);
    }
    let write = |value: f64| {
        spec.format
            .format(value, spec.unit.as_deref(), spec.formats)
    };
    let auto = spec.format == ValueFormat::Auto;
    let (unit1, compact) = (spec.unit.as_deref(), &spec.axes.compact);
    let axis = |svg: &str| axis_write(svg, AXIS_MARK, auto, unit1, spec.formats, compact, write);
    let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
        let mut svg = chart.svg().ok()?;
        let exact = exact_axes(&svg, &chart.y_axis_configs, false);
        if let Some(width) = y_width(&svg, &exact, AXIS_MARK, axis(&exact)) {
            if let Some(y) = chart.y_axis_configs.first_mut() {
                y.width = Some(width);
            }
            svg = chart.svg().ok()?;
        }
        Some(pin_to_axis(&svg))
    })?;
    let svg = exact_axes(&svg, &chart.y_axis_configs, false);
    let svg = fit_x_labels(&svg, &spec.x, spec.width, spec.height);
    let (svg, triggers) = inert(&svg, axis(&svg), "", false);
    let expected: Vec<_> = (spec.candles.iter().enumerate())
        .filter_map(|(i, candle)| Some((0, i, candle.as_ref()?[3], spec.name.as_str())))
        .collect();
    let marks = match_marks(&expected, &triggers)
        .into_iter()
        .map(|mark| Mark {
            shape: whole_candle(&svg, &mark.shape),
            ..mark
        })
        .collect();
    let svg = hollow_rising(&svg, spec.up, spec.ink.surface, spec.ohlc);
    let plot = Plot::of(&svg, spec.width, spec.height);
    let svg = match &spec.key {
        Some(names) => with_candle_key(&svg, spec, names, plot, spec.height - room),
        None => svg,
    };
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot,
            marks,
            hole: 0.0,
            share_of: None,
            cells: false,
        },
        svg: spec.ink.paint(svg),
    })
}

/// Room under a candlestick chart's plot for its key.
const KEY_ROOM: f32 = 28.0;

/// A rising and a falling candle, or OHLC bar, each by its name, centred under the plot.
fn with_candle_key(
    svg: &str,
    spec: &CandleSpec,
    (up, down): &(String, String),
    plot: Plot,
    top: f32,
) -> String {
    const SIZE: f32 = SCALE_BAR;
    let y = top + 9.0;
    let room = |name: &str| SIZE + 6.0 + text_width(&[("font-size", "12")], name);
    let mut x = (plot.left + plot.right - room(up) - 16.0 - room(down)) / 2.0;
    let mut key = String::from("<g>");
    for (name, rising) in [(up, true), (down, false)] {
        let color = if rising { spec.up } else { spec.down };
        key.push_str(&match (spec.ohlc, rising) {
            // The open's tick left, the close's right: low to high when rising.
            (true, _) => {
                let (open, close) = if rising { (y + 8.0, y + 2.0) } else { (y + 2.0, y + 8.0) };
                let middle = x + SIZE / 2.0;
                format!(
                    "<g stroke=\"{color}\" stroke-width=\"1.5\"><line x1=\"{middle}\" y1=\"{y}\" x2=\"{middle}\" y2=\"{}\"/><line x1=\"{x}\" y1=\"{open}\" x2=\"{middle}\" y2=\"{open}\"/><line x1=\"{middle}\" y1=\"{close}\" x2=\"{}\" y2=\"{close}\"/></g>",
                    y + SIZE,
                    x + SIZE
                )
            }
            (false, true) => format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{SIZE}\" height=\"{SIZE}\" fill=\"{}\" stroke=\"{color}\"/>",
                spec.ink.surface
            ),
            (false, false) => format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{SIZE}\" height=\"{SIZE}\" fill=\"{color}\" stroke=\"{color}\"/>"
            ),
        });
        key.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"12\" font-family=\"inherit\" fill=\"{}\">{}</text>",
            x + SIZE + 6.0,
            y + SIZE,
            spec.ink.label,
            escape(name)
        ));
        x += room(name) + 16.0;
    }
    key.push_str("</g>");
    appended(svg, spec.width, spec.height, &key)
}

/// A candle's body stretched over its wick, from the high to the low, so the tooltip sits
/// above it all.
fn whole_candle(svg: &str, body: &MarkShape) -> MarkShape {
    let MarkShape::Rect {
        x,
        y,
        width,
        height,
    } = *body
    else {
        return body.clone();
    };
    let middle = x + width / 2.0;
    let (mut top, mut bottom) = (y, y + height);
    let mut rest = svg;
    while let Some(open) = rest.find("<line ") {
        let close = rest[open..]
            .find('>')
            .map_or(rest.len(), |close| open + close + 1);
        let attributes = attributes(&rest[open..close]);
        rest = &rest[close..];
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        if let (Some(x1), Some(x2), Some(y1), Some(y2)) =
            (number("x1"), number("x2"), number("y1"), number("y2"))
            && x1 == x2
            && (x1 - middle).abs() <= 0.5
        {
            top = top.min(y1.min(y2));
            bottom = bottom.max(y1.max(y2));
        }
    }
    MarkShape::Rect {
        x,
        y: top,
        width,
        height: bottom - top,
    }
}

/// Rising candles in `up` as outlines on the `surface`, a flat one as a line in `up`.
fn hollow_rising(svg: &str, up: HexColor, surface: HexColor, ohlc: bool) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<rect ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        if ohlc || attribute(&attributes, "fill") != Some(UP_HEX) {
            out.push_str(tag);
            continue;
        }
        let flat = attribute(&attributes, "height")
            .and_then(|height| height.parse::<f32>().ok())
            .is_some_and(|height| height <= 1.0);
        let kept: String = attributes
            .iter()
            .filter(|(name, _)| {
                !matches!(*name, "fill" | "stroke" | "stroke-opacity" | "fill-opacity")
            })
            .map(|(name, value)| format!(" {name}=\"{value}\""))
            .collect();
        out.push_str(&match flat {
            true => format!("<rect{kept} fill=\"{up}\"/>"),
            false => format!("<rect{kept} fill=\"{surface}\" stroke=\"{up}\"/>"),
        });
    }
    out.push_str(rest);
    out.replace(UP_HEX, &up.to_string())
}

/// Every slice's path drawn again with its corners rounded by `radius`, in the SVG and in
/// the triggers; the renderer's own rounding leaves a knob at each corner.
fn round_slices(
    svg: String,
    mut triggers: Vec<Trigger>,
    centre: (f32, f32),
    radius: f32,
) -> (String, Vec<Trigger>) {
    if radius <= 0.0 {
        return (svg, triggers);
    }
    let mut svg = svg;
    for trigger in &mut triggers {
        let MarkShape::Sector { path } = &mut trigger.shape else {
            continue;
        };
        if let Some(rounded) = rounded_sector(path, centre, radius) {
            svg = svg.replace(&format!("d=\"{path}\""), &format!("d=\"{rounded}\""));
            *path = rounded;
        }
    }
    (svg, triggers)
}

/// A sector's path with its outer corners, and its inner ones when it is a ring's, filleted
/// by `corner` pixels at most: what the thickness and the slice's angle leave of it. The
/// sector's radii and angles are read off the path; `None` for a full circle or none.
fn rounded_sector(path: &str, (cx, cy): (f32, f32), corner: f32) -> Option<String> {
    let spaced: String = path
        .chars()
        .flat_map(|c| match c.is_ascii_alphabetic() {
            true => vec![' ', c, ' '],
            false => vec![c],
        })
        .collect();
    let mut commands: Vec<(char, Vec<f32>)> = Vec::new();
    for token in spaced.split([' ', ',']).filter(|t| !t.is_empty()) {
        match token.parse::<f32>() {
            Ok(number) => commands.last_mut()?.1.push(number),
            Err(_) => commands.push((token.chars().next()?, Vec::new())),
        }
    }
    let radii: Vec<f32> = commands
        .iter()
        .filter(|(c, args)| *c == 'A' && args.first().is_some_and(|r| *r > 0.5))
        .map(|(_, args)| args[0])
        .collect();
    let outer = radii.iter().copied().fold(0.0, f32::max);
    let inner = radii
        .iter()
        .copied()
        .filter(|r| outer - r > 0.5)
        .fold(0.0, f32::max);
    let rim: Vec<(f32, f32)> = commands
        .iter()
        .filter_map(|(c, args)| match (c, args.len()) {
            ('M' | 'L', 2) => Some((args[0], args[1])),
            ('A', 7) => Some((args[5], args[6])),
            _ => None,
        })
        .filter(|(x, y)| ((x - cx).hypot(y - cy) - outer).abs() < 0.6)
        .collect();
    let angle = |(x, y): (f32, f32)| (x - cx).atan2(cy - y).to_degrees().rem_euclid(360.0);
    let start = angle(*rim.first()?);
    let mut span = angle(*rim.last()?) - start;
    if span <= 0.0 {
        span += 360.0;
    }
    if outer <= 0.0 || !(0.5..359.5).contains(&span) {
        return None;
    }
    // A fillet of radius `b` inside the outer arc sits `outer - b` from the centre, `b` off
    // the edge; round the ring's inside just so, outside the inner arc.
    let room = (span / 2.0 * 0.9).to_radians().sin();
    let mut b = corner
        .min((outer - inner) / 2.0)
        .min(outer * room / (1.0 + room));
    if inner > 0.0 {
        b = b.min(inner * room / (1.0 - room));
    }
    if b <= 0.1 {
        return None;
    }
    let b = (b * 100.0).floor() / 100.0;
    let at = |radius: f32, degrees: f32| {
        let (sin, cos) = degrees.to_radians().sin_cos();
        (cx + radius * sin, cy - radius * cos)
    };
    let point = |(x, y): (f32, f32)| {
        format!(
            "{},{}",
            (x * 100.0).round() / 100.0,
            (y * 100.0).round() / 100.0
        )
    };
    let end = start + span;
    // Where a fillet touches the edge, and how far round the arc it touches.
    let touch = |centre_radius: f32| {
        let turn = (b / centre_radius).asin().to_degrees();
        ((centre_radius.powi(2) - b * b).sqrt(), turn)
    };
    let (edge_o, turn_o) = touch(outer - b);
    let large = |turn: f32| u8::from(span - 2.0 * turn > 180.0);
    let d = match inner > 0.0 {
        true => {
            let (edge_i, turn_i) = touch(inner + b);
            format!(
                "M{} L{} A{b},{b} 0 0 1 {} A{outer},{outer} 0 {} 1 {} A{b},{b} 0 0 1 {} L{} A{b},{b} 0 0 1 {} A{inner},{inner} 0 {} 0 {} A{b},{b} 0 0 1 {} Z",
                point(at(edge_i, start)),
                point(at(edge_o, start)),
                point(at(outer, start + turn_o)),
                large(turn_o),
                point(at(outer, end - turn_o)),
                point(at(edge_o, end)),
                point(at(edge_i, end)),
                point(at(inner, end - turn_i)),
                large(turn_i),
                point(at(inner, start + turn_i)),
                point(at(edge_i, start)),
            )
        }
        false => format!(
            "M{} L{} A{b},{b} 0 0 1 {} A{outer},{outer} 0 {} 1 {} A{b},{b} 0 0 1 {} L{} Z",
            point((cx, cy)),
            point(at(edge_o, start)),
            point(at(outer, start + turn_o)),
            large(turn_o),
            point(at(outer, end - turn_o)),
            point(at(edge_o, end)),
            point((cx, cy)),
        ),
    };
    Some(d)
}

/// Moves every label on a slice to the middle of its slice's band, between the hole and the
/// slice's own radius. The renderer puts it two thirds of that radius out from `centre`,
/// in a donut's hole or by a rose's small slice's centre; anchored at its middle, it stays
/// centred once written again.
fn centre_inside_labels(svg: &str, centre: (f32, f32), hole: f32) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let label = rest[..rest.find('<').unwrap_or(rest.len())].trim();
        let on_slice = tag.contains(ON_BAR_FILL) && attribute(&attributes, "text-anchor").is_none();
        let (Some(x), Some(y), true) = (number("x"), number("y"), on_slice) else {
            out.push_str(tag);
            continue;
        };
        let (dx, dy) = (
            x + text_width(&attributes, label) / 2.0 - centre.0,
            y - centre.1,
        );
        let distance = dx.hypot(dy);
        if distance < 0.5 {
            out.push_str(tag);
            continue;
        }
        let outer = distance * 1.5;
        let scale = (hole + outer) / 2.0 / distance;
        let size = number("font-size").unwrap_or(14.0);
        let (x, y) = (centre.0 + dx * scale, centre.1 + dy * scale + size * 0.35);
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| !matches!(*attribute, "x" | "y"))
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\"{kept}>"
        ));
    }
    out.push_str(rest);
    out
}

/// Cuts a label that leaves the SVG at a side short with an ellipsis, inside [`EDGE`]; the
/// tooltip and the summary keep the whole text.
/// Drops the outside labels that overlap a larger slice's, with their leader lines: the
/// smaller slice's goes first. `values` are the slices' in drawing order.
fn drop_colliding_labels(svg: &str, values: &[f64]) -> String {
    // Per label: its slice's value, its box, and the byte ranges of its leader and its text.
    let mut labels: Vec<(f64, [f32; 4], Vec<std::ops::Range<usize>>)> = Vec::new();
    let (mut slice, mut leader) = (None::<usize>, None);
    let mut at = 0;
    while let Some(found) = svg[at..].find('<') {
        let start = at + found;
        let end = svg[start..]
            .find('>')
            .map_or(svg.len(), |end| start + end + 1);
        let tag = &svg[start..end];
        at = end;
        if tag.starts_with("<path ") {
            match tag.contains("fill=\"none\"") {
                true => leader = Some(start..end),
                false => {
                    slice = Some(slice.map_or(0, |i| i + 1));
                    leader = None;
                }
            }
        } else if tag.starts_with("<text ") {
            let close = svg[end..]
                .find("</text>")
                .map_or(svg.len(), |close| end + close + 7);
            at = close;
            let attributes = attributes(tag);
            let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
            let (Some(x), Some(y), Some(index)) = (number("x"), number("y"), slice) else {
                continue;
            };
            let label = unescape(svg[end..close].trim_end_matches("</text>").trim());
            let width = text_width(&attributes, &label);
            let left = match attribute(&attributes, "text-anchor") {
                Some("middle") => x - width / 2.0,
                Some("end") => x - width,
                _ => x,
            };
            let size = number("font-size").unwrap_or(14.0);
            let ranges = leader
                .take()
                .into_iter()
                .chain(std::iter::once(start..close))
                .collect();
            let value = values.get(index).copied().unwrap_or(0.0);
            labels.push((
                value,
                [left, y - size * 0.75, left + width, y + size * 0.25],
                ranges,
            ));
        }
    }
    let mut order: Vec<usize> = (0..labels.len()).collect();
    order.sort_by(|a, b| labels[*b].0.total_cmp(&labels[*a].0));
    let mut kept: Vec<[f32; 4]> = Vec::new();
    let mut dropped: Vec<std::ops::Range<usize>> = Vec::new();
    for i in order {
        let [left, top, right, bottom] = labels[i].1;
        let overlaps = kept
            .iter()
            .any(|[l, t, r, b]| left < *r && *l < right && top < *b && *t < bottom);
        match overlaps {
            true => dropped.extend(labels[i].2.iter().cloned()),
            false => kept.push(labels[i].1),
        }
    }
    dropped.sort_by_key(|range| range.start);
    let mut out = String::with_capacity(svg.len());
    let mut from = 0;
    for range in dropped {
        out.push_str(&svg[from..range.start]);
        from = range.end;
    }
    out.push_str(&svg[from..]);
    out
}

fn cut_labels(svg: &str, width: f32) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        let attributes = attributes(tag);
        let text_end = rest.find('<').unwrap_or(rest.len());
        let label = unescape(rest[..text_end].trim());
        let anchor = attribute(&attributes, "text-anchor");
        let Some(x) = attribute(&attributes, "x").and_then(|x| x.parse::<f32>().ok()) else {
            out.push_str(tag);
            continue;
        };
        let drawn = text_width(&attributes, &label);
        let left = match anchor {
            Some("middle") => x - drawn / 2.0,
            Some("end") => x - drawn,
            _ => x,
        };
        let (from, to) = (left.max(EDGE), (left + drawn).min(width - EDGE));
        let tip = attribute(&attributes, "class") == Some("ct-tip");
        if tip || label.is_empty() || (from <= left && to >= left + drawn) {
            out.push_str(tag);
            continue;
        }
        let cut = shorten(&attributes, &label, to - from);
        let x = match anchor {
            Some("middle") => (from + to) / 2.0,
            Some("end") => to,
            _ => from,
        };
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| *attribute != "x")
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!("<text x=\"{x}\"{kept}>\n{}\n", escape(&cut)));
        rest = &rest[text_end..];
    }
    out.push_str(rest);
    out
}

/// `label` cut short with an ellipsis to at most `room` wide in the tag's font, or as is
/// where it fits.
fn shorten(attributes: &[(&str, &str)], label: &str, room: f32) -> String {
    if text_width(attributes, label) <= room {
        return label.to_string();
    }
    let mut chars: Vec<char> = label.chars().collect();
    let mut cut = String::from("…");
    while chars.pop().is_some() {
        cut = chars.iter().collect::<String>().trim_end().to_string() + "…";
        if text_width(attributes, &cut) <= room || chars.is_empty() {
            break;
        }
    }
    cut
}

/// Fits the category `labels` under the plot each into its band, cut with an ellipsis; where
/// a band is too narrow for three letters and the ellipsis, only every n-th label is written.
fn fit_x_labels(svg: &str, labels: &[String], width: f32, height: f32) -> String {
    if labels.is_empty() {
        return svg.to_string();
    }
    // Each text's tag start, its element's end, its x and y and its content.
    let mut texts = Vec::new();
    let mut at = 0;
    while let Some(open) = svg[at..].find("<text ").map(|open| at + open) {
        let Some(close) = svg[open..].find('>').map(|close| open + close) else {
            break;
        };
        let end = svg[close..]
            .find("</text>")
            .map_or(close + 1, |end| close + end + 7);
        let attributes = attributes(&svg[open..=close]);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let content = unescape(svg[close + 1..end].trim_end_matches("</text>").trim());
        if let (Some(x), Some(y)) = (number("x"), number("y")) {
            texts.push((open, end, x, y, content));
        }
        at = close + 1;
    }
    // The renderer writes each one whole or cut short with its own ellipsis.
    let drawn = |content: &str, label: &str| {
        content == label
            || content
                .strip_suffix('…')
                .is_some_and(|start| label.starts_with(start.trim_end()))
    };
    let Some(first) = (0..texts.len().saturating_sub(labels.len() - 1)).find(|&start| {
        let row = &texts[start..start + labels.len()];
        row.iter()
            .zip(labels)
            .all(|(text, label)| drawn(&text.4, label))
            && row.iter().all(|text| text.3 == row[0].3)
    }) else {
        return svg.to_string();
    };
    let row = &texts[first..first + labels.len()];
    let plot = Plot::of(svg, width, height);
    let band = (plot.right - plot.left) / labels.len() as f32;
    let tag = |open: usize| attributes(&svg[open..=open + svg[open..].find('>').unwrap_or(0)]);
    // Each label's first three letters and the ellipsis, or the whole of a shorter one.
    let least = labels.iter().fold(0.0_f32, |most, label| {
        let start: String = label.chars().take(3).collect();
        let shown = match start.len() < label.len() {
            true => start + "…",
            false => start,
        };
        most.max(text_width(&tag(row[0].0), &shown))
    });
    let fits = |least: f32| {
        (1..=labels.len())
            .find(|n| *n as f32 * band - EDGE >= least)
            .unwrap_or(labels.len())
    };
    // Thinned anyway, they are written whole while at least three are left.
    let widest = labels.iter().fold(0.0_f32, |most, label| {
        most.max(text_width(&tag(row[0].0), label))
    });
    let every = match (fits(least), fits(widest)) {
        (1, _) => 1,
        (some, whole) if labels.len().div_ceil(whole) >= 3 => whole.max(some),
        (some, _) => some,
    };
    let mut out = String::with_capacity(svg.len());
    let mut last = 0;
    // Where the label before ends.
    let mut written = f32::MIN;
    for (index, ((open, end, _, _, _), label)) in row.iter().zip(labels).enumerate() {
        out.push_str(&svg[last..*open]);
        last = *end;
        if index % every != 0 {
            continue;
        }
        let attributes = tag(*open);
        let band_centre = plot.left + band * (index as f32 + 0.5);
        let mut cut = shorten(&attributes, label, every as f32 * band - EDGE);
        // Under its band, moved in from a side of the SVG, else cut to stay centred there.
        let half = text_width(&attributes, &cut) / 2.0;
        let mut centre = band_centre.clamp(EDGE + half, (width - EDGE - half).max(EDGE + half));
        if centre - half < written + EDGE {
            let side = (band_centre - EDGE).min(width - EDGE - band_centre);
            cut = shorten(&attributes, label, 2.0 * side);
            centre = band_centre;
        }
        written = centre + text_width(&attributes, &cut) / 2.0;
        let kept: String = attributes
            .iter()
            .filter(|(attribute, _)| !matches!(*attribute, "x" | "text-anchor"))
            .map(|(attribute, value)| format!(" {attribute}=\"{value}\""))
            .collect();
        out.push_str(&format!(
            "<text x=\"{centre}\" text-anchor=\"middle\"{kept}>\n{}\n</text>",
            escape(&cut)
        ));
    }
    out.push_str(&svg[last..]);
    out
}

/// Gives every label drawn on a slice the ink that reads on that slice; one off every
/// slice takes `off`.
fn ink_slice_labels(
    svg: &str,
    marks: &[Mark],
    ink: impl Fn(usize) -> HexColor,
    off: HexColor,
) -> String {
    let outlines: Vec<(usize, _)> = marks
        .iter()
        .filter_map(|mark| match &mark.shape {
            MarkShape::Sector { path } => Some((mark.index, super::hit::outline(path))),
            _ => None,
        })
        .collect();
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find('>')
            .map_or(rest.len(), |end| start + end + 1);
        let tag = &rest[start..end];
        rest = &rest[end..];
        if !tag.contains(ON_BAR_FILL) {
            out.push_str(tag);
            continue;
        }
        let attributes = attributes(tag);
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let label = &rest[..rest.find('<').unwrap_or(rest.len())];
        if label.trim().is_empty() {
            rest = rest.find("</text>").map_or("", |close| &rest[close + 7..]);
            continue;
        }
        // The renderer writes a label on a slice from its left end; `centre_inside_labels`
        // anchors it at its middle.
        let middle = number("x").zip(number("y")).map(|(x, y)| {
            let size = number("font-size").unwrap_or(14.0);
            let x = match attribute(&attributes, "text-anchor") {
                Some("middle") => x,
                _ => x + text_width(&attributes, label.trim()) / 2.0,
            };
            (x, y - size * 0.35)
        });
        let on = middle.and_then(|(x, y)| {
            outlines
                .iter()
                .find(|(_, outline)| super::hit::contains(outline, x, y))
        });
        // A label wider or taller than its slice is dropped; the summary still has it.
        let half = text_width(&attributes, label.trim()) / 2.0;
        let rise = number("font-size").unwrap_or(14.0) * 0.35;
        // Glyphs leave a box's corners empty: those are checked a little in.
        let inset = half * 0.8;
        if let (Some((x, y)), Some((_, outline))) = (middle, on)
            && ![
                (-half, 0.0),
                (half, 0.0),
                (-inset, -rise),
                (inset, -rise),
                (inset, rise),
                (-inset, rise),
            ]
            .iter()
            .all(|(dx, dy)| super::hit::contains(outline, x + dx, y + dy))
        {
            rest = rest.find("</text>").map_or("", |close| &rest[close + 7..]);
            continue;
        }
        let hex = on.map_or(off, |(slice, _)| ink(*slice));
        out.push_str(&tag.replace(ON_BAR_FILL, &format!("fill=\"{hex}\"")));
    }
    out.push_str(rest);
    out
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
    inert(svg, axis, "", false)
}

/// [`sanitize`], with `extra` attributes on every kept mark; with `circles`, a circle trigger
/// is a scatter's point, kept, not a line's invisible target.
fn inert(
    svg: &str,
    axis: impl Fn(f64) -> String,
    extra: &str,
    circles: bool,
) -> (String, Vec<Trigger>) {
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
            if name != "circle" || circles {
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
        // A funnel's stage or a scatter's triangle or diamond, as the path of its corners.
        "polygon" => {
            let corners: Vec<&str> = attribute(attributes, "points")?
                .split_whitespace()
                .collect();
            let first = corners.first()?.replace(',', " ");
            let rest: String = corners[1..]
                .iter()
                .map(|corner| format!(" L {}", corner.replace(',', " ")))
                .collect();
            MarkShape::Sector {
                path: format!("M {first}{rest} Z"),
            }
        }
        _ => return None,
    };
    // A heatmap's cell names its row instead.
    let series = attribute(attributes, "data-series").or(attribute(attributes, "data-y"))?;
    Some(Trigger {
        series: unescape(series),
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

/// How an axis writes its ticks: under `ValueFormat::Auto` (`auto`) every tick at the one
/// precision its step needs and, past seven digits, in the `compact` million, billion or
/// trillion; otherwise as `plain` writes them.
fn axis_write<'a, F: Fn(f64) -> String + 'a>(
    svg: &str,
    mark: char,
    auto: bool,
    unit: Option<&'a str>,
    formats: &'a Formats,
    compact: &'a [&'static str; 3],
    plain: F,
) -> impl Fn(f64) -> String + use<'a, F> {
    let ticks = auto.then(|| Ticks::of(svg, mark, compact));
    move |value| match ticks {
        Some(ticks) => ticks.write(value, unit, formats),
        None => plain(value),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ticks {
    decimals: usize,
    scale: f64,
    suffix: &'static str,
}

impl Ticks {
    fn of(svg: &str, mark: char, compact: &[&'static str; 3]) -> Self {
        let mut values = Vec::new();
        let mut rest = svg;
        while let Some(at) = rest.find(mark) {
            let label = &rest[at + mark.len_utf8()..];
            let end = label.find(['\n', '<', ' ']).unwrap_or(label.len());
            values.extend(scaled(label[..end].trim_end_matches('%')));
            rest = &label[end..];
        }
        let largest = values.iter().fold(0.0_f64, |most, v| most.max(v.abs()));
        let (scale, suffix) = match largest {
            l if l >= 1e12 => (1e12, compact[2]),
            l if l >= 1e9 => (1e9, compact[1]),
            l if l >= 1e7 => (1e6, compact[0]),
            _ => (1.0, ""),
        };
        let (low, high) = values.iter().fold((f64::MAX, f64::MIN), |(low, high), v| {
            (low.min(*v), high.max(*v))
        });
        let step = (high - low) / scale / values.len().saturating_sub(1).max(1) as f64;
        let decimals = match step > 0.0 && step.is_finite() {
            // The step's first digit, and one more for a step like 2.5 or 0.25.
            true => (0..=15)
                .find(|d| {
                    let shifted = step * 10_f64.powi(*d);
                    shifted >= 1.0 && (shifted - shifted.round()).abs() < shifted * 1e-4
                })
                .unwrap_or(15) as usize,
            false => 0,
        };
        Self {
            decimals,
            scale,
            suffix,
        }
    }

    fn write(&self, value: f64, unit: Option<&str>, formats: &Formats) -> String {
        let shown = format!("{:.*}", self.decimals, value / self.scale);
        // f32 noise round a zero tick must not read "-0".
        let shown = match shown
            .trim_start_matches('-')
            .chars()
            .all(|c| matches!(c, '0' | '.'))
        {
            true => shown.trim_start_matches('-').to_string(),
            false => shown,
        };
        // 0 needs no million.
        let suffix = match value == 0.0 {
            true => "",
            false => self.suffix,
        };
        let number = format!("{}{suffix}", formats.write_number(&shown));
        match unit.filter(|unit| !unit.is_empty()) {
            Some(unit) => format!("{number} {unit}"),
            None => number,
        }
    }
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
    pin_labels(svg, AXIS_MARK, Pin::<fn(f64) -> String>::End)
}

/// A lying bar chart's value labels, or a numeric x axis' behind `mark`, sit under their
/// ticks: pinned by their centre, as `axis` writes them, and kept inside the `width`.
fn pin_to_ticks(svg: &str, mark: char, width: f32, axis: impl Fn(f64) -> String) -> String {
    pin_labels(svg, mark, Pin::Middle { width, axis })
}

enum Pin<F> {
    End,
    Middle { width: f32, axis: F },
}

fn pin_labels<F: Fn(f64) -> String>(svg: &str, mark: char, pin: Pin<F>) -> String {
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
            Some(x) if label.starts_with(mark) => {
                let drawn = text_width(&attributes, label);
                let (x, anchor) = match &pin {
                    Pin::End => (x + drawn, "end"),
                    Pin::Middle { width, axis } => {
                        let number = label[mark.len_utf8()..].split(' ').next();
                        let half = number
                            .and_then(scaled)
                            .map_or(drawn, |value| text_width(&attributes, &axis(value)))
                            / 2.0;
                        let centre = x + drawn / 2.0;
                        (centre.min(width - EDGE - half).max(EDGE + half), "middle")
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

/// The most `width` gives a label behind `mark`, handed its tag's attributes, its text and
/// its number.
fn widest(
    svg: &str,
    mark: char,
    width: impl Fn(&[(&str, &str)], &str, &str) -> f32,
) -> Option<f32> {
    let mut most = 0.0f32;
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let close = open + rest[open..].find('>')?;
        let attributes = attributes(&rest[open..=close]);
        let label = &rest[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        if let Some(number) = label.strip_prefix(mark) {
            let number = number.split(' ').next().unwrap_or_default();
            most = most.max(width(&attributes, label, number));
        }
        rest = &rest[close + 1..];
    }
    Some(most)
}

/// The y axis' width once `axis` writes its labels, read in full from `exact`, when that is
/// more than the renderer kept for them in `svg`.
fn y_width(svg: &str, exact: &str, mark: char, axis: impl Fn(f64) -> String) -> Option<f32> {
    let kept = widest(svg, mark, |attributes, label, _| {
        text_width(attributes, label)
    })?;
    let needed = widest(exact, mark, |attributes, _, number| {
        scaled(number).map_or(0.0, |value| text_width(attributes, &axis(value)))
    })?;
    // The renderer's own room: the label's width and 5.
    (needed > kept).then_some(needed + 5.0)
}

/// A label's width in the renderer's own measure, in the tag's font.
fn text_width(attributes: &[(&str, &str)], text: &str) -> f32 {
    // `sanitize` hands the renderer's font to the page as `inherit`.
    let family = attribute(attributes, "font-family")
        .filter(|family| *family != "inherit")
        .unwrap_or(charts_rs::DEFAULT_FONT_FAMILY);
    let size = attribute(attributes, "font-size")
        .and_then(|size| size.parse().ok())
        .unwrap_or(14.0);
    charts_rs::measure_text_width_family(family, size, text).map_or(0.0, |text| text.width())
}

/// Draws `chart`, then once more with a wider margin if a label left the SVG.
fn fitted<C: std::ops::DerefMut<Target = charts_rs::ChartBase>>(
    chart: &mut C,
    width: f32,
    height: f32,
    draw: impl Fn(&mut C) -> Option<String>,
) -> Option<String> {
    let svg = draw(chart)?;
    let Some([left, top, right, bottom]) = overflow(&svg, width, height) else {
        return Some(svg);
    };
    chart.margin.left += left;
    chart.margin.top += top;
    chart.margin.right += right;
    chart.margin.bottom += bottom;
    draw(chart)
}

/// How far the texts reach past [`EDGE`] on each side, `[left, top, right, bottom]`;
/// `None` when they all keep inside.
fn overflow(svg: &str, width: f32, height: f32) -> Option<[f32; 4]> {
    let mut over = [0.0f32; 4];
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let Some(close) = rest[open..].find('>').map(|close| open + close) else {
            break;
        };
        let attributes = attributes(&rest[open..=close]);
        let label = &rest[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        rest = &rest[close + 1..];
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let (Some(x), Some(y)) = (number("x"), number("y")) else {
            continue;
        };
        if label.is_empty() || attribute(&attributes, "class") == Some("ct-tip") {
            continue;
        }
        let (x, y) = (
            x + number("dx").unwrap_or(0.0),
            y + number("dy").unwrap_or(0.0),
        );
        let size = number("font-size").unwrap_or(14.0);
        let drawn = text_width(&attributes, label);
        let left = match attribute(&attributes, "text-anchor") {
            Some("middle") => x - drawn / 2.0,
            Some("end") => x - drawn,
            _ => x,
        };
        // `y` is the baseline: the ascent above it, the descent below.
        let reach = [
            EDGE - left,
            EDGE - (y - size * 0.8),
            left + drawn - (width - EDGE),
            y + size * 0.25 - (height - EDGE),
        ];
        for (over, reach) in over.iter_mut().zip(reach) {
            *over = over.max(reach);
        }
    }
    over.iter().any(|over| *over > 0.0).then_some(over)
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

    fn scatter(points: &[(f64, f64, Option<f64>)], symbol: PointSymbol) -> ScatterSpec {
        let series = |name: &str, symbol| ScatterSpecSeries {
            name: name.to_string(),
            points: points
                .iter()
                .enumerate()
                .map(|(i, &(x, y, size))| (i, x, y, size))
                .collect(),
            color: chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6),
            symbol,
        };
        ScatterSpec {
            width: 320.0,
            height: 180.0,
            series: vec![series("A", PointSymbol::Circle), series("B", symbol)],
            bubble: points.iter().any(|p| p.2.is_some()),
            trend: None,
            axes: axes(),
            x_min: None,
            x_max: None,
            grid: true,
            radius: 5.0,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: None,
            formats: &Formats::GERMAN,
        }
    }

    /// Every point is a mark at its place, kept visible, its shape's centre; the axes fit the
    /// data, not 0, and are written in the locale.
    #[test]
    fn scatter_points_are_marks_on_fitted_axes() {
        let points = [
            (150.0, 42.5, None),
            (160.0, 51.0, None),
            (175.5, 70.0, None),
        ];
        for symbol in [
            PointSymbol::Triangle,
            PointSymbol::Square,
            PointSymbol::Diamond,
        ] {
            let drawn = draw_scatter(&scatter(&points, symbol)).expect("drawn");
            let marks = &drawn.geometry.marks;
            assert_eq!(marks.len(), 6, "{symbol:?}: {}", drawn.svg);
            let at = |m: &Mark| match m.shape {
                MarkShape::Point { x, y } => (x, y),
                _ => panic!("{m:?}"),
            };
            // Both series lie on each other: B's centres are A's.
            for i in 0..3 {
                let (a, b) = (at(&marks[i]), at(&marks[i + 3]));
                assert!(
                    (a.0 - b.0).abs() < 0.6 && (a.1 - b.1).abs() < 0.6,
                    "{a:?} {b:?}"
                );
                assert_eq!((marks[i].series, marks[i].index), (0, i));
            }
            assert!(at(&marks[0]).0 < at(&marks[1]).0 && at(&marks[1]).1 < at(&marks[0]).1);
            assert_eq!(marks[2].value, 70.0);
            assert!(drawn.svg.contains("<circle"), "{}", drawn.svg);
            assert!(!drawn.svg.contains("ct-"), "{}", drawn.svg);
            // Not from 0: the x axis starts near the lowest x.
            assert!(!drawn.svg.contains(">\n0\n<"), "{}", drawn.svg);
            assert!(drawn.svg.contains("150"), "{}", drawn.svg);
        }
    }

    /// Bubbles are drawn largest first; the marks keep each point's place in its series.
    #[test]
    fn bubbles_keep_their_places() {
        let points = [
            (1.0, 1.0, Some(10.0)),
            (2.0, 2.0, Some(90.0)),
            (3.0, 3.0, Some(40.0)),
        ];
        let mut spec = scatter(&points, PointSymbol::Circle);
        spec.trend = Some(Trend::Linear);
        let drawn = draw_scatter(&spec).expect("drawn");
        let order: Vec<_> = drawn
            .geometry
            .marks
            .iter()
            .map(|m| (m.series, m.index))
            .collect();
        assert_eq!(order, [(0, 1), (0, 2), (0, 0), (1, 1), (1, 2), (1, 0)]);
        // The trend line is drawn, its formula not.
        assert!(!drawn.svg.contains("y ="), "{}", drawn.svg);
        // Both trend lines over every bubble, the bubbles translucent.
        let last_bubble = drawn.svg.rfind("<circle").expect("bubbles");
        assert_eq!(
            drawn.svg[last_bubble..].matches("<path").count(),
            2,
            "{}",
            drawn.svg
        );
        assert!(drawn.svg.contains("fill-opacity"), "{}", drawn.svg);
    }

    fn line(series: Vec<SeriesSpec>, x: &[&str]) -> LineSpec {
        LineSpec {
            width: 320.0,
            height: 180.0,
            x: x.iter().map(|x| x.to_string()).collect(),
            x_numbers: Vec::new(),
            series,
            errors: Vec::new(),
            curve: Curve::Linear,
            area: None,
            stacked: false,
            axes: axes(),
            grid: true,
            points: true,
            stroke_width: 2.0,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: Some("kg".to_string()),
            y2_unit: None,
            formats: &Formats::GERMAN,
        }
    }

    fn axes() -> Axes {
        Axes {
            ticks: Theme::DEFAULT.chart.y_ticks,
            ..Axes::default()
        }
    }

    /// The renderer's own step count is the theme's, so an unset `y_ticks` changes nothing.
    #[test]
    fn the_theme_s_ticks_are_the_renderer_s() {
        let chart = charts_rs::LineChart::new(Vec::new(), Vec::new());
        assert_eq!(
            chart.y_axis_configs[0].split_number,
            Theme::DEFAULT.chart.y_ticks
        );
    }

    /// Fixed ends and a step count give exactly those labels; titles sit by their axes.
    #[test]
    fn a_fixed_axis_range_and_titles_are_drawn() {
        let mut spec = line(vec![spec("A", &[Some(12.0), Some(31.0)])], &["Q1", "Q2"]);
        spec.axes = Axes {
            min: Some(10.0),
            max: Some(50.0),
            ticks: 4,
            x_title: Some("Quarter".to_string()),
            y_title: Some("Weight".to_string()),
            ..Axes::default()
        };
        let svg = draw_line(&spec).expect("drawn").svg;
        for label in [
            "10 kg",
            "20 kg",
            "30 kg",
            "40 kg",
            "50 kg",
            "\nQuarter\n",
            "\nWeight\n",
        ] {
            assert!(svg.contains(label), "{label}: {svg}");
        }
        assert!(!svg.contains("60 kg") && !svg.contains("\n0 kg"), "{svg}");
        assert!(svg.contains("rotate(-90)"), "{svg}");
    }

    /// Unset ends land on a round step, as many as `ticks` asks or one more or fewer.
    #[test]
    fn an_unset_axis_end_lands_on_a_round_step() {
        let labels = |values: &[f64], axes: Axes| {
            let values: Vec<_> = values.iter().map(|v| Some(*v)).collect();
            let mut spec = line(vec![spec("A", &values)], &["Q1", "Q2"]);
            spec.axes = axes;
            let svg = draw_line(&spec).expect("drawn").svg;
            let ticks = texts(&svg).into_iter().map(|text| text.3);
            ticks
                .filter(|label| label.ends_with(" kg"))
                .collect::<Vec<_>>()
        };
        let kg = |values: &[i32]| values.iter().map(|v| format!("{v} kg")).collect::<Vec<_>>();
        assert_eq!(
            labels(&[12.0, 31.0], axes()),
            kg(&[35, 30, 25, 20, 15, 10, 5, 0])
        );
        let four = Axes { ticks: 4, ..axes() };
        assert_eq!(
            labels(&[3.0, 48.0], four.clone()),
            kg(&[50, 40, 30, 20, 10, 0])
        );
        assert_eq!(labels(&[-7.0, 18.0], four), kg(&[20, 10, 0, -10]));
        assert_eq!(labels(&[0.0, 0.0], axes())[0], "1,0 kg");
        let from = Axes {
            min: Some(10.0),
            ..axes()
        };
        assert_eq!(labels(&[12.0, 31.0], from), kg(&[35, 30, 25, 20, 15, 10]));
    }

    /// Ticks past seven digits are written in the locale's millions; the rest keep theirs.
    #[test]
    fn a_long_axis_writes_its_ticks_in_millions() {
        let compact = crate::localization::ChartLabels::GERMAN.compact;
        let mut short = line(vec![spec("A", &[Some(1.0), Some(4.0)])], &["Q1", "Q2"]);
        short.axes.compact = compact;
        let mut long = line(
            vec![spec("A", &[Some(12_345_678.0), Some(48_000_000.0)])],
            &["Q1", "Q2"],
        );
        long.axes.compact = compact;
        let svg = draw_line(&long).expect("drawn").svg;
        let ticks: Vec<String> = texts(&svg)
            .into_iter()
            .map(|text| text.3)
            .filter(|label| label.contains("Mio."))
            .collect();
        assert!(ticks.len() >= 3, "{svg}");
        assert!(ticks.contains(&"40\u{a0}Mio. kg".to_string()), "{ticks:?}");
        assert!(texts(&svg).iter().any(|text| text.3 == "0 kg"), "{svg}");
        let digits = |label: &str| label.chars().filter(char::is_ascii_digit).count();
        assert!(texts(&svg).iter().all(|text| digits(&text.3) <= 7), "{svg}");

        let svg = draw_line(&short).expect("drawn").svg;
        assert!(!svg.contains("Mio."), "{svg}");
    }

    /// Every tick of a tiny axis has the decimals its step needs, so none reads 0 but zero.
    #[test]
    fn a_tiny_axis_writes_every_tick_at_one_precision() {
        let mut spec = line(vec![spec("A", &[Some(0.001), Some(0.004)])], &["Q1", "Q2"]);
        spec.unit = None;
        let svg = draw_line(&spec).expect("drawn").svg;
        let ticks: Vec<String> = texts(&svg)
            .into_iter()
            .map(|text| text.3)
            .filter(|label| label.starts_with("0,"))
            .collect();
        assert!(ticks.len() >= 3, "{svg}");
        let decimals = |label: &String| label.len() - 2;
        assert!(
            ticks.iter().all(|t| decimals(t) == decimals(&ticks[0])),
            "{ticks:?}"
        );
        let mut distinct = ticks.clone();
        distinct.dedup();
        assert_eq!(distinct, ticks);
    }

    /// A series on the right axis is drawn by its own scale, labelled in its unit there.
    #[test]
    fn a_secondary_series_is_measured_on_the_right() {
        let mut rain = spec("Rain", &[Some(2.0), Some(4.0)]);
        rain.extras.secondary = true;
        let mut spec = line(
            vec![spec("A", &[Some(120.0), Some(310.0)]), rain],
            &["Q1", "Q2"],
        );
        spec.y2_unit = Some("mm".to_string());
        spec.axes.y2_title = Some("Rainfall".to_string());
        let drawn = draw_line(&spec).expect("drawn");
        let plot = &drawn.geometry.plot;
        assert!(drawn.svg.contains("\nRainfall\n"), "{}", drawn.svg);
        assert!(!drawn.svg.contains(AXIS2_MARK), "{}", drawn.svg);
        let mut rest = drawn.svg.as_str();
        let mut right = 0;
        while let Some(at) = rest.find(" mm\n") {
            let tag = &rest[rest[..at].rfind("<text").expect("a label")..at];
            let x = attribute(&attributes(tag), "x").and_then(|x| x.parse::<f32>().ok());
            assert!(x.is_some_and(|x| x >= plot.right), "{tag}");
            right += 1;
            rest = &rest[at + 1..];
        }
        assert!(right >= 2, "{}", drawn.svg);
        let high = drawn
            .geometry
            .marks
            .iter()
            .find(|m| m.series == 1 && m.index == 1);
        let Some(MarkShape::Point { y, .. }) = high.map(|mark| &mark.shape) else {
            panic!("no point");
        };
        assert!(*y < (plot.top + plot.bottom) / 2.0, "{y} in {plot:?}");
    }

    /// Error ranges fill a band around a line, only where it has points, on an axis that
    /// spans them; the marks stay its points. A stack draws none.
    #[test]
    fn error_ranges_draw_a_band_around_the_line() {
        let series = spec("A", &[Some(2.0), None, Some(4.0), Some(3.0)]);
        let mut spec = line(vec![series], &["a", "b", "c", "d"]);
        spec.errors = vec![vec![
            Some((1.0, 3.0)),
            Some((1.0, 2.0)),
            Some((3.0, 9.0)),
            Some((2.0, 4.0)),
        ]];
        let drawn = draw_line(&spec).expect("drawn");
        assert_eq!(
            drawn.svg.matches("class=\"ct-band\"").count(),
            1,
            "{}",
            drawn.svg
        );
        assert!(
            drawn.svg.contains("9 kg") || drawn.svg.contains("10 kg"),
            "{}",
            drawn.svg
        );
        assert_eq!(drawn.geometry.marks.len(), 3);
        spec.stacked = true;
        let stacked = draw_line(&spec).expect("drawn");
        assert!(!stacked.svg.contains("ct-band"), "{}", stacked.svg);
    }

    /// Each shape draws its own marker at every point, in the series' colour; the points are
    /// still harvested.
    #[test]
    fn a_series_draws_its_point_symbol() {
        let mut series = spec("A", &[Some(1.0), Some(3.0), Some(2.0)]);
        for (symbol, tag) in [
            (PointSymbol::Square, "<rect"),
            (PointSymbol::Triangle, "<polygon"),
            (PointSymbol::Diamond, "<polygon"),
        ] {
            series.extras.symbol = symbol;
            let drawn = draw_line(&line(vec![series.clone()], &["a", "b", "c"])).expect("drawn");
            assert_eq!(
                drawn.svg.matches(tag).count(),
                3,
                "{symbol:?}: {}",
                drawn.svg
            );
            assert!(!drawn.svg.contains("<circle"), "{symbol:?}: {}", drawn.svg);
            assert_eq!(drawn.geometry.marks.len(), 3, "{symbol:?}");
        }
    }

    /// An area is filled see-through at the theme's opacity; stacked, each line sits on the
    /// one before and its point keeps its own value.
    #[test]
    fn stacked_areas_fill_between_the_lines() {
        let series = vec![
            spec("A", &[Some(1.0), Some(2.0)]),
            spec("B", &[Some(3.0), Some(1.0)]),
        ];
        let mut spec = line(series, &["a", "b"]);
        spec.area = Some(Theme::DEFAULT.chart.area_opacity);
        let alone = draw_line(&spec).expect("drawn");
        spec.stacked = true;
        let stacked = draw_line(&spec).expect("drawn");
        assert!(
            stacked.svg.contains("fill-opacity=\"0.2\""),
            "{}",
            stacked.svg
        );
        let values: Vec<f64> = stacked.geometry.marks.iter().map(|m| m.value).collect();
        assert_eq!(values, [1.0, 2.0, 3.0, 1.0]);
        let y = |drawn: &Drawn, i: usize| match drawn.geometry.marks[i].shape {
            MarkShape::Point { y, .. } => y,
            _ => panic!(),
        };
        // At b, B's 1 is under A's 2 alone, stacked over it at 2 + 1.
        assert!(y(&alone, 3) > y(&alone, 1), "{}", alone.svg);
        assert!(y(&stacked, 3) < y(&stacked, 1), "{}", stacked.svg);
    }

    /// A mark line is drawn at its value in the series' colour and labelled in the label ink;
    /// one past the axis' ends or in a stack is not drawn.
    #[test]
    fn mark_lines_are_drawn_and_labelled() {
        let mut series = spec("A", &[Some(1.0), Some(3.0), Some(2.0)]);
        series.extras.mark_lines = vec![(2.0, "Average: 2".into()), (9.0, "Out".into())];
        let mut spec = line(vec![series.clone()], &["a", "b", "c"]);
        let drawn = draw_line(&spec).expect("drawn");
        assert_eq!(
            drawn.svg.matches("stroke-dasharray=\"4,2\"").count(),
            1,
            "{}",
            drawn.svg
        );
        let label = format!(
            "fill=\"{}\" stroke=\"{}\"",
            spec.ink.label, spec.ink.surface
        );
        assert!(drawn.svg.contains(&label), "{}", drawn.svg);
        assert!(drawn.svg.contains("\nAverage: 2\n"), "{}", drawn.svg);
        assert!(!drawn.svg.contains("Out"), "{}", drawn.svg);
        assert_eq!(drawn.geometry.marks.len(), 3);
        spec.stacked = true;
        assert!(!draw_line(&spec).expect("drawn").svg.contains("Average"));

        let mut bars = bars(BarLayout::Grouped, false);
        bars.series[0].extras.mark_lines = vec![(1.0, "Target: 1".into())];
        assert!(
            draw_bar(&bars)
                .expect("drawn")
                .svg
                .contains("\nTarget: 1\n")
        );
        bars.horizontal = true;
        assert!(!draw_bar(&bars).expect("drawn").svg.contains("Target"));
    }

    /// A marked point's value sits in a pill of its colour above it, or below it at the top,
    /// in the ink that reads on that colour.
    #[test]
    fn a_marked_point_is_pinned_by_its_value() {
        let mut series = spec("A", &[Some(1.0), Some(3.0), Some(2.0)]);
        series.extras.mark_points = vec![(0, "1 kg".into()), (1, "3 kg".into())];
        let spec = line(vec![series.clone()], &["a", "b", "c"]);
        let drawn = draw_line(&spec).expect("drawn");
        assert_eq!(
            drawn.svg.matches("class=\"ct-pin\"").count(),
            2,
            "{}",
            drawn.svg
        );
        let ink = format!("fill=\"{}\">3 kg</text>", spec.ink.on(series.color));
        assert!(drawn.svg.contains(&ink), "{}", drawn.svg);
        let tops: Vec<f32> = drawn
            .svg
            .split("class=\"ct-pin\"><rect")
            .skip(1)
            .filter_map(|pin| pin.split("y=\"").nth(1)?.split('"').next()?.parse().ok())
            .collect();
        let ys: Vec<f32> = drawn.geometry.marks[..2]
            .iter()
            .map(|m| match m.shape {
                MarkShape::Point { y, .. } => y,
                _ => panic!(),
            })
            .collect();
        assert!(
            tops[0] < ys[0],
            "the low point's pin is above it: {tops:?} {ys:?}"
        );
        assert_eq!(drawn.geometry.marks.len(), 3);
    }

    /// A lying bar chart's value axis is at the bottom: the y title goes there.
    #[test]
    fn a_horizontal_bar_chart_swaps_its_titles() {
        let mut spec = bars(BarLayout::Grouped, true);
        spec.axes.x_title = Some("Team".to_string());
        spec.axes.y_title = Some("Score".to_string());
        let svg = draw_bar(&spec).expect("drawn").svg;
        let rotated = svg
            .split("<text")
            .find(|tag| tag.contains("rotate(-90)"))
            .unwrap_or_else(|| panic!("{svg}"));
        assert!(rotated.contains("\nTeam\n"), "{rotated}");
        assert!(svg.contains("\nScore\n"), "{svg}");
    }

    fn spec(name: &str, values: &[Option<f64>]) -> SeriesSpec {
        SeriesSpec {
            name: name.to_string(),
            values: values.to_vec(),
            color: chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6),
            dashed: false,
            extras: SeriesExtras::default(),
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
        assert!(drawn.svg.contains("25.000 kg"), "{}", drawn.svg);
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
            axes: axes(),
            labels_inside: layout != BarLayout::Grouped || horizontal,
            bar_colors: Vec::new(),
            errors: Vec::new(),
            bar_width: None,
            bar_gap: RENDERER_GAP,
            grid: true,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: None,
            y2_unit: None,
            curve: Curve::Linear,
            points: true,
            stroke_width: 2.0,
            formats: &Formats::GERMAN,
        }
    }

    /// A line among bars: its points are marks after the bars', centred in their bands,
    /// and it takes no slot of the group.
    #[test]
    fn a_line_over_bars_is_harvested_after_them() {
        let mut spec = bars(BarLayout::Grouped, false);
        spec.series[1].extras.line = true;
        spec.bar_width = Some(0.6);
        let drawn = draw_bar(&spec).expect("drawn");
        let marks = &drawn.geometry.marks;

        assert_eq!(marks.len(), 5, "{marks:?}");
        assert!(
            marks[..2]
                .iter()
                .all(|m| m.series == 0 && matches!(m.shape, MarkShape::Rect { .. }))
        );
        assert!(
            marks[2..]
                .iter()
                .all(|m| m.series == 1 && matches!(m.shape, MarkShape::Point { .. }))
        );
        let plot = drawn.geometry.plot;
        let band = (plot.right - plot.left) / 3.0;
        for mark in &marks[2..] {
            let MarkShape::Point { x, .. } = mark.shape else {
                unreachable!()
            };
            let middle = plot.left + band * (mark.index as f32 + 0.5);
            assert!((x - middle).abs() < 1.0, "{x} vs {middle}");
        }
        // The one bar series fills the whole 60% of its band.
        let (_, _, width, _) = rect(&marks[0]);
        assert!(
            (width - band * 0.6).abs() < 1.0,
            "{width} vs {}",
            band * 0.6
        );
    }

    /// A line on the right axis is measured by its own scale and unit.
    #[test]
    fn a_line_over_bars_can_take_the_right_axis() {
        let mut chart = bars(BarLayout::Stacked, false);
        chart.series.push(SeriesSpec {
            extras: SeriesExtras {
                line: true,
                secondary: true,
                ..Default::default()
            },
            ..spec("Share", &[Some(0.2), Some(0.5), Some(0.4)])
        });
        chart.y2_unit = Some("%".to_string());
        chart.axes.y2_title = Some("Share".to_string());
        let drawn = draw_bar(&chart).expect("drawn");

        assert_eq!(drawn.geometry.marks.len(), 8);
        // The right axis writes its unit, the left one none; its title stands beside it.
        assert_eq!(drawn.svg.matches("%\n</text>").count(), 5, "{}", drawn.svg);
        assert!(
            drawn.svg.contains("rotate(90)\"") && drawn.svg.contains("Share"),
            "{}",
            drawn.svg
        );
    }

    /// A stacked bar on the right axis is a stack of its own beside the left one, here too
    /// when the bars are set again at their own width.
    #[test]
    fn a_bar_on_the_right_axis_stacks_apart() {
        for bar_width in [None, Some(0.8)] {
            let mut chart = bars(BarLayout::Stacked, false);
            chart.series[1].extras.secondary = true;
            chart.bar_width = bar_width;
            let drawn = draw_bar(&chart).expect("drawn");
            let marks = &drawn.geometry.marks;
            let at = |series: usize| {
                let mark = marks
                    .iter()
                    .find(|m| m.series == series && m.index == 0)
                    .expect("a mark");
                rect(mark)
            };
            let (left, right) = (at(0), at(1));
            assert!(left.0 + left.2 <= right.0 + 0.5, "{left:?} {right:?}");
        }
    }

    /// A line among bars pins its marked point by its value, as a line chart does.
    #[test]
    fn a_line_over_bars_pins_its_marked_point() {
        let mut chart = bars(BarLayout::Grouped, false);
        chart.series.push(SeriesSpec {
            extras: SeriesExtras {
                line: true,
                mark_points: vec![(1, "Peak 7".to_string())],
                ..Default::default()
            },
            ..spec("Target", &[Some(4.0), Some(7.0), Some(5.0)])
        });
        let drawn = draw_bar(&chart).expect("drawn");
        assert_eq!(
            drawn.svg.matches("class=\"ct-pin\"").count(),
            1,
            "{}",
            drawn.svg
        );
        assert!(drawn.svg.contains("Peak 7"), "{}", drawn.svg);
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

    /// Which corners of each bar path are rounded, as `[top left, top right, bottom right,
    /// bottom left]`, in drawing order.
    fn corners(svg: &str) -> Vec<[bool; 4]> {
        svg.split("<path d=\"")
            .skip(1)
            .map(|path| {
                let d = path.split('"').next().unwrap_or_default();
                let arcs: Vec<bool> = d
                    .split('A')
                    .skip(1)
                    .map(|arc| arc.split(',').next() != Some("0"))
                    .collect();
                // The path walks top right, bottom right, bottom left, top left.
                [arcs[3], arcs[0], arcs[1], arcs[2]]
            })
            .collect()
    }

    const TOP: [bool; 4] = [true, true, false, false];
    const BOTTOM: [bool; 4] = [false, false, true, true];
    const RIGHT: [bool; 4] = [false, true, true, false];
    const LEFT: [bool; 4] = [true, false, false, true];
    const SQUARE: [bool; 4] = [false; 4];

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
        assert_eq!(corners(&drawn.svg), [TOP; 5], "{}", drawn.svg);
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

    /// Shares of the absolute sum, so a stack with losses runs from its deepest loss to 100%,
    /// never past it (3470).
    #[test]
    fn a_percent_axis_with_losses_stays_within_100() {
        let mut spec = bars(BarLayout::Percent, false);
        spec.x.push("d".to_string());
        spec.height = 480.0;
        spec.series = vec![
            self::spec("A", &[Some(40.0), Some(55.0), Some(-25.0), Some(62.0)]),
            self::spec("B", &[Some(30.0), Some(-15.0), Some(20.0), Some(48.0)]),
        ];
        let svg = draw_bar(&spec).expect("drawn").svg;
        assert!(svg.contains("100%") && svg.contains("60%"), "{svg}");
        assert!(!svg.contains("120%"), "{svg}");
        let axes = percent_axes(&spec.axes, &spec.series);
        assert_eq!(
            (axes.min, axes.max, axes.ticks),
            (Some(-60.0), Some(100.0), 8)
        );
    }

    /// The `[x1, y1, x2, y2]` of every error-bar line the engine drew itself.
    fn error_lines(svg: &str) -> Vec<[f32; 4]> {
        svg.split("<line stroke-width=\"1.5\"")
            .skip(1)
            .map(|tail| {
                let tag = format!("<line{}", &tail[..tail.find("/>").unwrap_or(tail.len())]);
                let attributes = attributes(&tag);
                ["x1", "y1", "x2", "y2"].map(|name| {
                    attribute(&attributes, name)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(f32::NAN)
                })
            })
            .collect()
    }

    /// A range on a stack or a lying bar is read from the end of its own segment (3480).
    #[test]
    fn a_range_on_a_stacked_or_lying_bar_starts_at_its_segment() {
        for horizontal in [false, true] {
            let mut spec = bars(BarLayout::Stacked, horizontal);
            spec.series[1].values = vec![Some(3.0), Some(1.0), Some(2.0)];
            spec.errors = vec![vec![], vec![Some((2.0, 5.0)), None, None]];
            let drawn = draw_bar(&spec).expect("drawn");
            let b = drawn
                .geometry
                .marks
                .iter()
                .find(|m| m.series == 1 && m.index == 0)
                .expect("B at a");
            let MarkShape::Rect {
                x,
                y,
                width,
                height,
            } = &b.shape
            else {
                panic!("a bar")
            };
            let (x, y, width, height) = (*x, *y, *width, *height);
            // The stem runs from the value 2 to the value 5, over a bar of 3.
            let stem = error_lines(&drawn.svg)
                .into_iter()
                .find(|l| match horizontal {
                    true => l[1] == l[3],
                    false => l[0] == l[2],
                })
                .expect("a stem");
            let (end, k) = match horizontal {
                true => (x + width, width / 3.0),
                false => (y, height / 3.0),
            };
            let [from, to] = match horizontal {
                true => [stem[0], stem[2]],
                false => [stem[1], stem[3]],
            };
            let (a, b) = match horizontal {
                true => (end - k, end + 2.0 * k),
                false => (end + k, end - 2.0 * k),
            };
            assert!(
                (from - a).abs() < 0.2 && (to - b).abs() < 0.2,
                "{stem:?} vs {a} {b}"
            );
            // The axis reaches the range's far end.
            let plot = &drawn.geometry.plot;
            let (start, stop) = match horizontal {
                true => (plot.left, plot.right),
                false => (plot.top, plot.bottom),
            };
            assert!(
                from.min(to) >= start - 0.2 && from.max(to) <= stop + 0.2,
                "{stem:?} outside {start}..{stop}"
            );
        }
    }

    /// A bar rounds only its free end: A's bars sit under B's, B's are on top or alone.
    #[test]
    fn a_stack_rounds_only_its_top_segment() {
        for (layout, horizontal, free) in [
            (BarLayout::Stacked, false, TOP),
            (BarLayout::Stacked, true, RIGHT),
            (BarLayout::Percent, false, TOP),
            (BarLayout::Percent, true, RIGHT),
        ] {
            let drawn = draw_bar(&bars(layout, horizontal)).expect("drawn");
            assert!(!drawn.svg.contains("<rect"), "{}", drawn.svg);
            let want = [SQUARE, SQUARE, free, free, free];
            assert_eq!(corners(&drawn.svg), want, "{}", drawn.svg);
        }
    }

    /// Grouped, every bar rounds its free end.
    #[test]
    fn grouped_bars_round_only_their_free_end() {
        for (horizontal, free) in [(false, TOP), (true, RIGHT)] {
            let drawn = draw_bar(&bars(BarLayout::Grouped, horizontal)).expect("drawn");
            assert_eq!(corners(&drawn.svg), [free; 5], "{}", drawn.svg);
        }
    }

    /// A negative value rounds its far end below (left of) the axis; a mixed stack the
    /// topmost positive and the lowest negative segment.
    #[test]
    fn a_negative_bar_rounds_the_end_away_from_the_axis() {
        for (horizontal, up, down) in [(false, TOP, BOTTOM), (true, RIGHT, LEFT)] {
            let spec = BarSpec {
                series: vec![
                    spec("A", &[Some(2.0), Some(-1.0), Some(2.0)]),
                    spec("B", &[Some(3.0), Some(-2.0), Some(-1.0)]),
                ],
                ..bars(BarLayout::Stacked, horizontal)
            };
            let drawn = draw_bar(&spec).expect("drawn");
            let marks = &drawn.geometry.marks;
            let mut found: Vec<_> = marks
                .iter()
                .zip(corners(&drawn.svg))
                .map(|(mark, corners)| ((mark.series, mark.index), corners))
                .collect();
            found.sort_by_key(|(place, _)| *place);
            let found: Vec<_> = found.into_iter().map(|(_, corners)| corners).collect();
            // A at a, b, c, then B at a, b, c.
            let want = [SQUARE, SQUARE, up, up, down, down];
            assert_eq!(found, want, "{}", drawn.svg);
        }
    }

    /// The `name` attribute of every `tag` in the SVG that has one.
    fn numbers(svg: &str, tag: &str, name: &str) -> Vec<f32> {
        svg.split(tag)
            .skip(1)
            .filter_map(|tag| {
                let tag = &tag[..tag.find('>')?];
                tag.split(&format!(" {name}=\""))
                    .nth(1)
                    .or_else(|| tag.strip_prefix(&format!("{name}=\"")))?
                    .split('"')
                    .next()?
                    .parse()
                    .ok()
            })
            .collect()
    }

    fn rects(drawn: &Drawn) -> Vec<[f32; 4]> {
        drawn
            .geometry
            .marks
            .iter()
            .map(|mark| match mark.shape {
                MarkShape::Rect {
                    x,
                    y,
                    width,
                    height,
                } => [x, y, width, height],
                _ => panic!("{mark:?}"),
            })
            .collect()
    }

    /// Set again at the renderer's own width and gap, every bar stays where it was drawn.
    #[test]
    fn a_relayout_at_the_renderer_s_measures_moves_nothing() {
        for (layout, horizontal) in [
            (BarLayout::Grouped, false),
            (BarLayout::Grouped, true),
            (BarLayout::Stacked, false),
        ] {
            let spec = bars(layout, horizontal);
            let drawn = draw_bar(&spec).expect("drawn");
            // The marks are in the renderer's order; it lists a lying chart's x bottom up.
            let marks = &drawn.geometry.marks;
            let triggers: Vec<Trigger> = marks
                .iter()
                .map(|mark| Trigger {
                    series: spec.series[mark.series].name.clone(),
                    shape: mark.shape.clone(),
                })
                .collect();
            let expected: Vec<_> = marks
                .iter()
                .map(|mark| {
                    let index = if horizontal {
                        2 - mark.index
                    } else {
                        mark.index
                    };
                    (mark.series, index, mark.value, "")
                })
                .collect();
            let (_, moved) = relayout(&drawn.svg, triggers.clone(), &expected, &spec);
            for (before, after) in triggers.iter().zip(&moved) {
                let (
                    MarkShape::Rect {
                        x: a,
                        y: b,
                        width: c,
                        height: d,
                    },
                    MarkShape::Rect {
                        x: e,
                        y: f,
                        width: g,
                        height: h,
                    },
                ) = (&before.shape, &after.shape)
                else {
                    panic!("{before:?}");
                };
                for (old, new) in [(a, e), (b, f), (c, g), (d, h)] {
                    assert!(
                        (old - new).abs() < 0.2,
                        "{layout:?} {horizontal}: {before:?} to {after:?}"
                    );
                }
            }
        }
    }

    /// Half its band, each bar of a pair is half that less the gap's half; its label moves
    /// with its centre.
    #[test]
    fn bars_fill_their_share_of_the_band() {
        let mut spec = bars(BarLayout::Grouped, false);
        spec.bar_width = Some(0.5);
        spec.bar_gap = 10.0;
        let wide = draw_bar(&spec).expect("drawn");
        let plot = wide.geometry.plot;
        let unit = (plot.right - plot.left) / 3.0;
        let found = rects(&wide);
        let labels = numbers(&wide.svg, "<text ", "x");
        for [x, _, width, _] in &found {
            assert!((width - (unit * 0.5 - 10.0) / 2.0).abs() < 0.2, "{found:?}");
            let centre = x + width / 2.0;
            assert!(
                labels.iter().any(|label| (label - centre).abs() < 0.2),
                "no label at {centre}: {labels:?}"
            );
        }
        // A of b and B of b: one gap apart.
        let (a, b) = (found[1], found[3]);
        assert!((b[0] - (a[0] + a[2]) - 10.0).abs() < 0.2, "{found:?}");
    }

    /// A band narrower than the renderer's margins still holds its bars, with a gap set.
    #[test]
    fn bars_stay_inside_a_narrow_band() {
        let mut narrow = bars(BarLayout::Grouped, false);
        narrow.x = (0..40).map(|i| i.to_string()).collect();
        narrow.series = vec![spec("A", &[Some(1.0); 40]), spec("B", &[Some(2.0); 40])];
        narrow.bar_gap = 2.0;
        let drawn = draw_bar(&narrow).expect("drawn");
        let plot = drawn.geometry.plot;
        let unit = (plot.right - plot.left) / 40.0;
        assert!(
            unit < 2.0 * RENDERER_MARGIN,
            "the band is not narrow: {unit}"
        );
        let found = rects(&drawn);
        assert_eq!(found.len(), 80, "{found:?}");
        for [x, _, width, _] in &found {
            let band = ((x - plot.left) / unit).floor();
            let end = plot.left + unit * (band + 1.0);
            assert!(
                *width > 1.0 && x + width <= end + 0.1,
                "{x}+{width} past {end}"
            );
        }
    }

    /// A bar's own colour wins over its series'; a negative one's too; its label reads on it.
    #[test]
    fn a_bar_takes_its_own_colour() {
        let mut spec = bars(BarLayout::Stacked, false);
        let (red, dark) = (HexColor::new(0xE03131), HexColor::new(0x101010));
        spec.bar_colors = vec![vec![None, Some(red)], vec![Some(dark)]];
        let svg = draw_bar(&spec).expect("drawn").svg;
        assert!(svg.contains(&format!("fill=\"{red}\"")), "{svg}");
        assert!(svg.contains(&format!("fill=\"{dark}\"")), "{svg}");
        assert!(
            svg.contains("fill=\"#FFFFFF\""),
            "white on the dark bar: {svg}"
        );
    }

    /// Upright grouped bars carry their error ranges, the axis reaching the highest end; a
    /// percent stack carries none.
    #[test]
    fn error_bars_stand_on_upright_grouped_bars() {
        let mut spec = bars(BarLayout::Grouped, false);
        spec.value_labels = false;
        spec.errors = vec![vec![Some((0.5, 1.5)), Some((2.0, 9.0))]];
        let drawn = draw_bar(&spec).expect("drawn");
        let [x, _, width, _] = rects(&drawn)[1];
        let centre = x + width / 2.0;
        let upright = numbers(&drawn.svg, "<line ", "x1")
            .into_iter()
            .zip(numbers(&drawn.svg, "<line ", "x2"))
            .any(|(x1, x2)| (x1 - centre).abs() < 0.2 && (x2 - centre).abs() < 0.2);
        assert!(upright, "no error bar at {centre}: {}", drawn.svg);
        assert!(drawn.svg.contains("\n10\n"), "{}", drawn.svg);
        assert!(!drawn.svg.contains("#010203"), "{}", drawn.svg);

        spec.layout = BarLayout::Percent;
        let percent = draw_bar(&spec).expect("drawn").svg;
        assert!(!percent.contains("<line stroke-width=\"1.5\""), "{percent}");
    }

    /// Outside, a lying bar's label is past its end in the page's ink.
    #[test]
    fn a_lying_bar_s_label_can_sit_past_its_end() {
        let mut spec = bars(BarLayout::Grouped, true);
        spec.labels_inside = false;
        let drawn = draw_bar(&spec).expect("drawn");
        assert!(!drawn.svg.contains(ON_BAR_FILL), "{}", drawn.svg);
        assert_eq!(overflow(&drawn.svg, 320.0, 180.0), None, "{}", drawn.svg);
    }

    fn pie(labels: PieLabels, donut: Option<f32>) -> PieSpec {
        PieSpec {
            width: 320.0,
            height: 320.0,
            slices: vec![spec("A", &[Some(1.0)]), spec("B", &[Some(3.0)])],
            donut,
            labels,
            start_angle: 0.0,
            end_angle: None,
            rose: false,
            slice_gap: 1.0,
            slice_radius: 0.0,
            labels_inside: false,
            min_label_angle: 0.0,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: Some("kg".to_string()),
            formats: &Formats::GERMAN,
        }
    }

    /// A slice's corners are circular fillets of the radius, tangent to the edges and the
    /// arcs, in the drawing and in what the pointer hits (3479).
    #[test]
    fn a_slice_s_corners_are_rounded_by_the_radius() {
        for (donut, fillets) in [(None, 2), (Some(0.4), 4)] {
            let mut spec = pie(PieLabels::None, donut);
            spec.slice_radius = 8.0;
            let drawn = draw_pie(&spec).expect("drawn");
            assert_eq!(drawn.geometry.marks.len(), 2);
            for mark in &drawn.geometry.marks {
                let MarkShape::Sector { path } = &mark.shape else {
                    panic!("a slice")
                };
                assert_eq!(path.matches("A8,8 ").count(), fillets, "{path}");
                assert!(!path.contains("A0 0"), "{path}");
                assert!(drawn.svg.contains(&format!("d=\"{path}\"")), "{path}");
            }
        }
    }

    /// A thin slice gives up corner radius rather than fold over itself.
    #[test]
    fn a_thin_slice_keeps_its_corners_inside_it() {
        let mut spec = pie(PieLabels::None, None);
        spec.slices = vec![
            self::spec("A", &[Some(1.0)]),
            self::spec("B", &[Some(99.0)]),
        ];
        spec.slice_radius = 40.0;
        let drawn = draw_pie(&spec).expect("drawn");
        let MarkShape::Sector { path } = &drawn.geometry.marks[0].shape else {
            panic!("a slice")
        };
        let radii: Vec<f32> = path
            .split(" A")
            .skip(1)
            .filter_map(|a| a.split(',').next()?.parse().ok())
            .collect();
        assert!(radii[0] < 4.0, "{path}");
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

    fn sector_points(mark: &Mark) -> Vec<(f32, f32)> {
        match &mark.shape {
            MarkShape::Sector { path } => super::super::hit::outline(path).concat(),
            _ => panic!("{mark:?}"),
        }
    }

    /// A half donut from 9 to 3 o'clock lies above its middle and takes a larger radius
    /// than a full one; its `center` room is the upper half of the hole.
    #[test]
    fn a_half_donut_bends_over_its_middle() {
        let mut full = pie(PieLabels::None, Some(0.4));
        full.height = 180.0;
        let mut half = full.clone();
        let full = draw_pie(&full).expect("drawn");
        (half.start_angle, half.end_angle) = (-90.0, Some(90.0));
        let half = draw_pie(&half).expect("drawn");
        let points: Vec<(f32, f32)> = half.geometry.marks.iter().flat_map(sector_points).collect();
        let (low, high) = points.iter().fold((f32::MAX, f32::MIN), |(low, high), p| {
            (low.min(p.1), high.max(p.1))
        });
        let room = half.geometry.plot;
        let middle_y = room.bottom;
        assert!(high <= middle_y + 0.5, "{high} below {middle_y}");
        assert!(half.geometry.hole > full.geometry.hole, "{half:?}");
        assert!(high - low > full.geometry.hole, "{low}..{high}");
        assert!(
            (room.right - room.left - half.geometry.hole).abs() < 0.5,
            "{room:?}"
        );
    }

    /// An end before the start counts a turn on: 3 o'clock to 9 o'clock is the lower half.
    #[test]
    fn an_end_before_the_start_wraps_round() {
        let mut wrapped = pie(PieLabels::None, Some(0.4));
        (wrapped.start_angle, wrapped.end_angle) = (90.0, Some(-90.0));
        let mut lower = wrapped.clone();
        lower.end_angle = Some(270.0);
        assert_eq!(wrapped.span(), 180.0);
        let (wrapped, lower) = (draw_pie(&wrapped), draw_pie(&lower));
        assert_eq!(
            wrapped.map(|d| d.geometry.marks),
            lower.map(|d| d.geometry.marks)
        );
    }

    /// A rose gives every slice the same angle and the larger value the longer radius.
    #[test]
    fn a_rose_s_slices_reach_out_by_value() {
        let mut rose = pie(PieLabels::None, None);
        rose.rose = true;
        let drawn = draw_pie(&rose).expect("drawn");
        let reach = |mark: &Mark| {
            sector_points(mark)
                .iter()
                .map(|(x, y)| (x - 160.0).hypot(y - 160.0))
                .fold(0.0, f32::max)
        };
        let marks = &drawn.geometry.marks;
        assert!(reach(&marks[1]) > reach(&marks[0]) + 10.0, "{marks:?}");
        // A, the smaller value, still spans half the turn: it reaches 6 o'clock.
        assert!(
            sector_points(&marks[0])
                .iter()
                .any(|(x, y)| (x - 160.0).abs() < 1.0 && *y > 160.0),
            "{marks:?}"
        );
    }

    /// On its slice a label takes the ink that reads there; a slice narrower than the
    /// least angle has none. The slices part by the gap.
    #[test]
    fn labels_inside_read_on_their_slice() {
        let mut inside = pie(PieLabels::Percent, None);
        inside.labels_inside = true;
        inside.slice_gap = 4.0;
        let drawn = draw_pie(&inside).expect("drawn");
        let on = inside.ink.on(inside.slices[1].color);
        assert!(
            drawn.svg.contains(&format!("fill=\"{on}\"")),
            "{}",
            drawn.svg
        );
        assert!(!drawn.svg.contains(ON_BAR_FILL), "{}", drawn.svg);
        assert!(drawn.svg.contains("stroke-width=\"4\""), "{}", drawn.svg);

        inside.min_label_angle = 100.0;
        let svg = draw_pie(&inside).expect("drawn").svg;
        assert!(svg.contains("75%") && !svg.contains("25%"), "{svg}");
    }

    /// Of two outside labels that overlap, the smaller slice's leaves with its leader line;
    /// the largest slice keeps its label and no two left overlap.
    #[test]
    fn colliding_outside_labels_keep_the_larger_slice_s() {
        let mut many = pie(PieLabels::Percent, None);
        (many.width, many.height) = (304.0, 168.0);
        let values: Vec<f64> = (1..=14).map(f64::from).collect();
        many.slices = values
            .iter()
            .map(|value| spec(&format!("S{value}"), &[Some(*value)]))
            .collect();
        let svg = draw_pie(&many).expect("drawn").svg;
        let labels = text_contents(&svg);
        assert!(labels.len() < 14, "{svg}");
        assert!(
            labels.iter().any(|label| label.starts_with("13,3")),
            "{svg}"
        );
        assert_eq!(
            svg.matches("fill=\"none\"").count(),
            labels.len(),
            "a leader line per label: {svg}"
        );
        assert_eq!(drop_colliding_labels(&svg, &values), svg);
    }

    /// A slice that takes the whole pie, beside empty ones, shows no seam up to 12 o'clock.
    #[test]
    fn a_lone_slice_has_no_seam() {
        let mut lone = pie(PieLabels::Percent, None);
        lone.slice_gap = 4.0;
        lone.slices = vec![spec("A", &[Some(0.0)]), spec("B", &[Some(3.0)])];
        let svg = draw_pie(&lone).expect("drawn").svg;
        assert!(!svg.contains("stroke-width=\"4\""), "{svg}");
        assert!(svg.contains("stroke-width=\"0\""), "{svg}");
    }

    /// A label inside sits on its own slice: not in a thin donut's hole, nor by a rose's
    /// small slice's centre.
    #[test]
    fn labels_inside_stay_on_their_slice() {
        for rose in [false, true] {
            let mut inside = pie(PieLabels::Percent, Some(0.25));
            inside.slices = vec![
                spec("A", &[Some(60.0)]),
                spec("B", &[Some(25.0)]),
                spec("C", &[Some(15.0)]),
            ];
            inside.rose = rose;
            inside.labels_inside = true;
            let drawn = draw_pie(&inside).expect("drawn");
            let outlines: Vec<_> = drawn
                .geometry
                .marks
                .iter()
                .map(|mark| match &mark.shape {
                    MarkShape::Sector { path } => super::super::hit::outline(path),
                    _ => panic!("{mark:?}"),
                })
                .collect();
            let middles: Vec<(f32, f32)> = numbers(&drawn.svg, "<text ", "x")
                .into_iter()
                .zip(numbers(&drawn.svg, "<text ", "y"))
                .map(|(x, y)| (x, y - 14.0 * 0.35))
                .collect();
            // The rose's smallest slice is a ring thinner than its label: that one is dropped.
            assert_eq!(middles.len(), if rose { 2 } else { 3 }, "{}", drawn.svg);
            for (slice, (x, y)) in middles.iter().enumerate() {
                assert!(
                    super::super::hit::contains(&outlines[slice], *x, *y),
                    "rose {rose}: label {slice} at {x},{y} off its slice: {}",
                    drawn.svg
                );
            }
        }
    }

    /// A label wider than its slice is dropped rather than spill onto the next one.
    #[test]
    fn a_label_wider_than_its_slice_is_dropped() {
        let mut inside = pie(PieLabels::Name, None);
        inside.labels_inside = true;
        inside.slices = [("Chrome", 85.0), ("Internet Explorer", 15.0)]
            .map(|(name, value)| spec(name, &[Some(value)]))
            .to_vec();
        let drawn = draw_pie(&inside).expect("drawn");
        assert!(drawn.svg.contains("Chrome"), "{}", drawn.svg);
        assert!(!drawn.svg.contains("Internet Explorer"), "{}", drawn.svg);
        assert_eq!(drawn.geometry.marks.len(), 2, "{:?}", drawn.geometry);
    }

    /// On a half pie, the 10% label fits its slice's width but not its height: it would
    /// reach onto the next slice, so it is dropped.
    #[test]
    fn a_label_taller_than_its_slice_is_dropped() {
        let mut half = pie(PieLabels::Percent, None);
        (half.width, half.height) = (304.0, 168.0);
        (half.start_angle, half.end_angle) = (-90.0, Some(90.0));
        half.labels_inside = true;
        half.slices = [("A", 62.0), ("B", 20.0), ("C", 10.0), ("D", 8.0)]
            .map(|(name, value)| spec(name, &[Some(value)]))
            .to_vec();
        let svg = draw_pie(&half).expect("drawn").svg;
        let labels = text_contents(&svg);
        assert!(labels.iter().any(|label| label.starts_with("62")), "{svg}");
        assert!(!labels.iter().any(|label| label.starts_with("10")), "{svg}");
    }

    /// A donut's ring holds the labels of its wide slices: a box's corner just past the ring,
    /// where no glyph reaches, keeps its label.
    #[test]
    fn a_ring_keeps_its_wide_slices_labels() {
        let mut ring = pie(PieLabels::Percent, Some(0.4));
        (ring.width, ring.height) = (304.0, 168.0);
        ring.labels_inside = true;
        ring.slices = [("A", 62.0), ("B", 20.0), ("C", 10.0), ("D", 8.0)]
            .map(|(name, value)| spec(name, &[Some(value)]))
            .to_vec();
        let svg = draw_pie(&ring).expect("drawn").svg;
        let labels = text_contents(&svg);
        assert!(
            ["62", "20"]
                .iter()
                .all(|share| labels.iter().any(|label| label.starts_with(share))),
            "{svg}"
        );
    }

    fn text_contents(svg: &str) -> Vec<&str> {
        svg.split("<text ")
            .skip(1)
            .filter_map(|text| Some(text.split_once('>')?.1.split('<').next()?.trim()))
            .collect()
    }

    /// Every label keeps inside the SVG: a pie's beside its slices on a phone, a bar's
    /// above the tallest bar, the top y label. The pie stays in the middle.
    #[test]
    fn every_label_keeps_inside_the_svg() {
        let mut narrow = pie(PieLabels::Percent, None);
        (narrow.width, narrow.height) = (280.0, 180.0);
        narrow.slices = [
            ("Chrome", 62.0),
            ("Safari", 20.0),
            ("Firefox", 10.0),
            ("Other", 8.0),
        ]
        .map(|(name, value)| spec(name, &[Some(value)]))
        .to_vec();
        let lines = line(vec![spec("A", &[Some(12.0), Some(180.0)])], &["Mon", "Tue"]);
        for (drawn, width, height) in [
            (draw_pie(&narrow), 280.0, 180.0),
            (draw_bar(&bars(BarLayout::Grouped, false)), 320.0, 180.0),
            (draw_line(&lines), 320.0, 180.0),
        ] {
            let drawn = drawn.expect("drawn");
            assert_eq!(overflow(&drawn.svg, width, height), None, "{}", drawn.svg);
        }
        let drawn = draw_pie(&narrow).expect("drawn");
        let MarkShape::Sector { path } = &drawn.geometry.marks[0].shape else {
            panic!("{drawn:?}")
        };
        assert!(path.starts_with("M140,90 "), "{path}");
    }

    /// A name too long to fit beside a pie at its least radius ends in an ellipsis.
    #[test]
    fn a_long_name_beside_a_pie_is_cut_short() {
        let mut small = pie(PieLabels::Name, None);
        (small.width, small.height) = (288.0, 162.0);
        small.slices = [
            ("Chrome", 50.0),
            ("Internet Explorer", 30.0),
            ("Safari & Co", 20.0),
        ]
        .map(|(name, value)| spec(name, &[Some(value)]))
        .to_vec();
        let drawn = draw_pie(&small).expect("drawn");
        assert_eq!(overflow(&drawn.svg, 288.0, 162.0), None, "{}", drawn.svg);
        assert!(drawn.svg.contains('…'), "{}", drawn.svg);
        assert!(drawn.svg.contains(">\nChrome\n<"), "{}", drawn.svg);
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
        assert!(labels.len() > 2 && svg.contains("25.000"), "{svg}");
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

    /// Each text's `(left, right, baseline, label)`, `dx` and `dy` applied.
    fn texts(svg: &str) -> Vec<(f32, f32, f32, String)> {
        let mut found = Vec::new();
        let mut rest = svg;
        while let Some(open) = rest.find("<text") {
            let close = open + rest[open..].find('>').expect("closed");
            let attributes = attributes(&rest[open..=close]);
            let label = &rest[close + 1..];
            let label = label[..label.find('<').unwrap_or(label.len())].trim();
            rest = &rest[close + 1..];
            let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
            let (Some(x), Some(y)) = (number("x"), number("y")) else {
                continue;
            };
            let (x, y) = (
                x + number("dx").unwrap_or(0.0),
                y + number("dy").unwrap_or(0.0),
            );
            let drawn = text_width(&attributes, label);
            let left = match attribute(&attributes, "text-anchor") {
                Some("middle") => x - drawn / 2.0,
                Some("end") => x - drawn,
                _ => x,
            };
            found.push((left, left + drawn, y, label.to_string()));
        }
        found
    }

    #[test]
    fn the_last_numeric_x_label_keeps_inside_the_svg() {
        let mut numeric = line(
            vec![spec("A", &[Some(3.0), Some(5.0), Some(4.0), Some(8.0)])],
            &["1", "2", "10", "1000.5"],
        );
        numeric.x_numbers = vec![1.0, 2.0, 10.0, 1000.5];
        (numeric.width, numeric.height) = (480.0, 272.0);
        let drawn = draw_line(&numeric).expect("drawn");
        assert_eq!(
            overflow(&drawn.svg, 480.0, 272.0),
            None,
            "{:?}",
            texts(&drawn.svg)
        );
        assert!(
            texts(&drawn.svg).iter().any(|t| t.3 == "1000"),
            "{}",
            drawn.svg
        );
    }

    #[test]
    fn a_bar_s_label_writes_its_own_value_not_the_renderer_s() {
        let mut huge = bars(BarLayout::Grouped, false);
        huge.series = vec![spec("A", &[Some(1.2e9), Some(3.4e9), Some(9.9e9)])];
        let drawn = draw_bar(&huge).expect("drawn");
        let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
        assert!(labels.contains(&"9.900.000.000".to_string()), "{labels:?}");
    }

    /// A long category label is cut to its band; bands too narrow for any word show every
    /// n-th label. Neither runs past the SVG nor over its neighbour.
    #[test]
    fn category_labels_keep_to_their_bands() {
        let row = |chart: &BarSpec| {
            let drawn = draw_bar(chart).expect("drawn");
            let plot = drawn.geometry.plot;
            let below = texts(&drawn.svg).into_iter();
            let row: Vec<_> = below.filter(|t| t.2 > plot.bottom + PLOT_TICK).collect();
            for pair in row.windows(2) {
                assert!(pair[0].1 <= pair[1].0, "{row:?}");
            }
            assert!(
                row.iter().all(|t| t.0 >= 0.0 && t.1 <= chart.width),
                "{row:?}"
            );
            (row, (plot.right - plot.left) / chart.x.len() as f32)
        };
        let mut long = bars(BarLayout::Grouped, false);
        long.x = ["A very long category label", "Another long one", "Short"]
            .map(String::from)
            .to_vec();
        let (labels, band) = row(&long);
        assert_eq!(labels.len(), 3, "{labels:?}");
        assert!(
            labels[0].3.ends_with('…') && labels[2].3 == "Short",
            "{labels:?}"
        );
        assert!(
            labels.iter().all(|t| t.1 - t.0 <= band),
            "{labels:?} {band}"
        );

        let mut many = bars(BarLayout::Grouped, false);
        many.x = (1..=40).map(|i| format!("Week {i}")).collect();
        many.series = vec![spec("A", &[Some(1.0); 40])];
        let (labels, _) = row(&many);
        assert!(labels.len() > 1 && labels.len() < 40, "{labels:?}");
        // Thinned, they have room to be written whole; the last may meet the SVG's side.
        let (_, inner) = labels.split_last().expect("labels");
        assert!(inner.iter().all(|t| t.3.starts_with("Week ")), "{labels:?}");
    }

    #[test]
    fn a_falling_bar_s_label_sits_past_its_lower_end() {
        let mut falling = bars(BarLayout::Grouped, false);
        falling.series = vec![spec("A", &[Some(-10.0), Some(-200.0), Some(35.0)])];
        let drawn = draw_bar(&falling).expect("drawn");
        let left = drawn.geometry.plot.left;
        let found: Vec<_> = texts(&drawn.svg)
            .into_iter()
            .filter(|t| t.0 > left)
            .collect();
        let baseline = |value: &str| found.iter().find(|t| t.3 == value).expect(value).2;
        let ends: Vec<_> = (drawn.geometry.marks.iter())
            .map(|mark| match mark.shape {
                MarkShape::Rect { y, height, .. } => (y, y + height),
                _ => panic!("{mark:?}"),
            })
            .collect();
        assert!(baseline("-10") - 10.0 > ends[0].1, "{found:?} {ends:?}");
        // The lowest bar meets the x axis: its label goes inside, on the bar's ink.
        let lowest = baseline("-200");
        assert!(
            lowest > ends[1].0 && lowest < ends[1].1,
            "{found:?} {ends:?}"
        );
        assert!(baseline("35") < ends[2].0, "{found:?} {ends:?}");
        assert!(!drawn.svg.contains(ON_BAR_FILL), "{}", drawn.svg);
        // The two labels past their bars are haloed, the one inside is not.
        assert_eq!(
            drawn.svg.matches("paint-order=\"stroke\"").count(),
            2,
            "{}",
            drawn.svg
        );
    }

    #[test]
    fn a_mark_line_label_wider_than_the_plot_is_left_out() {
        let mut series = spec("A", &[Some(1.0), Some(3.0), Some(2.0)]);
        series.extras.mark_lines = vec![(2.0, "Average over the whole week: 2".into())];
        let mut narrow = line(vec![series], &["a", "b", "c"]);
        let drawn = draw_line(&narrow).expect("drawn");
        assert!(drawn.svg.contains("Average over"), "{}", drawn.svg);
        (narrow.width, narrow.height) = (160.0, 120.0);
        let drawn = draw_line(&narrow).expect("drawn");
        assert!(!drawn.svg.contains("Average over"), "{}", drawn.svg);
        assert_eq!(drawn.svg.matches("stroke-dasharray=\"4,2\"").count(), 1);
    }

    #[test]
    fn bars_keep_half_a_narrow_band_by_default() {
        let mut narrow = bars(BarLayout::Grouped, false);
        narrow.x = (0..40).map(|i| i.to_string()).collect();
        narrow.series = vec![spec("A", &[Some(1.0); 40])];
        (narrow.width, narrow.height) = (480.0, 272.0);
        let drawn = draw_bar(&narrow).expect("drawn");
        let plot = drawn.geometry.plot;
        let unit = (plot.right - plot.left) / 40.0;
        for [_, _, width, _] in rects(&drawn) {
            assert!(width >= unit / 2.0 - 0.1, "{width} of {unit}");
        }
    }

    #[test]
    fn dense_x_labels_do_not_overlap() {
        let x: Vec<String> = (0..200).map(|i| i.to_string()).collect();
        let x: Vec<&str> = x.iter().map(String::as_str).collect();
        let mut dense = line(vec![spec("A", &[Some(1.0); 200])], &x);
        (dense.width, dense.height) = (480.0, 272.0);
        let drawn = draw_line(&dense).expect("drawn");
        let bottom = drawn.geometry.plot.bottom;
        let mut labels: Vec<_> = texts(&drawn.svg)
            .into_iter()
            .filter(|(.., y, _)| *y > bottom)
            .collect();
        labels.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in labels.windows(2) {
            assert!(pair[0].1 <= pair[1].0, "{pair:?}");
        }
    }

    fn heatmap(values: Vec<Vec<Option<f64>>>) -> HeatmapSpec {
        HeatmapSpec {
            width: 480.0,
            height: 240.0,
            x: (0..values[0].len()).map(|c| format!("C{c}")).collect(),
            y: (0..values.len()).map(|r| format!("R{r}")).collect(),
            values,
            colors: vec![HexColor::new(0xEEEEFF), HexColor::new(0x2233AA)],
            range: (0.0, 10.0),
            steps: 0,
            labels: true,
            scale: false,
            no_value: None,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: None,
            formats: &Formats::AMERICAN,
        }
    }

    /// Cells are harvested row by row from the top, an empty one left out and outlined.
    #[test]
    fn a_heatmap_harvests_its_cells_row_by_row_from_the_top() {
        let drawn = draw_heatmap(&heatmap(vec![
            vec![Some(1.0), Some(2.0), None],
            vec![Some(9.0), Some(5.5), Some(10.0)],
        ]))
        .expect("drawn");
        let cells: Vec<_> = drawn
            .geometry
            .marks
            .iter()
            .map(|m| (m.series, m.index, m.value))
            .collect();
        assert_eq!(
            cells,
            [
                (0, 0, 1.0),
                (0, 1, 2.0),
                (1, 0, 9.0),
                (1, 1, 5.5),
                (1, 2, 10.0)
            ]
        );
        let top = |mark: &Mark| match mark.shape {
            MarkShape::Rect { y, .. } => y,
            _ => panic!("{mark:?}"),
        };
        assert!(top(&drawn.geometry.marks[0]) < top(&drawn.geometry.marks[2]));
        assert!(drawn.geometry.cells);
        assert!(
            drawn.svg.contains("stroke-dasharray=\"4,3\""),
            "{}",
            drawn.svg
        );
        assert!(!drawn.svg.contains("#6E7079") && !drawn.svg.contains("data-"));
    }

    /// The scale sits under the plot, in classes with `steps`, its ends' values below it.
    #[test]
    fn a_heatmap_draws_its_scale_under_the_plot() {
        let mut spec = heatmap(vec![vec![Some(0.0), Some(10.0)]]);
        spec.scale = true;
        spec.steps = 5;
        let drawn = draw_heatmap(&spec).expect("drawn");
        assert!(
            drawn.svg.contains("viewBox=\"0 0 480 240\""),
            "{}",
            drawn.svg
        );
        let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
        for value in ["0", "2", "4", "6", "8", "10"] {
            assert!(labels.contains(&value.to_string()), "{labels:?}");
        }
        assert_eq!(
            drawn.svg.matches("height=\"10\"").count(),
            5,
            "{}",
            drawn.svg
        );
        let bottom = drawn.geometry.plot.bottom;
        assert!(bottom <= 240.0 - SCALE_ROOM + 1.0, "{bottom}");

        spec.values = vec![vec![Some(0.0), None]];
        spec.no_value = Some("No value".to_string());
        let drawn = draw_heatmap(&spec).expect("drawn");
        let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
        assert!(labels.contains(&"No value".to_string()), "{labels:?}");
        assert_eq!(drawn.svg.matches("stroke-dasharray").count(), 2);
    }

    /// Each cell writes its value at its middle; none without labels or in a narrow cell.
    #[test]
    fn a_heatmap_writes_values_that_fit_their_cells() {
        let mut spec = heatmap(vec![vec![Some(1.5), Some(10.0)]]);
        spec.unit = Some("°C".to_string());
        let labels = |spec: &HeatmapSpec| -> Vec<String> {
            let drawn = draw_heatmap(spec).expect("drawn");
            texts(&drawn.svg)
                .into_iter()
                .map(|t| t.3)
                .filter(|t| !t.starts_with('C') && !t.starts_with('R'))
                .collect()
        };
        assert_eq!(labels(&spec), ["1.5 °C", "10 °C"]);
        spec.labels = false;
        assert!(labels(&spec).is_empty());
        spec.labels = true;
        spec.width = 120.0;
        spec.unit = Some("kilowatt hours".to_string());
        assert!(labels(&spec).is_empty());
    }

    fn candles(ohlc: bool) -> CandleSpec {
        CandleSpec {
            width: 400.0,
            height: 300.0,
            x: ["Mon", "Tue", "Wed", "Thu"].map(String::from).to_vec(),
            name: "Price".to_string(),
            candles: vec![
                Some([20.0, 38.0, 10.0, 34.0]),
                Some([40.0, 50.0, 30.0, 35.0]),
                None,
                Some([30.0, 36.0, 28.0, 30.0]),
            ],
            up: HexColor::new(0x2E7D32),
            down: HexColor::new(0xC62828),
            ohlc,
            key: Some(("Up".to_string(), "Down".to_string())),
            axes: axes(),
            grid: true,
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: None,
            formats: &Formats::AMERICAN,
        }
    }

    /// A rising candle is an outline on the surface, a falling one filled, a flat one a line;
    /// each mark spans its wick and a gap has none.
    #[test]
    fn a_rising_candle_is_hollow_and_a_falling_one_filled() {
        let drawn = draw_candles(&candles(false)).expect("drawn");
        let surface = ChartInk::of(&Theme::DEFAULT, false).surface;
        assert!(
            drawn
                .svg
                .contains(&format!("fill=\"{surface}\" stroke=\"#2E7D32\""))
        );
        assert!(drawn.svg.contains("stroke=\"#C62828\" fill=\"#C62828\""));
        assert!(drawn.svg.contains("height=\"1\" fill=\"#2E7D32\""));
        assert!(!drawn.svg.contains(UP_HEX));
        let marks = &drawn.geometry.marks;
        let found: Vec<_> = marks.iter().map(|m| (m.index, m.value)).collect();
        assert_eq!(found, [(0, 34.0), (1, 35.0), (3, 30.0)]);
        let MarkShape::Rect { y, height, .. } = marks[0].shape else {
            panic!("a rect")
        };
        // From the high, 38, to the low, 10, on an axis from 10 to 50.
        let plot = drawn.geometry.plot;
        let at = |price: f32| plot.bottom - (price - 10.0) / 40.0 * (plot.bottom - plot.top);
        assert!((y - at(38.0)).abs() < 0.5 && (y + height - at(10.0)).abs() < 0.5);
    }

    /// OHLC bars draw their ticks in the rising and falling colours, no stand-in left.
    #[test]
    fn ohlc_bars_tick_their_open_and_close() {
        let drawn = draw_candles(&candles(true)).expect("drawn");
        assert!(!drawn.svg.contains(UP_HEX));
        let up = drawn.svg.matches("stroke=\"#2E7D32\"").count();
        let down = drawn.svg.matches("stroke=\"#C62828\"").count();
        // A line and two ticks per bar, two rising (one flat) and one falling; a key each.
        assert_eq!((up, down), (7, 4), "{}", drawn.svg);
        assert_eq!(drawn.geometry.marks.len(), 3);
    }

    fn funnel(labels: PieLabels, values: &[f64]) -> FunnelSpec {
        FunnelSpec {
            width: 480.0,
            height: 320.0,
            stages: values
                .iter()
                .enumerate()
                .map(|(i, v)| spec(&format!("S{i}"), &[Some(*v)]))
                .collect(),
            labels,
            first: values.first().copied().unwrap_or(0.0),
            ink: ChartInk::of(&Theme::DEFAULT, false),
            format: ValueFormat::Auto,
            unit: None,
            formats: &Formats::AMERICAN,
        }
    }

    /// Stages are drawn widest first, each under its own anchor, and walked in data order.
    #[test]
    fn a_funnel_draws_its_stages_widest_first() {
        let drawn = draw_funnel(&funnel(PieLabels::Value, &[100.0, 40.0, 60.0])).expect("drawn");
        let order: Vec<_> = drawn.geometry.marks.iter().map(|m| m.index).collect();
        assert_eq!(order, [0, 1, 2], "{}", drawn.svg);
        let mut tops = Vec::new();
        for mark in &drawn.geometry.marks {
            let (x, y) = super::super::hit::anchor(&mark.shape);
            let outline = super::super::hit::outline(match &mark.shape {
                MarkShape::Sector { path } => path,
                _ => panic!("{mark:?}"),
            });
            assert!(super::super::hit::contains(&outline, x, y), "{mark:?}");
            tops.push(y);
        }
        assert!(tops[0] < tops[2] && tops[2] < tops[1], "{tops:?}");
        assert!(!drawn.svg.contains("ct-trigger") && !drawn.svg.contains("<title"));
    }

    /// A percent is a share of the first stage; a label too wide for its stage is dropped.
    #[test]
    fn a_funnel_writes_percent_of_the_first_stage() {
        let drawn = draw_funnel(&funnel(PieLabels::Percent, &[200.0, 50.0])).expect("drawn");
        let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
        assert!(labels.contains(&"100%".to_string()), "{labels:?}");
        assert!(labels.contains(&"25%".to_string()), "{labels:?}");

        let mut narrow = funnel(PieLabels::Name, &[1000.0, 1.0]);
        narrow.stages[1].name = "A very long stage name".to_string();
        let drawn = draw_funnel(&narrow).expect("drawn");
        let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
        assert_eq!(labels, ["S0"], "{}", drawn.svg);
        assert!(!drawn.svg.contains("A very long"), "{}", drawn.svg);

        let drawn = draw_funnel(&funnel(PieLabels::None, &[2.0, 1.0])).expect("drawn");
        assert!(texts(&drawn.svg).is_empty(), "{}", drawn.svg);
    }
}
