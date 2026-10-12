//! Props to `charts-rs` and back: it draws the static SVG, we harvest where each mark
//! landed and strip its own tooltips, so the overlay and keyboard layers can be ours.

use charts_rs::{
    AxisType, Color as InkColor, DEFAULT_FONT_FAMILY, LineStep, MarkLineCategory, Position, Symbol,
};

use super::data::{ValueFormat, ungrouped};
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
    }
}

/// Every series' mark lines inside the value axis' ends, where unset the data's extent and
/// 0, which the axis always spans; in the order the renderer draws them.
fn kept_mark_lines<'a>(series: &'a [SeriesSpec], axes: &Axes) -> Vec<Vec<&'a (f64, String)>> {
    let values = series
        .iter()
        .flat_map(|s| s.values.iter().flatten().copied());
    let low = axes.min.unwrap_or(values.clone().fold(0.0, f64::min));
    let high = axes.max.unwrap_or(values.fold(0.0, f64::max));
    series
        .iter()
        .map(|series| {
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
fn label_mark_lines(svg: &str, texts: &[&str], ink: &ChartInk) -> String {
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
        let kept: String = attributes(tag)
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
fn pin_mark_points(svg: &str, spec: &LineSpec, marks: &[Mark]) -> String {
    const HEIGHT: f32 = 20.0;
    const GAP: f32 = 6.0;
    const SIZE: f32 = 12.0;
    let mut pins = String::new();
    for (s, series) in spec.series.iter().enumerate() {
        for (index, text) in &series.extras.mark_points {
            let place = marks.iter().find(|m| m.series == s && m.index == *index);
            let Some(MarkShape::Point { x, y }) = place.map(|mark| &mark.shape) else {
                continue;
            };
            let width = charts_rs::measure_text_width_family(DEFAULT_FONT_FAMILY, SIZE, text)
                .map_or(0.0, |text| text.width())
                + HEIGHT * 0.6;
            let left = (x - width / 2.0).clamp(1.0, (spec.width - width - 1.0).max(1.0));
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
                spec.ink.on(series.color),
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
fn symbol(spec: &LineSpec, series: &SeriesSpec) -> Symbol {
    let (r, fill) = (spec.stroke_width + 1.0, Some(ink(series.color)));
    match (spec.points, series.extras.symbol) {
        (false, _) => Symbol::None,
        (true, PointSymbol::Circle) => Symbol::Circle(r, fill),
        (true, PointSymbol::Square) => Symbol::Rect(r * 0.9, fill),
        (true, PointSymbol::Triangle) => Symbol::Triangle(r * 1.35, fill),
        (true, PointSymbol::Diamond) => Symbol::Diamond(r * 1.25, fill),
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
            drawn.symbol = Some(symbol(spec, series));
            drawn.stack = spec.stacked.then(|| "stack".to_string());
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
    spec.axes
        .style(&mut chart.base, &mut chart.y_axis_configs, false);
    let write = |label| {
        spec.format
            .format(label, spec.unit.as_deref(), spec.formats)
    };
    let svg = fitted(&mut chart, spec.width, spec.height, |chart| {
        let mut svg = chart.svg().ok()?;
        if let Some(width) = y_width(&svg, write) {
            if let Some(y) = chart.y_axis_configs.first_mut() {
                y.width = Some(width);
            }
            svg = chart.svg().ok()?;
        }
        Some(svg)
    })?;

    let expected: Vec<_> = expected_marks(&spec.series);
    let (svg, triggers) = sanitize(&pin_to_axis(&svg), write);
    let x_formats = ungrouped(spec.formats);
    let svg = relabel(&svg, X_MARK, |label| {
        spec.format.format(label, None, &x_formats)
    });
    let marks = match_marks(&expected, &triggers);
    let texts: Vec<&str> = marked
        .iter()
        .flatten()
        .map(|(_, text)| text.as_str())
        .collect();
    let svg = label_mark_lines(&svg, &texts, &spec.ink);
    let plot = Plot::of(&svg, spec.width, spec.height);
    let svg = pin_mark_points(&svg, spec, &marks);
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot,
            marks,
            hole: 0.0,
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
    pub(crate) axes: Axes,
    /// The value labels in the middle of their bars rather than past their ends.
    pub(crate) labels_inside: bool,
    /// Per series, a bar's own colour per x value; `None`, the series' colour.
    pub(crate) bar_colors: Vec<Vec<Option<HexColor>>>,
    /// Per series, each value's error range; drawn on upright grouped bars only.
    pub(crate) errors: Vec<Vec<Option<(f64, f64)>>>,
    /// The share of a category's band its bars fill; `None`, the renderer's 5 px margins.
    pub(crate) bar_width: Option<f32>,
    /// Between two bars of a group, in pixels.
    pub(crate) bar_gap: f32,
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
        $spec.axes.style(
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
    // Upright and side by side, each bar stands at its own value.
    let marked = match !spec.horizontal && spec.layout == BarLayout::Grouped {
        true => kept_mark_lines(&ordered, &spec.axes),
        false => Vec::new(),
    };
    let bar_color = |series: usize, index: usize| {
        spec.bar_colors
            .get(series)
            .and_then(|colors| colors.get(index).copied().flatten())
            .unwrap_or(spec.series[series].color)
    };
    // A range on a stacked or lying bar would not read against its own segment.
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
            drawn.label_show = spec.value_labels;
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
    let svg = match spec.horizontal {
        true => {
            let mut chart = charts_rs::HorizontalBarChart::new(series, x);
            style_bars!(chart, spec, unit);
            chart.stack_percent = percent;
            fitted(&mut chart, spec.width, spec.height, |chart| {
                Some(pin_to_ticks(&chart.svg().ok()?, spec.width, write))
            })?
        }
        false => {
            let mut chart = charts_rs::BarChart::new(series, x);
            style_bars!(chart, spec, unit);
            chart.stack_percent = percent;
            fitted(&mut chart, spec.width, spec.height, |chart| {
                let mut svg = chart.svg().ok()?;
                if let Some(width) = y_width(&svg, write) {
                    if let Some(y) = chart.y_axis_configs.first_mut() {
                        y.width = Some(width);
                    }
                    svg = chart.svg().ok()?;
                }
                Some(pin_to_axis(&svg))
            })?
        }
    };

    let (svg, triggers) = inert(&svg, write, "");
    let (svg, triggers) = match spec.bar_width.is_some() || spec.bar_gap != RENDERER_GAP {
        true => relayout(&svg, triggers, &expected_marks(&ordered), spec),
        false => (svg, triggers),
    };
    let negative: Vec<bool> = expected_marks(&ordered)
        .iter()
        .map(|&(_, _, value, _)| value < 0.0)
        .collect();
    let svg = round_free_ends(&svg, &triggers, &negative, spec.horizontal, spec.radius);
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
    let svg = ink_bar_labels(&svg, &marks, |mark| {
        spec.ink.on(bar_color(mark.series, mark.index))
    });
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
    let svg = label_mark_lines(&svg, &texts, &spec.ink);
    Some(Drawn {
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot::of(&svg, spec.width, spec.height),
            marks,
            hole: 0.0,
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
    let slots = if grouped { spec.series.len().max(1) } else { 1 };
    let unit = span / count as f32;
    let group = match spec.bar_width {
        Some(share) => unit * share.clamp(0.1, 1.0),
        None => unit - 2.0 * RENDERER_MARGIN,
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
            let slot = if grouped { series } else { 0 };
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
    /// The degrees the slices share.
    fn span(&self) -> f32 {
        match self.end_angle {
            Some(end) if end.is_finite() && end > self.start_angle => {
                (end - self.start_angle).min(360.0)
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
    // What `center` fills: the hole, or the half of it a ring of half a turn or less bends round.
    let (inner, inner_radius) = match span <= 180.0 {
        true => (
            on_circle(centre, hole / 2.0, spec.start_angle + span / 2.0),
            hole / 2.0,
        ),
        false => (centre, hole),
    };

    // A line of the page between two slices of a close colour.
    let edge = format!(
        " stroke=\"{}\" stroke-width=\"{}\"",
        spec.ink.surface,
        spec.slice_gap.max(0.0)
    );
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
        },
    })
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
        // The renderer writes a label on a slice from its left end, centred there.
        let middle = number("x").zip(number("y")).map(|(x, y)| {
            let size = number("font-size").unwrap_or(14.0);
            (
                x + text_width(&attributes, label.trim()) / 2.0,
                y - size * 0.35,
            )
        });
        let hex = middle
            .and_then(|(x, y)| {
                outlines
                    .iter()
                    .find(|(_, outline)| super::hit::contains(outline, x, y))
            })
            .map_or(off, |(slice, _)| ink(*slice));
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

    fn line(series: Vec<SeriesSpec>, x: &[&str]) -> LineSpec {
        LineSpec {
            width: 320.0,
            height: 180.0,
            x: x.iter().map(|x| x.to_string()).collect(),
            x_numbers: Vec::new(),
            series,
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

    /// Upright grouped bars carry their error ranges, the axis reaching the highest end;
    /// a stack carries none.
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

        spec.layout = BarLayout::Stacked;
        let stacked = draw_bar(&spec).expect("drawn").svg;
        assert!(!stacked.contains("\n10\n"), "{stacked}");
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
            labels_inside: false,
            min_label_angle: 0.0,
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
