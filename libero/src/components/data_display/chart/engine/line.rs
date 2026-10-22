//! A line chart's spec and drawing, its series' symbols, bands and curves.

use super::*;

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

/// A point's marker, sized so every shape covers about a circle's area.
pub(super) fn symbol(points: bool, stroke_width: f32, series: &SeriesSpec) -> Symbol {
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
pub(super) fn band(spec: &LineSpec, series: usize) -> Option<charts_rs::SeriesBand> {
    let ranges = spec.errors.get(series).filter(|_| !spec.stacked)?;
    band_of(ranges, &spec.series.get(series)?.values)
}

pub(super) fn band_of(
    ranges: &[Option<(f64, f64)>],
    values: &[Option<f64>],
) -> Option<charts_rs::SeriesBand> {
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

pub(super) fn bend(drawn: &mut charts_rs::Series, curve: Curve) {
    match curve {
        Curve::Linear => {}
        Curve::Smooth => drawn.smooth = Some(true),
        Curve::Step => drawn.step = Some(LineStep::End),
    }
}

/// A right y axis like the left one, its labels behind [`AXIS2_MARK`].
pub(super) fn second_axis(y: &mut Vec<charts_rs::YAxisConfig>, unit: &Option<String>) {
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
    style_chart!(chart, spec, unit_suffix(spec.unit.as_deref()));
    chart.series.stroke_width = spec.stroke_width;
    chart.series.fill = spec.area.is_some();
    chart.series.fill_opacity = spec.area;
    if !spec.x_numbers.is_empty() {
        chart.x_axis.kind = AxisType::Value;
        chart.x_axis.values = spec.x_numbers.clone();
        chart.x_axis.formatter = Some(format!("{X_MARK}{{c}}"));
        // Ends on round values, so the last x is not on the plot's edge.
        let mut span = charts_rs::YAxisConfig {
            split_number: (spec.width / 60.0) as usize,
            ..Default::default()
        };
        span.split_number = span.split_number.clamp(2, 10);
        let xs = spec.x_numbers.iter().map(|x| *x as f32).collect();
        let column = [charts_rs::Series::new(String::new(), xs)];
        round_axis(&column, 0, &mut span, None, false);
        chart.x_axis.min = span.min.map(f64::from);
        chart.x_axis.max = span.max.map(f64::from);
    } else {
        // Every label drawn, so `fit_x_labels` finds each.
        chart.x_axis.label_overflow = AxisLabelOverflow::Ellipsis;
    }
    if spec.series.iter().any(|series| series.extras.secondary) {
        second_axis(&mut chart.y_axis_configs, &spec.y2_unit);
    }
    spec.axes
        .style(&mut chart.base, &mut chart.y_axis_configs, false);
    round_axes(&chart.base, &mut chart.y_axis_configs, false);
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
        let exact = exact_axes(&svg, &chart.y_axis_configs, LabelEdge::Side);
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
    let svg = exact_axes(&pin_to_axis(&svg), &chart.y_axis_configs, LabelEdge::Side);
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
    let pins = pins_of(&spec.series, spec.width, &marks);
    let lines = series_lines(&marks);
    let svg = label_mark_lines(&svg, &texts, &spec.ink, plot.left, &pins, &[], &lines);
    let svg = pin_mark_points(&svg, &pins, &spec.ink);
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
