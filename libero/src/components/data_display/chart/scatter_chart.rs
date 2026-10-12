use dioxus::prelude::*;

use super::{
    ScatterSeries, Trend,
    engine::{self, Axes, ScatterSpec, ScatterSpecSeries},
    export::ExportSource,
    frame::{Frame, Interaction, axis_titles, chart_unit_of, hex, use_plot_size},
    legend::use_hidden_series,
    overlay::{Layout, OverlaySeries},
    props::chart_props,
};
use crate::{
    components::common::{Input, names_itself, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    localization::{ChartLabels, fill},
    theme::{ChartInk, PointSymbol, Theme, chart_palette},
    utils::warn,
};

chart_props! {
    pub struct ScatterChartProps {
        /// The groups of points, each in its colour and shape.
        series: Vec<ScatterSeries>,
        /// A curve fitted through each series' points, in its colour. Unset, none.
        #[props(default)]
        trend: Option<Trend>,
        /// The x axis' lowest value. Unset, a round number at or under the lowest x.
        #[props(default)]
        x_min: Option<f64>,
        /// The x axis' highest value. Unset, a round number at or over the highest x.
        #[props(default)]
        x_max: Option<f64>,
        /// The y axis' lowest value. Unset, a round number at or under the lowest y.
        #[props(default)]
        y_min: Option<f64>,
        /// The y axis' highest value. Unset, a round number at or over the highest y.
        #[props(default)]
        y_max: Option<f64>,
        /// About how many steps each axis is cut into, each 1, 2, 2.5 or 5 times a power of
        /// ten. Unset, the theme's.
        #[props(default)]
        y_ticks: Option<usize>,
        /// Under the x axis: what it measures.
        #[props(default, into)]
        x_title: Option<String>,
        /// Beside the y axis, read upward: what it measures.
        #[props(default, into)]
        y_title: Option<String>,
    }
}

/// Points by two values, x and y: how one measure goes with another. A point with a size
/// makes its series bubbles, their area by the size.
///
/// Screen readers get the title and a summary (each series' count and ranges, its trend).
/// The keys walk a series' points left to right, up and down to the nearest point of the
/// next series. `zoom` and `onrange` are a line's or bar's. Web and Android; enable the
/// `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{ScatterChart, ScatterSeries, Trend};
/// # fn app() -> Element {
/// rsx! {
///     ScatterChart {
///         title: "Height and weight",
///         x_title: "Height (cm)",
///         y_title: "Weight (kg)",
///         trend: Trend::Linear,
///         series: vec![
///             ScatterSeries::new("Girls", [(150, 42.5), (160, 51.0), (171, 60.2)]),
///             ScatterSeries::new("Boys", [(155, 48.0), (168, 61.5), (180, 72.3)]),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/chart>
#[component]
pub fn ScatterChart(props: ScatterChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let formats = use_formats();
    let id = use_root_id(&props.attributes);
    use_name_warning(
        props.title.is_some() || names_itself(&props.attributes),
        "ScatterChart: no `title` - name the chart: it is how a screen reader finds it.",
    );

    let ratio = props
        .aspect_ratio
        .unwrap_or(theme.chart.aspect_ratio)
        .max(0.1);
    let size = use_plot_size();
    let (width, height) = size.get(ratio);

    let empty = props
        .series
        .iter()
        .all(|series| series.points.iter().all(|p| p.finite().is_none()));
    let bubble = props.series.iter().any(ScatterSeries::is_bubble);
    let unfit = props.trend.is_some_and(|trend| {
        let mut points = props.series.iter().flat_map(|s| &s.points);
        points.any(|p| p.finite().is_some_and(|p| !fits(trend, &p)))
    });
    use_hook(move || {
        if unfit {
            warn(
                "ScatterChart: an exponential trend skips the points at or under y 0, a logarithmic one those at or left of x 0.",
            );
        }
    });
    let forced = use_accessibility().forced_colors();
    let all_specs = specs(theme, forced, &props.series, bubble);
    let names: Vec<String> = all_specs.iter().map(|s| s.name.clone()).collect();
    let hidden = use_hidden_series(
        &props.hidden,
        &props.default_hidden,
        props.onhiddenchange,
        &names,
    );
    let shown: Vec<usize> = (0..all_specs.len())
        .filter(|i| !hidden.get().contains(&all_specs[*i].name))
        .collect();
    let unit = chart_unit_of(&props.unit, props.series.iter().map(|s| &s.unit));
    let spec = ScatterSpec {
        width,
        height,
        series: shown.iter().map(|i| all_specs[*i].clone()).collect(),
        bubble,
        trend: props.trend,
        axes: Axes {
            min: props.y_min,
            max: props.y_max,
            ticks: props.y_ticks.unwrap_or(theme.chart.y_ticks),
            x_title: props.x_title.clone(),
            y_title: props.y_title.clone(),
            y2_title: None,
            compact: labels.compact,
        },
        x_min: props.x_min,
        x_max: props.x_max,
        grid: props.grid.unwrap_or(theme.chart.grid),
        radius: theme.chart.stroke_width + 3.0,
        ink: ChartInk::of(theme, forced),
        format: props.value_format,
        unit: unit.clone(),
        formats,
    };
    // The file keeps the normal palette under forced colours.
    let export = {
        let specs = specs(theme, false, &props.series, bubble);
        let spec = ScatterSpec {
            series: shown.iter().map(|i| specs[*i].clone()).collect(),
            ink: ChartInk::of(theme, false),
            ..spec.clone()
        };
        ExportSource::of(theme, props.title.clone(), move || {
            engine::draw_scatter(&spec)
        })
    };
    let drawn = use_memo(use_reactive!(|spec, shown| {
        engine::draw_scatter(&spec).map(|drawn| drawn.with_points(&shown))
    }));

    let format = props.value_format;
    let write_x = |value: f64| format.format(value, None, formats);
    let write_y = |series: &ScatterSeries, value: f64| {
        let unit = series.unit.as_deref().or(unit.as_deref());
        format.format(value, unit, formats)
    };
    let overlay: Vec<OverlaySeries> = props
        .series
        .iter()
        .zip(&all_specs)
        .map(|(series, spec)| {
            let written = series.points.iter().map(|point| match point.finite() {
                Some((x, y, size)) => {
                    let (x, y) = (write_x(x), write_y(series, y));
                    let value = match size {
                        Some(size) => fill(
                            labels.bubble_value,
                            &[("x", &x), ("y", &y), ("size", &write_x(size))],
                        ),
                        None => fill(labels.scatter_value, &[("x", &x), ("y", &y)]),
                    };
                    (x, value)
                }
                None => Default::default(),
            });
            let (xs, points) = written.unzip();
            OverlaySeries {
                name: spec.name.clone(),
                color: spec.color.to_string(),
                dashed: false,
                symbol: Some(spec.symbol),
                line: false,
                unit: series.unit.clone(),
                errors: Vec::new(),
                xs,
                points,
            }
        })
        .collect();

    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => {
            let mut sentences = summary(&labels, &props.series, props.trend, write_x, write_y);
            sentences.extend(axis_titles(&labels, &props.x_title, &props.y_title, &None));
            sentences
        }
    };

    Frame {
        id,
        labels,
        title: props.title.clone(),
        description: props.description.clone(),
        loading: props.loading,
        empty,
        empty_slot: props.empty.clone(),
        drawn,
        center: None,
        legend: Frame::legend(&props.legend, theme, props.series.len()),
        legend_label: labels.legend,
        series: overlay,
        hidden,
        x: Vec::new(),
        summary,
        plot: size,
        aspect_ratio: props.aspect_ratio,
        interaction: Interaction {
            format,
            unit,
            formats,
            tooltip: props.tooltip.unwrap_or(theme.chart.tooltip),
            tooltip_content: props.tooltip_content,
            onpointclick: props.onpointclick,
            onactivechange: props.onactivechange,
            swap_axes: false,
            layout: Layout::Free,
            onrange: None,
        },
        export,
        handle: props.handle,
        zoom: None,
    }
    .render(
        &props.class,
        &props.sx,
        &props.parts,
        &props.states,
        props.attributes,
    )
}

