use dioxus::prelude::*;

use super::{
    ChartRange, Series, XValues,
    engine::{self, Axes, BarSpec, SeriesExtras, SeriesSpec},
    export::ExportSource,
    frame::{
        Frame, Interaction, axis_titles, band_summary, chart_unit, hex, mark_line_specs,
        mark_point_specs, overlay_series, series_specs, shown_places, summary, use_plot_size,
        with_extras,
    },
    legend::use_hidden_series,
    overlay::Layout,
    props::chart_props,
    zoom::{cut, use_zoom, windowed},
};
use crate::{
    components::common::{Input, Orientation, names_itself, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    localization::fill,
    theme::{BarLabelPosition, BarLayout, ChartInk, chart_palette},
    tokens::Size,
    utils::warn,
};

chart_props! {
    pub struct BarChartProps {
        /// The categories, one per bar or group of bars.
        #[props(into)]
        x: XValues,
        /// The bars' series, each with one value per x value. Upright, a
        /// [`SeriesKind::Line`](super::SeriesKind) one is drawn as a line over the bars.
        series: Vec<Series>,
        /// Side by side, stacked, or stacked to 100%. Unset, the theme's.
        #[props(default, into)]
        layout: Input<BarLayout>,
        /// `"horizontal"` lays the categories down the side, top to bottom. Unset, vertical.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Corner radius of a bar's free end, the one away from its axis. Unset, the theme's.
        #[props(default, into)]
        radius: Input<Size>,
        /// Each bar's value written on it. Unset, the theme's.
        #[props(default)]
        value_labels: Option<bool>,
        /// The value axis' lowest value. Unset, 0 or a round number at or under the lowest
        /// value. Ignored by `percent`, which runs 0 to 100%.
        #[props(default)]
        y_min: Option<f64>,
        /// The value axis' highest value. Unset, a round number at or over the highest value.
        /// Ignored by `percent`.
        #[props(default)]
        y_max: Option<f64>,
        /// About how many steps the value axis is cut into, each 1, 2, 2.5 or 5 times a power of
        /// ten: one more or fewer may fit better. With `y_min` and `y_max` set, exactly as many:
        /// 4 writes 5 labels. Unset, the theme's.
        #[props(default)]
        y_ticks: Option<usize>,
        /// Along the categories: what they are. Under them, or beside them when `horizontal`.
        #[props(default, into)]
        x_title: Option<String>,
        /// Along the values: what they measure. Beside them, or under them when `horizontal`.
        #[props(default, into)]
        y_title: Option<String>,
        /// Beside the right y axis, read downward: what its [`Series::secondary_axis`] series
        /// measure. An upright chart only.
        #[props(default, into)]
        y2_title: Option<String>,
        /// `"inside"` the bar or `"outside"` past its end; a stack writes inside. Unset, the
        /// theme's, else chart-wide: inside on a `horizontal` chart or once any shown series
        /// has error ranges, past the end otherwise.
        #[props(default, into)]
        value_label_position: Input<BarLabelPosition>,
        /// The share of a category's band its bars fill, clamped to `0.1..=1`. Unset, the
        /// theme's.
        #[props(default)]
        bar_width: Option<f32>,
        /// Between two bars of a group, in pixels. Unset, the theme's.
        #[props(default)]
        bar_gap: Option<f32>,
        /// Zoom and pan along the categories: Ctrl or Cmd with the wheel, a pinch, a drag
        /// (Shift or Alt with it once `onrange` is set), the plus, minus and 0 keys, and a
        /// reset button. Unset, the theme's.
        #[props(default)]
        zoom: Option<bool>,
        /// Set, a drag or Shift with an arrow key selects a band of categories; `None`
        /// when Escape clears it. With `zoom`, Shift or Alt with a drag pans.
        #[props(default)]
        onrange: Option<EventHandler<Option<ChartRange>>>,
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
    let zoom = use_zoom(x.len(), props.zoom.unwrap_or(theme.chart.zoom));
    let window = zoom.map_or((0, x.len()), |zoom| zoom.range());
    let empty = props
        .series
        .iter()
        .all(|series| series.values.iter().all(Option::is_none));
    let forced = use_accessibility().forced_colors();
    let horizontal = props.orientation.copied_or(Orientation::Vertical) == Orientation::Horizontal;
    let layout = props.layout.copied_or(theme.chart.bar_layout);
    // Bars are never dashed, so neither are their legend keys. A lying chart draws its
    // lines as bars; a percent one measures them on the right axis, off the shares, and
    // keeps its bars on the left. A lying chart has one value axis.
    let percent = layout == BarLayout::Percent;
    let solid = |forced: bool| -> Vec<SeriesSpec> {
        series_specs(theme, forced, &props.series)
            .into_iter()
            .zip(&props.series)
            .map(|(spec, series)| {
                let line = series.is_line() && !horizontal;
                let right = match line {
                    true => spec.extras.secondary || percent,
                    false => spec.extras.secondary && !percent,
                };
                SeriesSpec {
                    dashed: spec.dashed && line,
                    extras: SeriesExtras {
                        line,
                        secondary: right && !horizontal,
                        ..spec.extras
                    },
                    ..spec
                }
            })
            .collect()
    };
    let lying_lines = horizontal && props.series.iter().any(Series::is_line);
    use_hook(move || {
        if lying_lines {
            warn(
                "BarChart: a `horizontal` chart draws no lines; its `SeriesKind::Line` series are bars.",
            );
        }
    });
    let mut all_specs = solid(forced);
    let names: Vec<String> = all_specs.iter().map(|s| s.name.clone()).collect();
    let hidden = use_hidden_series(
        &props.hidden,
        &props.default_hidden,
        props.onhiddenchange,
        &names,
    );
    let shown = shown_places(&all_specs, &hidden.get());
    let unit = chart_unit(&props.unit, &props.series);
    // Shares run 0 to 100%: raw bounds would stretch them.
    let ignored_bounds = percent && (props.y_min.is_some() || props.y_max.is_some());
    use_hook(move || {
        if ignored_bounds {
            warn(
                "BarChart: `y_min` and `y_max` do nothing on a `percent` layout, which runs 0 to 100%.",
            );
        }
    });
    let side = |right: bool| -> Vec<Series> {
        props
            .series
            .iter()
            .zip(&all_specs)
            .filter(|(_, spec)| spec.extras.secondary == right)
            .map(|(series, _)| series.clone())
            .collect()
    };
    let (right, left) = (side(true), side(false));
    // Each y axis writes the unit its series share.
    let y2_unit = chart_unit(&None, &right).or(props.unit.clone());
    let lines: Vec<String> = all_specs
        .iter()
        .filter(|spec| spec.extras.line)
        .map(|spec| spec.name.clone())
        .collect();
    // In a stack a label sits on its own segment, never on the next one's.
    let labels_inside = layout != BarLayout::Grouped
        || match props
            .value_label_position
            .as_ref()
            .copied()
            .or(theme.chart.bar_label_position)
        {
            Some(position) => position == BarLabelPosition::Inside,
            // An error bar would cross a label past the bar's end.
            None => horizontal || shown.iter().any(|i| !props.series[*i].errors.is_empty()),
        };
    let palette = chart_palette(theme, forced);
    let color = |color| hex(theme, forced, &palette, 0, Some(color));
    let write = |series: &Series, value: f64| {
        let unit = series.unit.as_deref().or(unit.as_deref());
        props.value_format.format(value, unit, formats)
    };
    // Only upright bars side by side stand at their own values; a line always does.
    let upright_grouped = !horizontal && layout == BarLayout::Grouped;
    let marking: Vec<Series> = props
        .series
        .iter()
        .zip(&all_specs)
        .map(|(series, spec)| Series {
            mark_lines: match spec.extras.line || upright_grouped {
                true => series.mark_lines.clone(),
                false => Vec::new(),
            },
            mark_points: match spec.extras.line {
                true => series.mark_points.clone(),
                false => Vec::new(),
            },
            ..series.clone()
        })
        .collect();
    let mut marked = mark_line_specs(&labels, &mut all_specs, &marking, write);
    marked.extend(mark_point_specs(
        &labels,
        &mut all_specs,
        &marking,
        &x,
        write,
    ));
    let spec = BarSpec {
        width,
        height,
        x: cut(&x, window),
        series: windowed(
            shown.iter().map(|i| all_specs[*i].clone()).collect(),
            window,
        ),
        layout,
        horizontal,
        radius: f32::from(
            theme
                .radius
                .get(props.radius.copied_or(theme.chart.bar_radius)),
        ),
        value_labels: props.value_labels.unwrap_or(theme.chart.value_labels),
        axes: Axes {
            min: props.y_min.filter(|_| !percent),
            max: props.y_max.filter(|_| !percent),
            ticks: props.y_ticks.unwrap_or(theme.chart.y_ticks),
            x_title: props.x_title.clone(),
            y_title: props.y_title.clone(),
            y2_title: props.y2_title.clone(),
            compact: labels.compact,
        },
        labels_inside,
        bar_colors: shown
            .iter()
            .map(|i| {
                let series = &props.series[*i];
                (0..series.values.len())
                    .map(|index| {
                        let negative = series.values[index].is_some_and(|value| value < 0.0);
                        let own = series.colors.get(index).copied().flatten();
                        own.or(series.negative_color.filter(|_| negative))
                            .map(color)
                    })
                    .collect::<Vec<_>>()
            })
            .map(|colors| cut(&colors, window))
            .collect(),
        errors: shown
            .iter()
            .map(|i| cut(&props.series[*i].errors, window))
            .collect(),
        bar_width: props.bar_width.or(theme.chart.bar_width),
        bar_gap: props.bar_gap.unwrap_or(theme.chart.bar_gap),
        grid: props.grid.unwrap_or(theme.chart.grid),
        ink: ChartInk::of(theme, forced),
        format: props.value_format,
        unit: chart_unit(&props.unit, &left),
        y2_unit,
        curve: theme.chart.curve,
        points: theme.chart.points && window.1 - window.0 <= theme.chart.points_limit,
        stroke_width: theme.chart.stroke_width,
        formats,
    };
    // The file keeps the normal palette under forced colours.
    let export = {
        let specs = with_extras(solid(false), &all_specs);
        let spec = BarSpec {
            series: windowed(shown.iter().map(|i| specs[*i].clone()).collect(), window),
            ink: ChartInk::of(theme, false),
            ..spec.clone()
        };
        ExportSource::of(theme, props.title.clone(), move || engine::draw_bar(&spec))
    };
    let drawn = use_memo(use_reactive!(|spec, shown, window| {
        engine::draw_bar(&spec).map(|drawn| drawn.with_series(&shown).with_offset(window.0))
    }));

    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => {
            let mut sentences = summary(&labels, labels.bar, &x, &props.series, write);
            sentences.extend(marked);
            sentences.extend(band_summary(&labels, &props.series, write));
            if !lines.is_empty() {
                sentences.push(fill(labels.lines, &[("names", &lines.join(", "))]));
            }
            if !right.is_empty() {
                let names: Vec<&str> = right.iter().map(|series| series.name.as_str()).collect();
                sentences.push(fill(labels.secondary, &[("names", &names.join(", "))]));
            }
            sentences.extend(axis_titles(
                &labels,
                &props.x_title,
                &props.y_title,
                &props.y2_title.clone().filter(|_| !right.is_empty()),
            ));
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
        series: overlay_series(&all_specs, &props.series, false),
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
            layout: Layout::Columns,
            onrange: props.onrange,
        },
        export,
        handle: props.handle,
        export_button: props.export_button.unwrap_or(theme.chart.export_button),
        zoom,
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

    /// The bars drawn: each is a path.
    fn bars(html: &str) -> usize {
        html.split("data-chart-drawing")
            .nth(1)
            .and_then(|rest| rest.split("</svg>").next())
            .unwrap_or_default()
            .matches("<path")
            .count()
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
        assert_eq!(bars(&html), 3, "{html}");
        assert!(html.contains("Bar chart from Q1 to Q2."), "{html}");
        assert!(html.contains("2025: 3 to 5, highest at Q2."), "{html}");
        assert!(!html.contains("<table"), "{html}");
    }

    /// Without a title the figure is still named, as the plot is; a spread label wins.
    #[test]
    fn an_untitled_figure_is_named_chart_unless_the_page_names_it() {
        let figure = |html: &str| {
            let at = html.find("role=\"figure\"").expect("a figure");
            let from = html[..at].rfind("<div").unwrap_or(0);
            html[from..at + html[at..].find('>').unwrap_or(0)].to_string()
        };
        let plain = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart { x: ["Q1"], series: vec![Series::new("2025", [3.0])] }
                }
            }
        });
        assert!(
            figure(&plain).contains("aria-label=\"chart\""),
            "{}",
            figure(&plain)
        );
        let named = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart {
                        aria_label: "Sales",
                        x: ["Q1"],
                        series: vec![Series::new("2025", [3.0])],
                    }
                }
            }
        });
        assert_eq!(named.matches("aria-label=\"Sales\"").count(), 1, "{named}");
    }

    /// One category is named once, not as a range from it to itself.
    #[test]
    fn one_category_is_not_a_range() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Sales",
                        x: ["Q1"],
                        series: vec![Series::new("2025", [3.0])],
                    }
                }
            }
        });
        assert!(html.contains("Bar chart for Q1."), "{html}");
        assert!(!html.contains("from Q1 to Q1"), "{html}");
    }

    /// Every series hidden: the plot and the summary say so, the legend stays to show one.
    #[test]
    fn every_series_hidden_says_so_and_keeps_the_legend() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Sales",
                        x: ["Q1", "Q2"],
                        series: vec![Series::new("2025", [3.0, 5.0]), Series::new("2026", [4.0, 6.0])],
                        default_hidden: vec!["2025".to_string(), "2026".to_string()],
                    }
                }
            }
        });
        assert_eq!(html.matches("All series are hidden.").count(), 2, "{html}");
        assert!(html.contains("aria-label=\"Series\""), "{html}");
        assert!(
            !html.contains("No data") && !html.contains("data-chart-drawing="),
            "{html}"
        );
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

        assert_eq!(bars(&html), 2, "{html}");
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

        assert_eq!(bars(&html), 4, "{html}");
        assert!(html.contains("75%"), "{html}");
        assert!(html.contains("A: 1 to 3, highest at b."), "{html}");
    }

    /// A percent axis runs 0 to 100% whatever `y_min` and `y_max` say.
    #[test]
    fn a_percent_axis_ignores_the_bounds() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Shares",
                        layout: "percent",
                        x: ["a", "b"],
                        series: vec![Series::new("A", [1.0, 3.0]), Series::new("B", [3.0, 1.0])],
                        y_min: -50.0,
                        y_max: 200.0,
                    }
                }
            }
        });
        let drawing = html.split("data-chart-drawing=").nth(1).unwrap_or_default();
        assert!(drawing.contains("\n100%\n"), "{html}");
        assert!(
            !drawing.contains("\n200%\n") && !drawing.contains("\n-50%\n"),
            "{html}"
        );
    }

    /// Unset, upright bars with error ranges write their values inside, off the error bars.
    #[test]
    fn error_ranges_keep_the_labels_inside() {
        fn chart(position: Option<BarLabelPosition>) -> Element {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Profit",
                        x: ["a", "b"],
                        series: vec![Series::new("A", [3.0, 5.0]).errors([(2.0, 4.0), (4.0, 7.0)])],
                        value_labels: true,
                        value_label_position: position,
                    }
                }
            }
        }
        let drawing = |html: String| {
            html.split("data-chart-drawing")
                .nth(1)
                .unwrap_or_default()
                .to_string()
        };
        let auto_html = render(|| chart(None));
        // The summary says the drawn ranges.
        assert!(
            auto_html.contains("A: each value has a range, 2 to 7 in all."),
            "{auto_html}"
        );
        let auto = drawing(auto_html);
        assert_eq!(
            auto,
            drawing(render(|| chart(Some(BarLabelPosition::Inside))))
        );
        assert_ne!(
            auto,
            drawing(render(|| chart(Some(BarLabelPosition::Outside))))
        );
    }

    /// A line series is drawn as a line over the bars, keyed by a stroke and named in the
    /// summary; lying, it is a bar like the rest.
    #[test]
    fn a_line_series_is_drawn_over_the_bars() {
        fn chart(orientation: Orientation) -> Element {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Sales",
                        x: ["Q1", "Q2"],
                        orientation,
                        y2_title: "Margin",
                        series: vec![
                            Series::new("Revenue", [3.0, 5.0]),
                            Series::new("Margin", [12.0, 18.0])
                                .unit("%")
                                .kind(crate::components::data_display::SeriesKind::Line)
                                .mark_point(crate::components::data_display::MarkPoint::Max)
                                .secondary_axis(),
                        ],
                    }
                }
            }
        }
        let upright = render(|| chart(Orientation::Vertical));
        assert_eq!(bars(&upright), 3, "{upright}");
        assert!(upright.contains("data-line=\"true\""), "{upright}");
        assert!(
            upright.contains("Drawn as a line over the bars: Margin."),
            "{upright}"
        );
        assert!(
            upright.contains("On the right y axis: Margin."),
            "{upright}"
        );
        assert!(upright.contains("Right y axis: Margin."), "{upright}");
        assert!(
            upright.contains("Margin: Highest marked at Q2"),
            "{upright}"
        );

        let lying = render(|| chart(Orientation::Horizontal));
        assert_eq!(bars(&lying), 4, "{lying}");
        assert!(!lying.contains("data-line=\"true\""), "{lying}");
        assert!(!lying.contains("On the right y axis"), "{lying}");
        assert!(!lying.contains("marked at"), "{lying}");
    }

    /// A bar may stand on the right axis; a percent chart keeps its bars on the left.
    #[test]
    fn a_bar_takes_the_right_axis_unless_shares() {
        fn chart(layout: BarLayout) -> Element {
            rsx! {
                LiberoProvider {
                    BarChart {
                        title: "Sales",
                        x: ["Q1", "Q2"],
                        layout,
                        series: vec![
                            Series::new("Revenue", [3.0, 5.0]),
                            Series::new("Visits", [1200.0, 1800.0]).secondary_axis(),
                        ],
                    }
                }
            }
        }
        let grouped = render(|| chart(BarLayout::Grouped));
        assert!(
            grouped.contains("On the right y axis: Visits."),
            "{grouped}"
        );
        let percent = render(|| chart(BarLayout::Percent));
        assert!(!percent.contains("On the right y axis"), "{percent}");
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
