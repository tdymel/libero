//! A scatter chart's spec and drawing, its points' symbols and trend lines.

use super::*;

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
pub(super) fn scatter_symbol(symbol: PointSymbol, r: f32) -> Symbol {
    match symbol {
        PointSymbol::Circle => Symbol::Circle(r, None),
        PointSymbol::Square => Symbol::Rect(r * 0.9, None),
        PointSymbol::Triangle => Symbol::Triangle(r * 1.35, None),
        PointSymbol::Diamond => Symbol::Diamond(r * 1.25, None),
    }
}

/// The scatter's points in the renderer's order: bubbles largest first, as it draws them.
pub(super) fn scatter_marks(spec: &ScatterSpec) -> Vec<(usize, usize, f64, &str)> {
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
        let svg = exact_axes(svg, &chart.y_axis_configs, LabelEdge::Side);
        exact_ticks(&svg, X_MARK, &chart.x_axis_config, LabelEdge::Bottom)
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
    let (svg, triggers) = inert(&svg, write(&svg, AXIS_MARK), "", CircleTriggers::Kept);
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
pub(super) fn trends_on_top(svg: &str) -> String {
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