/// Each series' finite points, colour and shape: its own, else by its place; a bubble's a
/// circle.
fn specs(
    theme: &Theme,
    forced: bool,
    series: &[ScatterSeries],
    bubble: bool,
) -> Vec<ScatterSpecSeries> {
    const SHAPES: [PointSymbol; 4] = [
        PointSymbol::Circle,
        PointSymbol::Triangle,
        PointSymbol::Square,
        PointSymbol::Diamond,
    ];
    let palette = chart_palette(theme, forced);
    series
        .iter()
        .enumerate()
        .map(|(i, series)| ScatterSpecSeries {
            name: series.name.clone(),
            points: series
                .points
                .iter()
                .enumerate()
                .filter_map(|(i, point)| point.finite().map(|(x, y, size)| (i, x, y, size)))
                .collect(),
            color: hex(theme, forced, &palette, i, series.color),
            symbol: series.symbol.unwrap_or(match bubble {
                true => PointSymbol::Circle,
                false => SHAPES[i % SHAPES.len()],
            }),
        })
        .collect()
}

/// The x range over every point, then each series' count and ranges, its sizes and trend.
fn summary(
    labels: &ChartLabels,
    series: &[ScatterSeries],
    trend: Option<Trend>,
    write_x: impl Fn(f64) -> String,
    write_y: impl Fn(&ScatterSeries, f64) -> String,
) -> Vec<String> {
    let range = |values: &mut dyn Iterator<Item = f64>| {
        values.fold(None, |range: Option<(f64, f64)>, v| {
            Some(range.map_or((v, v), |(low, high)| (low.min(v), high.max(v))))
        })
    };
    let finite = |series: &ScatterSeries| -> Vec<(f64, f64, Option<f64>)> {
        series.points.iter().filter_map(|p| p.finite()).collect()
    };
    let mut sentences = Vec::new();
    let all: Vec<_> = series.iter().flat_map(finite).collect();
    if let Some((low, high)) = range(&mut all.iter().map(|p| p.0)) {
        let kind = labels.scatter;
        sentences.push(match low == high {
            true => fill(
                labels.range_one,
                &[("kind", &kind), ("first", &write_x(low))],
            ),
            false => fill(
                labels.range,
                &[
                    ("kind", &kind),
                    ("first", &write_x(low)),
                    ("last", &write_x(high)),
                ],
            ),
        });
    }
    for one in series {
        let points = finite(one);
        let name = one.name.as_str();
        let (Some(x), Some(y)) = (
            range(&mut points.iter().map(|p| p.0)),
            range(&mut points.iter().map(|p| p.1)),
        ) else {
            sentences.push(fill(labels.series_empty, &[("name", &name)]));
            continue;
        };
        sentences.push(fill(
            labels.scatter_series,
            &[
                ("name", &name),
                ("count", &points.len().to_string()),
                ("xmin", &write_x(x.0)),
                ("xmax", &write_x(x.1)),
                ("min", &write_y(one, y.0)),
                ("max", &write_y(one, y.1)),
            ],
        ));
        if let Some((low, high)) = range(&mut points.iter().filter_map(|p| p.2)) {
            sentences.push(fill(
                labels.bubble_sizes,
                &[
                    ("name", &name),
                    ("min", &write_x(low)),
                    ("max", &write_x(high)),
                ],
            ));
        }
        // Its kind, and which way its least-squares line runs over the points it fits.
        let fitted = trend.and_then(|trend| {
            let fits: Vec<_> = points.iter().copied().filter(|p| fits(trend, p)).collect();
            Some((trend, slope(&fits)?))
        });
        if let Some((trend, slope)) = fitted {
            let kind = match trend {
                Trend::Linear => labels.trends[0],
                Trend::Exponential => labels.trends[1],
                Trend::Logarithmic => labels.trends[2],
                Trend::Polynomial(_) => labels.trends[3],
            };
            let direction = match slope {
                s if s > 0.0 => labels.directions[0],
                s if s < 0.0 => labels.directions[1],
                _ => labels.directions[2],
            };
            sentences.push(fill(
                labels.trend,
                &[("name", &name), ("kind", &kind), ("direction", &direction)],
            ));
        }
    }
    sentences
}

