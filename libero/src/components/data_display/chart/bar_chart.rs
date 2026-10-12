use dioxus::prelude::*;

use super::{
    Series, XValues,
    engine::{self, BarSpec, SeriesSpec},
    frame::{
        Frame, Interaction, chart_unit, overlay_series, series_specs, shown_places, summary,
        use_plot_size,
    },
    legend::use_hidden_series,
    props::chart_props,
};
use crate::{
    components::common::{Input, Orientation, names_itself, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    theme::{BarLayout, ChartInk},
    tokens::Size,
};

chart_props! {
    pub struct BarChartProps {
        /// The categories, one per bar or group of bars.
        #[props(into)]
        x: XValues,
        /// The bars' series, each with one value per x value.
        series: Vec<Series>,
        /// Side by side, stacked, or stacked to 100%. Unset, the theme's.
        #[props(default, into)]
        layout: Input<BarLayout>,
        /// `"horizontal"` lays the categories down the side, top to bottom. Unset, vertical.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Corner radius of every bar. Unset, the theme's.
        #[props(default, into)]
        radius: Input<Size>,
        /// Each bar's value written on it. Unset, the theme's.
        #[props(default)]
        value_labels: Option<bool>,
    }
}

/// Values per category as bars: compare amounts side by side, or their parts stacked.
///
/// Screen readers get the title and a summary (range, lowest and highest of each series).
/// Web and Android; enable the `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{BarChart, Series};
/// # fn app() -> Element {
/// rsx! {
///     BarChart {
///         title: "Sales per quarter",
///         x: ["Q1", "Q2", "Q3", "Q4"],
///         series: vec![
///             Series::new("2025", [120, 132, 101, 134]),
///             Series::new("2026", [140, 128, 150, 162]),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/chart>
#[component]
pub fn BarChart(props: BarChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let formats = use_formats();
    let id = use_root_id(&props.attributes);
    use_name_warning(
        props.title.is_some() || names_itself(&props.attributes),
        "BarChart: no `title` - name the chart: it is how a screen reader finds it.",
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
    let forced = use_accessibility().forced_colors();
    // Bars are never dashed, so neither are their legend keys.
    let all_specs: Vec<SeriesSpec> = series_specs(theme, forced, &props.series)
        .into_iter()
        .map(|spec| SeriesSpec {
            dashed: false,
            ..spec
        })
        .collect();
    let names: Vec<String> = all_specs.iter().map(|s| s.name.clone()).collect();
    let hidden = use_hidden_series(
        &props.hidden,
        &props.default_hidden,
        props.onhiddenchange,
        &names,
    );
    let shown = shown_places(&all_specs, &hidden.get());
    let horizontal = props.orientation.copied_or(Orientation::Vertical) == Orientation::Horizontal;
    let unit = chart_unit(&props.unit, &props.series);
    let spec = BarSpec {
        width,
        height,
        x: x.clone(),
        series: shown.iter().map(|i| all_specs[*i].clone()).collect(),
        layout: props.layout.copied_or(theme.chart.bar_layout),
        horizontal,
        radius: f32::from(
            theme
                .radius
                .get(props.radius.copied_or(theme.chart.bar_radius)),
        ),
        value_labels: props.value_labels.unwrap_or(theme.chart.value_labels),
        grid: props.grid.unwrap_or(theme.chart.grid),
        ink: ChartInk::of(theme, forced),
        format: props.value_format,
        unit: unit.clone(),
        formats,
    };
    let drawn = use_memo(use_reactive!(|spec, shown| {
        engine::draw_bar(&spec).map(|drawn| drawn.with_series(&shown))
    }));

    let write = |series: &Series, value: f64| {
        let unit = series.unit.as_deref().or(unit.as_deref());
        props.value_format.format(value, unit, formats)
    };
    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => summary(&labels, labels.bar, &x, &props.series, write),
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
        series: overlay_series(&all_specs, &props.series),
        hidden,
        x,
        summary,
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
            swap_axes: horizontal,
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

    /// The server's HTML has the bars, the name and what a screen reader reads instead.
    #[test]
    fn the_first_render_is_a_named_figure_with_bars_and_a_summary() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Sales",
                        x: ["Q1", "Q2"],
                        series: vec![Series::new("2025", [3.0, 5.0]), Series::new("2026", [Some(4.0), None])],
                    }
                }
            }
        });

        assert!(html.contains("role=\"figure\""), "{html}");
        assert_eq!(html.matches("<rect").count(), 3, "{html}");
        assert!(html.contains("Bar chart from Q1 to Q2."), "{html}");
        assert!(html.contains("2025: 3 to 5, highest at Q2."), "{html}");
        assert!(!html.contains("<table"), "{html}");
    }

    /// A hidden series leaves the bars, not the legend or the summary; the plot takes keys.
    #[test]
    fn a_default_hidden_series_is_not_drawn_and_the_plot_is_walkable() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Sales",
                        x: ["Q1", "Q2"],
                        series: vec![Series::new("2025", [3.0, 5.0]), Series::new("2026", [4.0, 6.0])],
                        default_hidden: vec!["2026".to_string()],
                    }
                }
            }
        });

        assert_eq!(html.matches("<rect").count(), 2, "{html}");
        assert!(html.contains("aria-label=\"Series\""), "{html}");
        assert!(html.contains("aria-pressed=\"false\""), "{html}");
        assert!(
            html.contains("aria-pressed=\"true\" aria-disabled=\"true\""),
            "{html}"
        );
        assert!(html.contains("2026: 4 to 6, highest at Q2."), "{html}");
        assert!(
            html.contains("role=\"application\" tabindex=\"0\""),
            "{html}"
        );
    }

    /// Stacked, both series' bars are drawn and the summary keeps the values, not shares.
    #[test]
    fn a_percent_chart_summarises_its_values() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Shares",
                        layout: "percent",
                        orientation: "horizontal",
                        value_labels: true,
                        x: ["a", "b"],
                        series: vec![Series::new("A", [1.0, 3.0]), Series::new("B", [3.0, 1.0])],
                    }
                }
            }
        });

        // A stack's bars are paths, rounded at its outer ends only.
        let drawing = html
            .split("data-chart-drawing")
            .nth(1)
            .and_then(|rest| rest.split("</svg>").next())
            .unwrap_or_default();
        assert_eq!(drawing.matches("<path").count(), 4, "{html}");
        assert!(html.contains("75%"), "{html}");
        assert!(html.contains("A: 1 to 3, highest at b."), "{html}");
    }

    #[test]
    fn a_bar_chart_without_values_says_so() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart { title: "Nothing", x: ["a"], series: vec![Series::new("A", [None::<f64>])] }
                }
            }
        });

        assert!(html.contains("No data"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }
}
