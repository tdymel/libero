use dioxus::prelude::*;

use super::{
    Series, XValues,
    engine::{self, LineSpec},
    frame::{
        Frame, Interaction, chart_unit, overlay_series, series_specs, shown_places, summary,
        table_rows, use_plot_size,
    },
    legend::use_hidden_series,
    props::chart_props,
};
use crate::{
    components::common::{Input, names_itself, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    theme::{ChartInk, Curve},
};

chart_props! {
    pub struct LineChartProps {
        /// The x axis, one value per point.
        #[props(into)]
        x: XValues,
        /// The lines, each with one value per x value.
        series: Vec<Series>,
        /// Unset, the theme's.
        #[props(default, into)]
        curve: Input<Curve>,
        /// A dot on every point. Unset, the theme's; never past its `points_limit` points.
        #[props(default)]
        points: Option<bool>,
    }
}

/// One or more series of values over a shared x axis: a trend, a comparison over time.
///
/// Screen readers get the title, a summary (range, lowest and highest of each line) and
/// the values as a table. Web and Android; enable the `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{LineChart, Series};
/// # fn app() -> Element {
/// rsx! {
///     LineChart {
///         title: "Weekly visitors",
///         x: ["Mon", "Tue", "Wed", "Thu", "Fri"],
///         series: vec![
///             Series::new("This week", [120, 132, 101, 134, 90]),
///             Series::new("Last week", [98, 110, 115, 120, 104]).dashed(),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/line-chart>
#[component]
pub fn LineChart(props: LineChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let formats = use_formats();
    let id = use_root_id(&props.attributes);
    use_name_warning(
        props.title.is_some() || names_itself(&props.attributes),
        "LineChart: no `title` - name the chart: it is how a screen reader finds it.",
    );

    let ratio = props
        .aspect_ratio
        .unwrap_or(theme.chart.aspect_ratio)
        .max(0.1);
    let size = use_plot_size();
    let (width, height) = size.get(ratio);

    let x = props.x.labels(&props.value_format, formats);
    let empty = props
        .series
        .iter()
        .all(|series| series.values.iter().all(Option::is_none));
    let points = props.points.unwrap_or(theme.chart.points) && x.len() <= theme.chart.points_limit;
    let forced = use_accessibility().forced_colors();
    let all_specs = series_specs(theme, forced, &props.series);
    let names: Vec<String> = all_specs.iter().map(|s| s.name.clone()).collect();
    let hidden = use_hidden_series(
        &props.hidden,
        &props.default_hidden,
        props.onhiddenchange,
        &names,
    );
    // Drawn series' places in `series`, to map the marks back.
    let shown = shown_places(&all_specs, &hidden.get());
    let unit = chart_unit(&props.unit, &props.series);
    let spec = LineSpec {
        width,
        height,
        x: x.clone(),
        x_numbers: props.x.numbers().to_vec(),
        series: shown.iter().map(|i| all_specs[*i].clone()).collect(),
        curve: props.curve.copied_or(theme.chart.curve),
        grid: props.grid.unwrap_or(theme.chart.grid),
        points,
        stroke_width: theme.chart.stroke_width,
        ink: ChartInk::of(theme, forced),
        format: props.value_format,
        unit: unit.clone(),
        formats,
    };
    let drawn = use_memo(use_reactive!(|spec, shown| {
        engine::draw_line(&spec).map(|drawn| drawn.with_series(&shown))
    }));

    let write = |series: &Series, value: f64| {
        let unit = series.unit.as_deref().or(unit.as_deref());
        props.value_format.format(value, unit, formats)
    };
    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => summary(&labels, labels.line, &x, &props.series, write),
    };
    let table = props.table.copied_or_default();
    let rows = Frame::rows(
        table,
        !props.loading && !empty,
        x.len() * (props.series.len() + 1),
        theme,
        || table_rows(&x, &props.series, write),
    );

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
        series: overlay_series(&all_specs, &props.series),
        hidden,
        x,
        summary,
        table,
        rows,
        plot: size,
        aspect_ratio: props.aspect_ratio,
        interaction: Interaction {
            format: props.value_format,
            unit: unit.clone(),
            formats,
            tooltip: props.tooltip.unwrap_or(theme.chart.tooltip),
            tooltip_content: props.tooltip_content,
            onpointclick: props.onpointclick,
            onactivechange: props.onactivechange,
            swap_axes: false,
        },
    }
    .render(
        &props.class,
        &props.sx,
        &props.parts,
        &props.states,
        props.attributes,
    )
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

    /// The server's HTML has the drawing, the name and what a screen reader reads instead.
    #[test]
    fn the_first_render_is_a_named_figure_with_its_summary_and_table() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    LineChart {
                        title: "Visitors",
                        x: ["Mon", "Tue", "Wed"],
                        series: vec![
                            Series::new("This week", [1.0, 3.0, 2.0]),
                            Series::new("Last week", [Some(2.0), None, Some(1.5)]),
                        ],
                    }
                }
            }
        });

        assert!(html.contains("role=\"figure\""), "{html}");
        assert!(html.contains("<svg"), "{html}");
        assert!(html.contains("aria-labelledby="), "{html}");
        assert!(html.contains("Line chart from Mon to Wed."), "{html}");
        assert!(
            html.contains("This week: 1 to 3, highest at Tue."),
            "{html}"
        );
        assert!(html.contains("<caption"), "{html}");
        assert!(
            html.contains("<th scope=\"row\">Tue</th><td>3</td><td></td>"),
            "{html}"
        );
        // Two series: a legend of toggles, and a plot the keys walk.
        assert!(html.contains("aria-label=\"Series\""), "{html}");
        assert_eq!(html.matches("aria-pressed=\"true\"").count(), 2, "{html}");
        assert!(
            html.contains("role=\"application\" tabindex=\"0\""),
            "{html}"
        );
        assert!(html.contains("aria-roledescription=\"chart\""), "{html}");
    }

    /// Todo 3335 s5: values group in the locale, a series' own unit beats the chart's,
    /// and a unit every series shares labels the axis.
    #[test]
    fn values_are_grouped_and_carry_their_series_unit() {
        let html = render(|| {
            rsx! {
                LiberoProvider { formats: &crate::localization::Formats::GERMAN,
                    LineChart {
                        title: "Revenue",
                        x: ["Q1", "Q2"],
                        unit: "EUR",
                        series: vec![
                            Series::new("Revenue", [12_500.5, 14_000.0]),
                            Series::new("Margin", [12.5, 14.0]).unit("%"),
                        ],
                    }
                }
            }
        });
        assert!(
            html.contains("<th scope=\"row\">Q1</th><td>12.500,5 EUR</td><td>12,5 %</td>"),
            "{html}"
        );

        let units = [
            Series::new("a", [1.0]).unit("kg"),
            Series::new("b", [2.0]).unit("kg"),
        ];
        assert_eq!(chart_unit(&None, &units).as_deref(), Some("kg"));
        assert_eq!(chart_unit(&Some("g".into()), &units).as_deref(), Some("g"));
        let mixed = [units[0].clone(), Series::new("c", [3.0])];
        assert_eq!(chart_unit(&None, &mixed), None);
    }

    /// Past the palette's end a line starts over in the first colour, dashed.
    #[test]
    fn the_ninth_series_repeats_the_first_colour_dashed() {
        let series: Vec<Series> = (0..9)
            .map(|i| Series::new(format!("s{i}"), [1.0]))
            .collect();
        let specs = series_specs(&crate::theme::Theme::DEFAULT, false, &series);
        assert_eq!(specs[8].color, specs[0].color);
        assert!(specs[8].dashed && !specs[7].dashed);
    }

    /// A hidden series leaves the drawing, not the legend; the last shown one stays.
    #[test]
    fn a_default_hidden_series_is_not_drawn() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    LineChart {
                        title: "Visitors",
                        x: ["Mon", "Tue"],
                        series: vec![
                            Series::new("This week", [1.0, 3.0]),
                            Series::new("Last week", [2.0, 1.0]).dashed(),
                        ],
                        default_hidden: vec!["Last week".to_string()],
                    }
                }
            }
        });

        assert!(!html.contains("stroke-dasharray=\""), "{html}");
        assert!(html.contains("aria-pressed=\"false\""), "{html}");
        assert!(
            html.contains("aria-pressed=\"true\" aria-disabled=\"true\""),
            "{html}"
        );
    }

    #[test]
    fn a_chart_without_values_says_so_and_has_no_table() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    LineChart { title: "Nothing", x: ["a"], series: vec![Series::new("A", [None::<f64>])] }
                }
            }
        });

        assert!(html.contains("No data"), "{html}");
        assert!(!html.contains("<svg") && !html.contains("<table"), "{html}");
    }

    #[test]
    fn a_loading_chart_is_busy() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    LineChart { title: "Later", loading: true, x: ["a"], series: vec![Series::new("A", [1.0])] }
                }
            }
        });

        assert!(html.contains("aria-busy=\"true\""), "{html}");
        assert!(html.contains("Loading chart"), "{html}");
        assert!(!html.contains("<table"), "{html}");
    }

    /// Past the cell cap only the summary speaks: a 10 000-cell table is no help.
    #[test]
    fn a_table_past_the_cap_is_left_out() {
        let html = render(|| {
            let x: Vec<String> = (0..1_500).map(|i| i.to_string()).collect();
            rsx! {
                LiberoProvider {
                    LineChart { title: "Many", x, series: vec![Series::new("A", (0..1_500).map(f64::from))] }
                }
            }
        });

        assert!(!html.contains("<table"), "{html}");
        assert!(html.contains("A: 0 to 1,499, highest at 1499."), "{html}");
    }
}