/// An exponential trend fits only the points above 0, a logarithmic one those right of 0.
fn fits(trend: Trend, point: &(f64, f64, Option<f64>)) -> bool {
    match trend {
        Trend::Exponential => point.1 > 0.0,
        Trend::Logarithmic => point.0 > 0.0,
        _ => true,
    }
}

/// The least-squares line's slope through the points; `None` with no spread in x.
fn slope(points: &[(f64, f64, Option<f64>)]) -> Option<f64> {
    let n = points.len() as f64;
    let (mx, my) = points
        .iter()
        .fold((0.0, 0.0), |(sx, sy), p| (sx + p.0 / n, sy + p.1 / n));
    let (cov, var) = points.iter().fold((0.0, 0.0), |(c, v), p| {
        (c + (p.0 - mx) * (p.1 - my), v + (p.0 - mx).powi(2))
    });
    (var > 0.0).then(|| cov / var)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LiberoProvider;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    /// A named figure, its drawing, a summary with each series' ranges and trend, and a
    /// legend whose keys show each series' shape.
    #[test]
    fn the_first_render_is_a_named_figure_with_its_summary() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    ScatterChart {
                        title: "Height and weight",
                        trend: Trend::Linear,
                        unit: "kg",
                        series: vec![
                            ScatterSeries::new("Girls", [(150.0, 42.5), (160.0, 51.0), (f64::NAN, 1.0)]),
                            ScatterSeries::new("Boys", [(155, 70), (168, 61), (180, 52)]),
                        ],
                    }
                }
            }
        });

        assert!(html.contains("role=\"figure\""), "{html}");
        assert!(html.contains("<svg"), "{html}");
        assert!(html.contains("Scatter chart from 150 to 180."), "{html}");
        assert!(
            html.contains("Girls: 2 points, x from 150 to 160, y from 42.5 kg to 51 kg."),
            "{html}"
        );
        assert!(html.contains("Girls: linear trend, rising."), "{html}");
        assert!(html.contains("Boys: linear trend, falling."), "{html}");
        assert!(html.contains("role=\"application\""), "{html}");
        assert_eq!(html.matches("aria-pressed=\"true\"").count(), 2, "{html}");
    }

    /// Sized points are bubbles; their sizes are in the summary.
    #[test]
    fn sizes_make_bubbles() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    ScatterChart {
                        title: "Cities",
                        series: vec![ScatterSeries::new("Cities", [(1, 2, 300), (2, 4, 1200)])],
                    }
                }
            }
        });
        assert!(html.contains("Cities: sizes from 300 to 1,200."), "{html}");
        assert!(!html.contains("trend"), "{html}");
    }

    /// No finite point: the empty state, no drawing.
    #[test]
    fn no_finite_point_is_empty() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    ScatterChart {
                        title: "Nothing",
                        series: vec![ScatterSeries::new("A", [(f64::NAN, 1.0)])],
                    }
                }
            }
        });
        assert!(html.contains("No data"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }
}
