use dioxus::prelude::*;

use super::{
    ChartColor,
    engine::{self, HeatmapSpec},
    export::ExportSource,
    frame::{Frame, Interaction, use_plot_size},
    legend::use_hidden_series,
    overlay::{Layout, OverlaySeries},
    props::chart_props,
};
use crate::{
    components::common::{Input, names_itself, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    localization::{ChartLabels, fill},
    theme::{ChartInk, Color, ColorShade, HexColor, LegendPosition, Theme, chart_color, chart_hex},
    utils::warn,
};

chart_props! {
    pub struct HeatmapChartProps {
        /// The columns' names, left to right.
        x: Vec<String>,
        /// The rows' names, top to bottom.
        y: Vec<String>,
        /// A row per `y`, a value per `x`. NaN, or a value missing at a row's end, is an
        /// empty cell; a row of the wrong length warns in development.
        values: Vec<Vec<f64>>,
        /// The scale from the lowest value's colour to the highest's, two or more. Unset, a
        /// light to a strong `primary`.
        #[props(default)]
        colors: Vec<ChartColor>,
        /// The scale's ends. Unset, the lowest and the highest value.
        #[props(default)]
        min: Option<f64>,
        #[props(default)]
        max: Option<f64>,
        /// Sorts the values into this many classes of the same width, one colour each,
        /// instead of a gradient.
        #[props(default)]
        steps: Option<usize>,
        /// Each value written in its cell, where it fits. Unset, true.
        #[props(default)]
        cell_labels: Option<bool>,
    }
}

/// A grid of values coloured on a scale, as visits by weekday and hour.
///
/// Never colour alone: each cell writes its value where it fits, the tooltip and the keys
/// read it, the summary lists every row. The scale and its ends are drawn under the plot;
/// `legend: "none"` leaves it out. Web and Android; enable the `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::HeatmapChart;
/// # fn app() -> Element {
/// rsx! {
///     HeatmapChart {
///         title: "Visits",
///         x: vec!["9 am".to_string(), "12 pm".to_string(), "3 pm".to_string()],
///         y: vec!["Mon".to_string(), "Tue".to_string()],
///         values: vec![vec![12.0, 30.0, 18.0], vec![8.0, 26.0, 22.0]],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/chart>
#[component]
pub fn HeatmapChart(props: HeatmapChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let formats = use_formats();
    let id = use_root_id(&props.attributes);
    use_name_warning(
        props.title.is_some() || names_itself(&props.attributes),
        "HeatmapChart: no `title` - name the chart: it is how a screen reader finds it.",
    );
    let ragged = props.values.len() != props.y.len()
        || props.values.iter().any(|row| row.len() != props.x.len());
    let shape = (props.y.len(), props.x.len());
    use_hook(move || {
        if ragged {
            warn(&format!(
                "HeatmapChart: `values` is not a grid of {} by {} - missing cells are empty, extra ones dropped.",
                shape.0, shape.1
            ));
        }
    });

    let ratio = props
        .aspect_ratio
        .unwrap_or(theme.chart.pie_aspect_ratio)
        .max(0.1);
    let size = use_plot_size();
    let (width, height) = size.get(ratio);

    let values: Vec<Vec<Option<f64>>> = (0..props.y.len())
        .map(|row| {
            (0..props.x.len())
                .map(|column| {
                    props
                        .values
                        .get(row)
                        .and_then(|values| values.get(column))
                        .copied()
                        .filter(|value| value.is_finite())
                })
                .collect()
        })
        .collect();
    let found = values.iter().flatten().flatten().copied();
    let (low, high) = found.fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), v| {
        (low.min(v), high.max(v))
    });
    let empty = low > high;
    let range = (
        props.min.filter(|v| v.is_finite()).unwrap_or(low),
        props.max.filter(|v| v.is_finite()).unwrap_or(high),
    );
    let forced = use_accessibility().forced_colors();
    let gaps = values.iter().flatten().any(Option::is_none);
    let scale = Frame::legend(&props.legend, theme, 2) != LegendPosition::None;
    let spec = HeatmapSpec {
        width,
        height,
        x: props.x.clone(),
        y: props.y.clone(),
        values,
        colors: scale_colors(theme, forced, &props.colors),
        range,
        steps: props.steps.unwrap_or(0),
        labels: props.cell_labels.unwrap_or(true),
        scale,
        no_value: gaps.then(|| labels.no_value.to_string()),
        ink: ChartInk::of(theme, forced),
        format: props.value_format,
        unit: props.unit.clone(),
        formats,
    };
    // The file keeps the normal palette under forced colours.
    let export = {
        let spec = HeatmapSpec {
            colors: scale_colors(theme, false, &props.colors),
            ink: ChartInk::of(theme, false),
            ..spec.clone()
        };
        ExportSource::of(theme, props.title.clone(), move || {
            engine::draw_heatmap(&spec)
        })
    };
    let drawn = use_memo(use_reactive!(|spec, empty| {
        (!empty).then(|| engine::draw_heatmap(&spec)).flatten()
    }));
    let hidden = use_hidden_series(&None, &[], None, &[]);

    let write = |value: f64| {
        props
            .value_format
            .format(value, props.unit.as_deref(), formats)
    };
    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => summary(&labels, &spec, (low, high), write),
    };
    let series = props
        .y
        .iter()
        .map(|name| OverlaySeries {
            name: name.clone(),
            color: "transparent".to_string(),
            dashed: false,
            symbol: None,
            line: false,
            unit: None,
            errors: Vec::new(),
            xs: Vec::new(),
            points: Vec::new(),
        })
        .collect();

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
        legend: LegendPosition::None,
        legend_label: labels.legend,
        series,
        hidden,
        x: props.x.clone(),
        summary,
        plot: size,
        aspect_ratio: Some(ratio),
        interaction: Interaction {
            format: props.value_format,
            unit: props.unit.clone(),
            formats,
            tooltip: props.tooltip.unwrap_or(theme.chart.tooltip),
            tooltip_content: props.tooltip_content,
            onpointclick: props.onpointclick,
            onactivechange: props.onactivechange,
            swap_axes: false,
            onrange: None,
            layout: Layout::Columns,
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

/// The scale's hexes: as given (two or more), else a light to a strong `primary`.
fn scale_colors(theme: &Theme, forced: bool, colors: &[ChartColor]) -> Vec<HexColor> {
    if colors.len() < 2 {
        return vec![
            chart_hex(theme, Color::Primary, ColorShade::S1),
            chart_hex(theme, Color::Primary, ColorShade::S8),
        ];
    }
    colors
        .iter()
        .map(|color| match *color {
            ChartColor::Hex(hex) => hex,
            ChartColor::Palette(color, shade) => chart_color(theme, color, shade, forced),
        })
        .collect()
}

/// Rows by columns and the values' range, the highest and lowest cell, then each row's
/// cells; a large grid points to the keys instead of its rows.
fn summary(
    labels: &ChartLabels,
    spec: &HeatmapSpec,
    (low, high): (f64, f64),
    write: impl Fn(f64) -> String,
) -> Vec<String> {
    const MOST_ROWS: usize = 12;
    const MOST_CELLS: usize = 150;
    let (rows, columns) = (spec.y.len(), spec.x.len());
    let cells: Vec<(usize, usize, f64)> = (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .filter_map(|(row, column)| Some((row, column, spec.value(row, column)?)))
        .collect();
    let point = |&(row, column, value): &(usize, usize, f64)| {
        fill(
            labels.point,
            &[
                ("name", &spec.y[row]),
                ("x", &spec.x[column]),
                ("value", &write(value)),
            ],
        )
    };
    // The first of the highest and of the lowest, reading row by row.
    let highest = cells.iter().find(|cell| cell.2 == high);
    let lowest = cells.iter().find(|cell| cell.2 == low);
    let mut sentences = vec![fill(
        labels.heatmap_range,
        &[
            ("kind", &labels.heatmap),
            ("rows", &rows.to_string()),
            ("columns", &columns.to_string()),
            ("min", &write(low)),
            ("max", &write(high)),
        ],
    )];
    if let (Some(highest), Some(lowest)) = (highest, lowest) {
        sentences.push(fill(
            labels.cell_extremes,
            &[("high", &point(highest)), ("low", &point(lowest))],
        ));
    }
    if rows > MOST_ROWS || rows * columns > MOST_CELLS {
        sentences.push(labels.cells_by_keys.to_string());
        return sentences;
    }
    sentences.extend((0..rows).map(|row| {
        let cells: Vec<String> = (0..columns)
            .map(|column| match spec.value(row, column) {
                Some(value) => fill(
                    labels.cell,
                    &[("x", &spec.x[column]), ("value", &write(value))],
                ),
                None => fill(labels.cell_empty, &[("x", &spec.x[column])]),
            })
            .collect();
        fill(
            labels.row,
            &[("name", &spec.y[row]), ("cells", &cells.join(", "))],
        )
    }));
    sentences
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

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    /// The server's HTML has a cell per value, its scale and what a screen reader reads.
    #[test]
    fn the_first_render_is_a_named_figure_with_cells_and_a_summary() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    HeatmapChart {
                        title: "Visits",
                        x: names(&["9 am", "12 pm"]),
                        y: names(&["Mon", "Tue"]),
                        values: vec![vec![12.0, 30.0], vec![8.0, f64::NAN]],
                    }
                }
            }
        });

        assert!(html.contains("role=\"figure\""), "{html}");
        assert!(html.matches("<rect").count() >= 4, "{html}");
        assert!(
            html.contains("Heatmap of 2 rows by 2 columns, values 8 to 30."),
            "{html}"
        );
        assert!(
            html.contains("Highest: Mon, 12 pm: 30. Lowest: Tue, 9 am: 8."),
            "{html}"
        );
        assert!(html.contains("Mon: 9 am 12, 12 pm 30."), "{html}");
        assert!(html.contains("Tue: 9 am 8, 12 pm no value."), "{html}");
        assert!(html.contains("No value"), "{html}");
        assert!(!html.contains("NaN"), "{html}");
    }

    /// A large grid's summary points to the keys instead of reading every row.
    #[test]
    fn a_large_heatmap_points_to_the_keys() {
        let html = render(|| {
            let x: Vec<String> = (0..24).map(|h| format!("{h}h")).collect();
            let y: Vec<String> = (0..7).map(|d| format!("D{d}")).collect();
            let values = vec![(0..24).map(f64::from).collect(); 7];
            rsx! {
                LiberoProvider {
                    HeatmapChart { title: "Week", x, y, values }
                }
            }
        });
        assert!(
            html.contains("Use the arrow keys to read each cell."),
            "{html}"
        );
        assert!(!html.contains("D0: 0h"), "{html}");
    }

    #[test]
    fn rows_of_the_wrong_length_warn() {
        let warnings = crate::utils::warnings_of(|| {
            rsx! {
                LiberoProvider {
                    HeatmapChart {
                        title: "Visits",
                        x: names(&["a", "b"]),
                        y: names(&["r"]),
                        values: vec![vec![1.0]],
                    }
                }
            }
        });
        assert_eq!(
            warnings,
            [
                "HeatmapChart: `values` is not a grid of 1 by 2 - missing cells are empty, extra ones dropped."
            ]
        );
    }

    #[test]
    fn a_heatmap_without_values_says_so() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    HeatmapChart { title: "Nothing", x: names(&["a"]), y: names(&["r"]), values: vec![vec![f64::NAN]] }
                }
            }
        });
        assert!(html.contains("No data"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }
}
